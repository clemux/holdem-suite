use diesel::prelude::*;
use diesel::result::Error;
use diesel::SqliteConnection;
use serde::{Deserialize, Serialize};

use holdem_suite_parser::parser;
use holdem_suite_parser::parser::ActionType;
use holdem_suite_parser::summary_parser;

use crate::errors::DatabaseError;
use crate::models::{Action, Hand, NewAction, Seat, Summary};
use crate::schema::*;

pub mod errors;
pub mod models;
pub mod schema;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_summary(
    conn: &mut SqliteConnection,
    summary: summary_parser::TournamentSummary,
) -> Result<usize, DatabaseError> {
    let new_summary = Summary {
        id: summary.id as i32,
        name: summary.name,
        buyin: summary.buy_in.buy_in,
        date: summary.date.naive_local(),
        play_time: summary.play_time,
        entries: summary.entries as i32,
        mode: summary.mode,
        tournament_type: summary.tournament_type.to_string(),
        speed: summary.speed,
        finish_place: summary.finish_place as i32,
        won: summary.won,
    };
    Ok(diesel::insert_into(summaries::table)
        .values(&new_summary)
        .on_conflict_do_nothing()
        .execute(conn)?)
}

pub fn get_summaries(conn: &mut SqliteConnection) -> Result<Vec<Summary>, DatabaseError> {
    Ok(summaries::dsl::summaries
        .select(Summary::as_select())
        .load(conn)?)
}

fn get_board_card(hand: &parser::Hand, n: usize) -> Option<String> {
    hand.summary.board.as_ref()?.cards[n]
        .as_ref()
        .map(|card| card.to_string())
}

