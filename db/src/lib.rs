use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::{Hand, NewAction, Summary};
use crate::schema::hands::dsl::hands;
use crate::schema::summaries::dsl::summaries;
use holdem_suite_parser::parser;
use holdem_suite_parser::parser::ActionType;
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
    // let new_hands: Vec<Hand> = hands_vec
    let mut new_hands: Vec<Hand> = vec![];
    let mut new_actions: Vec<NewAction> = vec![];
    //     .into_iter()
    //     .map(|h| Hand {
    //         id: h.hand_info.hand_id,
    //         hole_card_1: h.dealt_cards.hole_cards.card1.to_string(),
    //         hole_card_2: h.dealt_cards.hole_cards.card2.to_string(),
    //         tournament_id: match h.table_info.table_name {
    //             parser::TableName::Tournament(_, tournament_id_, _) => Some(tournament_id_ as i32),
    //             _ => None,
    //         },
    //         datetime: h.hand_info.datetime.to_string(),
    //     })
    //     .collect();

    for hand in &hands_vec {
        new_hands.push(Hand {
            id: hand.hand_info.hand_id.to_owned(),
            hole_card_1: hand.dealt_cards.hole_cards.card1.to_string(),
            hole_card_2: hand.dealt_cards.hole_cards.card2.to_string(),
            tournament_id: match hand.table_info.table_name {
                parser::TableName::Tournament(_, tournament_id_, _) => Some(tournament_id_ as i32),
                _ => None,
            },
            datetime: hand.hand_info.datetime.to_string(),
        });
        for street in hand.streets.iter() {
            street.actions.iter().for_each(|action| {
                new_actions.push(NewAction {
                    hand_id: hand.hand_info.hand_id.to_owned(),
                    player_name: action.player_name.to_owned(),
                    action_type: action.action.to_string(),
                    amount: match action.action {
                        ActionType::Bet { amount } => Some(amount.into()),
                        ActionType::Call { amount } => Some(amount.into()),
                        ActionType::Raise { amount, .. } => Some(amount.into()),
                        _ => None,
                    },
                    is_all_in: action.is_all_in as i32,
                    street: street.street_type.to_string(),
                })
            });
        }
    }

    diesel::insert_or_ignore_into(schema::hands::table)
        .values(&new_hands)
        .execute(conn)
        .expect("Error saving new hands");
    diesel::insert_or_ignore_into(schema::actions::table)
        .values(&new_actions)
        .execute(conn)
        .expect("Error saving new hands");
    new_hands.len() as u32
}

pub fn get_hands(url: &str) -> Vec<Hand> {
    let mut connection = establish_connection(url);
    hands
        .select(Hand::as_select())
        .load(&mut connection)
        .expect("Error loading hands")
}
