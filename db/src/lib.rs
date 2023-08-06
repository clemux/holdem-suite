use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};

use holdem_suite_parser::parser;
use holdem_suite_parser::parser::ActionType;
use holdem_suite_parser::summary_parser;

use crate::models::{Action, Hand, NewAction, Summary};
use crate::schema::*;

pub mod models;
pub mod schema;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    println!("Connecting to {}", database_url);
    let mut connection = SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    connection
        .batch_execute("PRAGMA journal_mode=WAL;")
        .unwrap();
    connection
}

pub fn insert_summary(conn: &mut SqliteConnection, summary: summary_parser::TournamentSummary) {
    let new_summary = models::NewSummary {
        id: summary.id as i32,
        name: summary.name,
        finish_place: summary.finish_place as i32,
    };
    diesel::insert_into(summaries::table)
        .values(&new_summary)
        .on_conflict_do_nothing()
        .execute(conn)
        .expect("Error saving new summary");
}

pub fn get_summaries(url: &str) -> Vec<Summary> {
    let mut connection = establish_connection(url);
    summaries::dsl::summaries
        .select(Summary::as_select())
        .load(&mut connection)
        .expect("Error loading summaries")
}

pub fn insert_hands(conn: &mut SqliteConnection, hands_vec: Vec<parser::Hand>) -> u32 {
    let mut new_hands: Vec<Hand> = vec![];
    let mut new_actions: Vec<NewAction> = vec![];
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
            street
                .actions
                .iter()
                .filter(|action| {
                    action.action != ActionType::Collect && action.action != ActionType::Shows
                })
                .for_each(|action| {
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
    diesel::insert_or_ignore_into(hands::table)
        .values(&new_hands)
        .execute(conn)
        .expect("Error saving new hands");
    diesel::insert_or_ignore_into(actions::table)
        .values(&new_actions)
        .execute(conn)
        .expect("Error saving new hands");
    new_hands.len() as u32
}

pub fn get_hands(url: &str) -> Vec<Hand> {
    let mut connection = establish_connection(url);
    match hands::dsl::hands
        .select(Hand::as_select())
        .load(&mut connection)
    {
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
    let mut query = hands::dsl::hands.into_boxed();
    if let Some(tournament_id_) = tournament_id {
        query = query.filter(hands::tournament_id.eq(tournament_id_ as i32));
    }
    if let Some(cash_game_name_) = cash_game_name {
        query = query.filter(hands::cash_game_name.eq(cash_game_name_));
    }
    match query
        .order(hands::datetime.desc())
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
    match actions::table
        .filter(actions::hand_id.eq(hand_id))
        .load(&mut connection)
    {
        Ok(actions) => actions,
        Err(e) => {
            println!("Error getting actions: {}", e);
            vec![]
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub name: String,
    pub nb_hands: i64,
}

pub fn get_players(url: &str) -> Result<Vec<Player>, &'static str> {
    let mut connection = establish_connection(url);
    let action_vec: Vec<(String, i64)> = actions::table
        .group_by(actions::dsl::player_name)
        .select((
            actions::dsl::player_name,
            diesel::dsl::count(actions::hand_id),
        ))
        .load(&mut connection)
        .map_err(|_| "Error getting players")?;
    Ok(action_vec
        .iter()
        .map(|(n, c)| Player {
            name: n.to_owned(),
            nb_hands: *c,
        })
        .collect())
}

pub fn get_players_for_table_cash(url: &str, table_name: String) -> Result<Vec<Player>, &str> {
    let mut connection = establish_connection(url);
    let hand_ids = hands::table
        .filter(hands::cash_game_name.eq(table_name))
        .select(Hand::as_select())
        .get_results(&mut connection)
        .map_err(|_| "Error getting players")?;

    let action_vec: Vec<(String, i64)> = Action::belonging_to(&hand_ids)
        .group_by(actions::dsl::player_name)
        .select((
            actions::dsl::player_name,
            diesel::dsl::count(actions::hand_id),
        ))
        .load(&mut connection)
        .map_err(|_| "Error getting players")?;
    Ok(action_vec
        .iter()
        .map(|(n, c)| Player {
            name: n.to_owned(),
            nb_hands: *c,
        })
        .collect())
}

pub fn get_players_for_table_tournament(
    url: &str,
    tournament_id_: i32,
) -> Result<Vec<Player>, &str> {
    let mut connection = establish_connection(url);
    let (hands1, hands2) = diesel::alias!(hands as h1, hands as h2);
    let (actions1, actions2) = diesel::alias!(actions as a1, actions as a2);
    let players_vec: Vec<(String, i64)> = hands1
        .inner_join(actions1.on(hands1.field(hands::id).eq(actions1.field(actions::hand_id))))
        .filter(
            actions1.field(actions::dsl::player_name).eq_any(
                hands2
                    .inner_join(
                        actions2.on(hands2.field(hands::id).eq(actions2.field(actions::hand_id))),
                    )
                    .filter(hands2.field(hands::tournament_id).eq(tournament_id_))
                    .select(actions2.field(actions::player_name))
                    .distinct(),
            ),
        )
        .group_by(actions1.field(actions::dsl::player_name))
        .select((
            actions1.field(actions::dsl::player_name),
            diesel::dsl::count_distinct(hands1.field(hands::id)),
        ))
        .load(&mut connection)
        .map_err(|_| "Error getting players")?;
    Ok(players_vec
        .into_iter()
        .map(|(name, nb_hands)| Player { name, nb_hands })
        .collect())
}