pub fn insert_hands(
    conn: &mut SqliteConnection,
    hands_vec: Vec<parser::Hand>,
) -> Result<u32, DatabaseError> {
    let mut new_actions: Vec<NewAction> = vec![];
    let mut nb_hands = 0;
    conn.transaction::<_, Error, _>(|conn| {
        for hand in &hands_vec {
            let inserted = diesel::insert_or_ignore_into(hands::table)
                .values(Hand {
                    id: hand.hand_info.hand_id.to_owned(),
                    hole_card_1: hand.dealt_cards.hole_cards.card1.to_string(),
                    hole_card_2: hand.dealt_cards.hole_cards.card2.to_string(),
                    tournament_id: match &hand.table_info.table_name {
                        parser::TableName::Tournament(_, tournament_id_, _) => {
                            Some(*tournament_id_ as i32)
                        }
                        _ => None,
                    },
                    cash_game_name: match &hand.table_info.table_name {
                        parser::TableName::CashGame(name) => Some(name.to_owned()),
                        _ => None,
                    },
                    datetime: hand.hand_info.datetime.to_string(),
                    max_players: hand.table_info.max_players as i32,
                    button: hand.table_info.button as i32,
                    hero: hand.dealt_cards.player_name.to_owned(),
                    ante: hand.hand_info.blinds.ante,
                    small_blind: hand.hand_info.blinds.small_blind,
                    big_blind: hand.hand_info.blinds.big_blind,
                    pot: hand.summary.pot,
                    rake: hand.summary.rake,
                    flop1: get_board_card(hand, 0).to_owned(),
                    flop2: get_board_card(hand, 1).to_owned(),
                    flop3: get_board_card(hand, 2).to_owned(),
                    turn: get_board_card(hand, 3).to_owned(),
                    river: get_board_card(hand, 4).to_owned(),
                })
                .execute(conn)
                .expect("Error saving new hands");
            if inserted == 0 {
                continue;
            }
            nb_hands += 1;
            for seat in hand.seats.iter() {
                let summary_player = hand
                    .summary
                    .players
                    .iter()
                    .find(|player| player.name == seat.player_name);
                let (card1, card2) = match summary_player {
                    Some(player) => match player.hole_cards {
                        Some(ref cards) => {
                            (Some(cards.card1.to_string()), Some(cards.card2.to_string()))
                        }
                        None => (None, None),
                    },
                    None => (None, None),
                };
                diesel::insert_or_ignore_into(seats::table)
                    .values(Seat {
                        hand_id: hand.hand_info.hand_id.to_owned(),
                        player_name: seat.player_name.to_owned(),
                        seat_number: seat.seat_number as i32,
                        stack: seat.stack,
                        bounty: seat.bounty,
                        card1,
                        card2,
                    })
                    .execute(conn)
                    .expect("Error saving seat");
            }
            for street in hand.streets.iter() {
                street
                    .actions
                    .iter()
                    .filter(|action| action.action != ActionType::Shows)
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
        diesel::insert_or_ignore_into(actions::table)
            .values(&new_actions)
            .execute(conn)
            .expect("Error saving new hands");
        Ok(())
    })?;
    Ok(nb_hands)
}

pub fn get_seats(conn: &mut SqliteConnection, hand_id: &str) -> Result<Vec<Seat>, DatabaseError> {
    let seats = seats::dsl::seats
        .filter(seats::dsl::hand_id.eq(hand_id))
        .select(Seat::as_select())
        .load(conn)?;
    Ok(seats)
}

pub fn get_hands(conn: &mut SqliteConnection) -> Result<Vec<Hand>, DatabaseError> {
    Ok(hands::dsl::hands
        .select(Hand::as_select())
        .order(hands::datetime.desc())
        .load(conn)?)
}

pub fn get_hands_for_tournament(
    conn: &mut SqliteConnection,
    id: i32,
) -> Result<Vec<Hand>, DatabaseError> {
    Ok(hands::dsl::hands
        .filter(hands::dsl::tournament_id.eq(id))
        .select(Hand::as_select())
        .order(hands::datetime.desc())
        .load(conn)?)
}

pub fn get_hands_for_player(
    conn: &mut SqliteConnection,
    player_name: &str,
) -> Result<Vec<(Hand, Vec<Action>)>, DatabaseError> {
    let hands = hands::dsl::hands
        .inner_join(seats::dsl::seats)
        .filter(seats::dsl::player_name.eq(player_name))
        .select(Hand::as_select())
        .load(conn)?;
    let actions = Action::belonging_to(&hands)
        .select(Action::as_select())
        .load(conn)?;
    let hand_actions = actions
        .grouped_by(&hands)
        .into_iter()
        .zip(hands)
        .map(|(actions, hand)| (hand, actions))
        .collect::<Vec<(Hand, Vec<Action>)>>();
    Ok(hand_actions)
}

pub fn get_latest_hand(
    conn: &mut SqliteConnection,
    tournament_id: Option<u32>,
    cash_game_name: Option<String>,
) -> Result<Option<Hand>, DatabaseError> {
    let mut query = hands::dsl::hands.into_boxed();
    if let Some(tournament_id_) = tournament_id {
        query = query.filter(hands::tournament_id.eq(tournament_id_ as i32));
    }
    if let Some(cash_game_name_) = cash_game_name {
        query = query.filter(hands::cash_game_name.eq(cash_game_name_));
    }
    Ok(query.order(hands::datetime.desc()).first(conn).optional()?)
}

pub fn get_actions(
    conn: &mut SqliteConnection,
    hand_id: String,
) -> Result<Vec<Action>, DatabaseError> {
    Ok(actions::table
        .filter(actions::hand_id.eq(hand_id))
        .load(conn)?)
}

pub fn get_actions_for_hand(
    conn: &mut SqliteConnection,
    hand_id: &str,
) -> Result<Vec<Action>, DatabaseError> {
    Ok(actions::dsl::actions
        .filter(actions::dsl::hand_id.eq(hand_id))
        .filter(actions::dsl::action_type.ne("collect"))
        .select(Action::as_select())
        .load(conn)?)
}

pub fn get_actions_for_player(
    conn: &mut SqliteConnection,
    player_name: String,
) -> Result<Vec<Action>, DatabaseError> {
    Ok(actions::table
        .filter(actions::player_name.eq(player_name))
        .load(conn)?)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct Player {
    pub name: String,
}

pub fn get_players(conn: &mut SqliteConnection) -> Result<Vec<Player>, DatabaseError> {
    let seats: Vec<String> = seats::table
        .select(seats::dsl::player_name)
        .distinct()
        .load(conn)?;
    Ok(seats.into_iter().map(|name| Player { name }).collect())
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Eq, Hash)]
pub struct TablePlayer {
    pub name: String,
    pub seat_number: i32,
}

pub fn get_players_for_table(
    conn: &mut SqliteConnection,
    tournament_id: Option<i32>,
    cash_game_name: Option<String>,
) -> Result<Vec<TablePlayer>, DatabaseError> {
    let mut query = hands::dsl::hands.into_boxed();
    if let Some(tournament_id) = tournament_id {
        query = query.filter(hands::tournament_id.eq(tournament_id));
    }
    if let Some(cash_game_name) = cash_game_name {
        query = query.filter(hands::cash_game_name.eq(cash_game_name));
    }
    let hand: Hand = query.order(hands::datetime.desc()).first(conn)?;
    let seats = Seat::belonging_to(&hand)
        .select(Seat::as_select())
        .load(conn)?;
    Ok(seats
        .into_iter()
        .map(
            |Seat {
                 player_name,
                 seat_number,
                 ..
             }| TablePlayer {
                name: player_name,
                seat_number,
            },
        )
        .collect())
}
