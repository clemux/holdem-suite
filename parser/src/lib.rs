use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::{NewHand, Summary};
use crate::parser::Hand;

use self::schema::summaries::dsl::*;

pub mod models;
pub mod parser;
pub mod schema;
pub mod summary_parser;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    println!("Connecting to {}", database_url);
    SqliteConnection::establish(&database_url)
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
    let results = summaries
        .select(models::Summary::as_select())
        .load(&mut connection)
        .expect("Error loading summaries");
    results
}

pub fn insert_hands(conn: &mut SqliteConnection, hands: Vec<Hand>) {
    let new_hands: Vec<NewHand> = hands
        .into_iter()
        .map(|h| NewHand {
            id: h.hand_info.hand_id,
            hole_card_1: h.dealt_cards.hole_cards.card1.to_string(),
            hole_card_2: h.dealt_cards.hole_cards.card2.to_string(),
            tournament_id: match h.table_info.table_name {
                parser::TableName::Tournament(_, tournament_id, _) => {
                    Some(Some(tournament_id as i32))
                }
                _ => Some(None),
            },
        })
        .collect();

    for hand in &new_hands {
        diesel::insert_into(schema::hands::table)
            .values(hand)
            .on_conflict_do_nothing()
            .execute(conn)
            .expect("Error saving new hands");
    }
}
