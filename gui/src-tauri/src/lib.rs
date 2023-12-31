use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use diesel::SqliteConnection;
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, tuple};
use nom::{
    bytes::complete::{tag, take_while},
    error::Error,
    IResult, Parser,
};
use serde::{Deserialize, Serialize};

use holdem_suite_db::models::Action;
use holdem_suite_db::{get_latest_hand, insert_hands, insert_summary};
use holdem_suite_parser::parser::parse_hands;
use holdem_suite_parser::summary_parser::TournamentSummary;

use crate::errors::ApplicationError;

pub mod errors;
#[cfg_attr(target_os = "linux", path = "window_management_x11.rs")]
#[cfg_attr(target_os = "windows", path = "window_management_windows.rs")]
pub mod window_management;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, Hash)]
pub enum Table {
    CashGame(String),
    Tournament { name: String, id: u32, table: u32 },
    PendingTournament { name: String, id: u32 },
}

fn parse_tournament(input: &str) -> IResult<&str, Table> {
    let (input, (name, tournament_id, table_id)) = tuple((
        take_while(|c| c != '('),
        delimited(tag("("), nom::character::complete::u32, tag(")")),
        opt(delimited(
            tag("(#"),
            nom::character::complete::u32,
            tag(")"),
        )),
    ))
    .parse(input)?;
    match table_id {
        Some(table_id) => Ok((
            input,
            Table::Tournament {
                name: name.to_owned(),
                id: tournament_id,
                table: table_id,
            },
        )),
        None => Ok((
            input,
            Table::PendingTournament {
                name: name.to_owned(),
                id: tournament_id,
            },
        )),
    }
}

impl FromStr for Table {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_cash_game = map(take_while(|c: char| c != '\n'), |name: &str| {
            Table::CashGame(name.to_string())
        });
        let mut table = preceded(tag("Winamax "), alt((parse_tournament, parse_cash_game)));
        let (_, table) = table.parse(s).unwrap();
        Ok(table)
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PlayerMetrics {
    pub vpip: bool,
    pub pfr: bool,
    pub three_bet: bool,
    pub open_limp: bool,
}

pub fn compute_hand_metrics(actions: Vec<Action>) -> HashMap<String, PlayerMetrics> {
    let mut metrics = HashMap::new();
    let mut someone_limped = false;
    let mut someone_raised = false;
    let mut someone_three_bet = false;
    for Action {
        player_name,
        street,
        action_type,
        ..
    } in actions
    {
        let metric = metrics.entry(player_name).or_default();
        if street != "preflop" {
            return metrics;
        }
        match action_type.as_str() {
            "raise" => {
                if someone_raised && !someone_three_bet {
                    metric.three_bet = true;
                    someone_three_bet = true;
                }
                someone_raised = true;
                metric.vpip = true;
                metric.pfr = true;
            }
            "call" => {
                metric.vpip = true;
                if !someone_raised && !someone_limped {
                    metric.open_limp = true;
                    someone_limped = true;
                }
            }
            "fold" => {}
            _ => {}
        }
    }
    metrics
}

pub fn parse_file(
    path: PathBuf,
    connection: &mut SqliteConnection,
) -> Result<u32, ApplicationError> {
    let path_cloned = path.clone();
    let path_str = path_cloned.to_str().unwrap();
    if path.clone().to_str().unwrap().contains("summary") {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let parse_result = TournamentSummary::parse(&data);
        match parse_result {
            Ok((_, summary)) => {
                let _ = insert_summary(connection, summary)?;
                Ok(0)
            }
            Err(_) => {
                println!("Error parsing {}", path_str);
                Ok(0)
            }
        }
    } else {
        println!("Parsing {}", path_str);
        let data = fs::read_to_string(path).expect("Unable to read file");
        let start = Instant::now();
        let parse_result = parse_hands(&data);
        match parse_result {
            Ok((_, hands)) => {
                let nb_hands = insert_hands(connection, hands)?;
                println!("Parsed {} hands in {:?}", nb_hands, start.elapsed());
                Ok(nb_hands)
            }
            Err(_) => {
                println!("Error parsing {}", path_str);
                Ok(0)
            }
        }
    }
}

pub fn get_table_max_players_and_hero(
    conn: &mut SqliteConnection,
    table: Table,
) -> Result<Option<(i32, String)>, ApplicationError> {
    let hand = match table {
        Table::CashGame(name) => Some(get_latest_hand(conn, None, Some(name.clone()))?),
        Table::Tournament { id, .. } => Some(get_latest_hand(conn, Some(id), None)?),
        _ => None,
    };
    match hand {
        Some(Some(hand)) => Ok(Some((hand.max_players, hand.hero.to_owned()))),
        Some(None) => Ok(None),
        None => Err(ApplicationError::GetTableMaxPlayers),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hand_metrics() {
        let actions = vec![
            Action {
                id: 0,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 1".to_owned(),
                street: "preflop".to_owned(),
                action_type: "raise".to_owned(),
                amount: Some(1.0),
            },
            Action {
                id: 1,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 2".to_owned(),
                street: "preflop".to_owned(),
                action_type: "call".to_owned(),
                amount: Some(1.0),
            },
            Action {
                id: 2,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 3".to_owned(),
                street: "preflop".to_owned(),
                action_type: "fold".to_owned(),
                amount: None,
            },
            Action {
                id: 3,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 4".to_owned(),
                street: "preflop".to_owned(),
                action_type: "raise".to_owned(),
                amount: Some(3.0),
            },
            Action {
                id: 4,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 1".to_owned(),
                street: "preflop".to_owned(),
                action_type: "fold".to_owned(),
                amount: None,
            },
            Action {
                id: 5,
                hand_id: "whatever".to_owned(),
                is_all_in: 0,
                player_name: "Player 1".to_owned(),
                street: "preflop".to_owned(),
                action_type: "fold".to_owned(),
                amount: None,
            },
        ];
        let metrics = compute_hand_metrics(actions);
        assert!(metrics["Player 1"].vpip);
        assert!(metrics["Player 1"].pfr);
        assert!(!metrics["Player 1"].three_bet);
        assert!(metrics["Player 2"].vpip);
        assert!(!metrics["Player 2"].pfr);
        assert!(!metrics["Player 2"].three_bet);
        assert!(!metrics["Player 3"].vpip);
        assert!(!metrics["Player 3"].pfr);
        assert!(!metrics["Player 3"].three_bet);
        assert!(metrics["Player 4"].vpip);
        assert!(metrics["Player 4"].pfr);
        assert!(metrics["Player 4"].three_bet);
    }

    #[test]
    fn test_parse_tournament() {
        let input = "Winamax Monster Stack(676539671)(#0001)";
        let table = Table::from_str(input).unwrap();
        assert_eq!(
            table,
            Table::Tournament {
                name: "Monster Stack".to_owned(),
                id: 676539671,
                table: 1
            }
        );
    }

    #[test]
    fn test_parse_cash_game() {
        let input = "Winamax Wichita 05";
        let table = Table::from_str(input).unwrap();
        assert_eq!(table, Table::CashGame("Wichita 05".to_owned()));
    }
}
