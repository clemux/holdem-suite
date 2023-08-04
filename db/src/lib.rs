use diesel::prelude::*;
use diesel::SqliteConnection;

use crate::models::{Action, Hand, NewAction, Summary};
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
            tournament_id: match &hand.table_info.table_name {
                parser::TableName::Tournament(_, tournament_id_, _) => Some(*tournament_id_ as i32),
                _ => None,
            },
            cash_game_name: match &hand.table_info.table_name {
                parser::TableName::CashGame(name) => Some(name.to_owned()),
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
                        ActionType::Bet { amount } => Some(amount),
                        ActionType::Call { amount } => Some(amount),
                        ActionType::Raise { amount, .. } => Some(amount),
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
    match hands.select(Hand::as_select()).load(&mut connection) {
        Ok(hands_) => hands_,
        Err(e) => {
            println!("Error getting hands: {}", e);
            vec![]
        }
    }
}

pub fn get_latest_hand(
    url: &str,
    tournament_id: Option<u32>,
    cash_game_name: Option<String>,
) -> Option<Hand> {
    let mut connection = establish_connection(url);
    let mut query = hands.into_boxed();
    if let Some(tournament_id_) = tournament_id {
        query = query.filter(schema::hands::tournament_id.eq(tournament_id_ as i32));
    }
    if let Some(cash_game_name_) = cash_game_name {
        query = query.filter(schema::hands::cash_game_name.eq(cash_game_name_));
    }
    match query
        .order(schema::hands::datetime.desc())
        .first(&mut connection)
        .optional()
    {
        Ok(hand) => hand,
        Err(e) => {
            println!("Error getting latest hand: {}", e);
            None
        }
    }
}

pub fn get_actions(url: &str, hand_id: String) -> Vec<Action> {
    let mut connection = establish_connection(url);
    match schema::actions::table
        .filter(schema::actions::hand_id.eq(hand_id))
        .load(&mut connection)
    {
        Ok(actions) => actions,
        Err(e) => {
            println!("Error getting actions: {}", e);
            vec![]
        }
    }
}
