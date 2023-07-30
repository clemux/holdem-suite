use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::{Hand, NewHand, Summary};
use crate::schema::hands::dsl::hands;
use crate::schema::summaries::dsl::summaries;
use holdem_suite_parser::parser;
use holdem_suite_parser::summary_parser;

pub mod models;
pub mod schema;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    println!("Connecting to {}", database_url);
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_summary(conn: &mut SqliteConnection, summary: summary_parser::TournamentSummary) {
    let new_summary = models::NewSummary {
        id: summary.id as i32,
        name: summary.name,
        finish_place: summary.finish_place as i32,
    };
    diesel::insert_into(schema::summaries::table)
        .values(&new_summary)
        .on_conflict_do_nothing()
        .execute(conn)
        .expect("Error saving new summary");
}

pub fn get_summaries(url: &str) -> Vec<Summary> {
    let mut connection = establish_connection(url);
    summaries
        .select(Summary::as_select())
        .load(&mut connection)
        .expect("Error loading summaries")
}

pub fn insert_hands(conn: &mut SqliteConnection, hands_vec: Vec<parser::Hand>) -> u32 {
    let new_hands: Vec<NewHand> = hands_vec
        .into_iter()
        .map(|h| NewHand {
            id: h.hand_info.hand_id,
            hole_card_1: h.dealt_cards.hole_cards.card1.to_string(),
            hole_card_2: h.dealt_cards.hole_cards.card2.to_string(),
            tournament_id: match h.table_info.table_name {
                parser::TableName::Tournament(_, tournament_id_, _) => Some(tournament_id_ as i32),
                _ => None,
            },
            datetime: h.hand_info.datetime.to_string(),
        })
        .collect();

    for hand in &new_hands {
        diesel::insert_into(schema::hands::table)
            .values(hand)
            .on_conflict_do_nothing()
            .execute(conn)
            .expect("Error saving new hands");
    }
    new_hands.len() as u32
}

pub fn get_hands(url: &str) -> Vec<Hand> {
    let mut connection = establish_connection(url);
    hands
        .select(Hand::as_select())
        .load(&mut connection)
        .expect("Error loading hands")
}
