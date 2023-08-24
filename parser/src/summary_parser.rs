use chrono::prelude::*;

use nom;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::{line_ending, not_line_ending};
use nom::combinator::{map, map_res, opt};
use nom::multi::separated_list1;
use nom::number::complete::double;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
enum PokerType {
    HoldemNoLimit,
    OmahaPotLimit,
}

impl PokerType {
    fn parse(input: &str) -> IResult<&str, PokerType> {
        let (input, _) = alt((
            map(tag("holdem-no-limit"), |_| PokerType::HoldemNoLimit),
            map(tag("omaha-pot-limit"), |_| PokerType::OmahaPotLimit),
        ))(input)?;
        Ok((input, PokerType::HoldemNoLimit))
    }
}

#[derive(Debug, PartialEq)]
pub struct Level {
    ante: u32,
    small_blind: u32,
    big_blind: u32,
    seconds: u32,
    poker_type: PokerType,
}

impl Level {
    fn parse(input: &str) -> IResult<&str, Level> {
        let (input, (small_blind, _, big_blind, ante, seconds, poker_type)) = tuple((
            nom::character::complete::u32,
            tag("-"),
            nom::character::complete::u32,
            preceded(tag(":"), nom::character::complete::u32),
            preceded(tag(":"), nom::character::complete::u32),
            preceded(tag(":"), PokerType::parse),
        ))(input)?;

        Ok((
            input,
            Level {
                ante: ante,
                small_blind,
                big_blind,
                seconds,
                poker_type,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
pub enum TournamentType {
    DoubleOrNothing,
    Freeroll100k,
    HitnRun,
    Knockout,
    MadTilt,
    Normal,
    Qualifier,
    Sitngo,
    Unknown(String),
}

impl Display for TournamentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TournamentType::DoubleOrNothing => write!(f, "don"),
            TournamentType::Freeroll100k => write!(f, "freeroll100k"),
            TournamentType::HitnRun => write!(f, "hitnrun"),
            TournamentType::Knockout => write!(f, "knockout"),
            TournamentType::MadTilt => write!(f, "madtilt"),
            TournamentType::Normal => write!(f, "normal"),
            TournamentType::Qualifier => write!(f, "qualifier"),
            TournamentType::Sitngo => write!(f, "sitngo"),
            TournamentType::Unknown(s) => write!(f, "unknown-{}", s),
        }
    }
}

impl TournamentType {
    fn parse(input: &str) -> IResult<&str, TournamentType> {
        let (input, tournament_type) = alt((
            map(tag("doubleornothing"), |_| TournamentType::DoubleOrNothing),
            map(tag("freeroll100k"), |_| TournamentType::Freeroll100k),
            map(tag("hitnrun"), |_| TournamentType::HitnRun),
            map(tag("knockout"), |_| TournamentType::Knockout),
            map(tag("madtilt"), |_| TournamentType::MadTilt),
            map(tag("normal"), |_| TournamentType::Normal),
            map(tag("qualifier"), |_| TournamentType::Qualifier),
            map(tag("sitngo"), |_| TournamentType::Sitngo),
            map(take_until("\n"), |s: &str| {
                TournamentType::Unknown(s.to_string())
            }),
        ))(input)?;
        Ok((input, tournament_type))
    }
}

#[derive(Debug, PartialEq)]
pub struct TournamentSummary {
    pub name: String,
    pub hero: String,
    pub id: u32,
    pub buy_in: BuyIn,
    pub entries: u32,
    pub mode: String,
    pub tournament_type: TournamentType,
    pub speed: String,
    pub flight_id: u32,
    pub levels: Vec<Level>,
    pub prizepool: f64,
    pub date: DateTime<Utc>,
    pub play_time: String,
    pub finish_place: u32,
    pub won: Option<f64>,
}

#[derive(Debug, PartialEq)]
pub struct BuyIn {
    pub buy_in: f64,
    pub rake: f64,
    pub bounty: Option<f64>,
}

fn parse_amount(input: &str) -> IResult<&str, f64> {
    let (input, amount) = terminated(double, opt(tag("€")))(input)?;
    Ok((input, amount))
}

impl BuyIn {
    fn parse(input: &str) -> IResult<&str, BuyIn> {
        let buyin_rake = separated_pair(parse_amount, tag(" + "), parse_amount);
        let bounty = preceded(tag(" + "), parse_amount);
        let (input, ((buy_in, rake), bounty)) = tuple((buyin_rake, opt(bounty)))(input)?;
        Ok((
            input,
            BuyIn {
                buy_in,
                rake,
                bounty,
            },
        ))
    }
}

impl TournamentSummary {
    pub fn parse(input: &str) -> IResult<&str, TournamentSummary> {
        let tournament_start = delimited(
            tag("Tournament started "),
            map_res(not_line_ending, |s: &str| {
                Utc.datetime_from_str(s, "%Y/%m/%d %H:%M:%S %Z")
            }),
            line_ending,
        );
        let (
            input,
            (
                _,
                name,
                id,
                _,
                hero,
                buy_in,
                entries,
                mode,
                tournament_type,
                speed,
                flight_id,
                levels,
                prizepool,
                start,
                play_time,
                finish_place,
                won,
            ),
        ) = tuple((
            tag("Winamax Poker - Tournament summary : "),
            take_until("("),
            delimited(tag("("), nom::character::complete::u32, tag(")")),
            terminated(take_until("\n"), line_ending),
            delimited(tag("Player : "), take_until("\n"), line_ending),
            delimited(tag("Buy-In : "), BuyIn::parse, line_ending),
            delimited(
                tag("Registered players : "),
                nom::character::complete::u32,
                line_ending,
            ),
            delimited(tag("Mode : "), take_until("\n"), line_ending),
            delimited(tag("Type : "), TournamentType::parse, line_ending),
            delimited(tag("Speed : "), take_until("\n"), line_ending),
            delimited(
                tag("Flight ID : "),
                nom::character::complete::u32,
                line_ending,
            ),
            delimited(
                tag("Levels : Levels : ["),
                separated_list1(tag(","), Level::parse),
                tag("]\n"),
            ),
            delimited(
                tag("Prizepool : "),
                terminated(double, opt(tag("€"))),
                line_ending,
            ),
            tournament_start,
            delimited(tag("You played "), take_until("\n"), line_ending),
            delimited(
                tag("You finished in "),
                nom::character::complete::u32,
                alt((
                    tag("th place\n"),
                    tag("st place\n"),
                    tag("nd place\n"),
                    tag("rd place\n"),
                )),
            ),
            opt(preceded(tag("You won "), parse_amount)),
        ))(input)?;

        Ok((
            input,
            TournamentSummary {
                name: name.to_owned(),
                id,
                hero: hero.to_owned(),
                buy_in,
                entries,
                mode: mode.to_owned(),
                tournament_type: tournament_type,
                speed: speed.to_owned(),
                flight_id,
                levels,
                prizepool,
                date: start.to_owned(),
                play_time: play_time.to_owned(),
                finish_place,
                won,
            },
        ))
    }
}

// add unit tests for TournamentSummary::parse, Level::parse, PokerType::parse, TournamentType::parse
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_poker_type() {
        assert_eq!(
            PokerType::parse("holdem-no-limit"),
            Ok(("", PokerType::HoldemNoLimit))
        );
    }

    #[test]
    fn test_parse_level() {
        assert_eq!(
            Level::parse("10-20:0:600:holdem-no-limit"),
            Ok((
                "",
                Level {
                    ante: 0,
                    small_blind: 10,
                    big_blind: 20,
                    seconds: 600,
                    poker_type: PokerType::HoldemNoLimit,
                }
            ))
        );
    }

    #[test]
    fn test_parse_buyin() {
        let input = "0.45€ + 0.05€";
        let (_, buy_in) = BuyIn::parse(input).unwrap();
        assert_eq!(
            buy_in,
            BuyIn {
                buy_in: 0.45,
                rake: 0.05,
                bounty: None,
            }
        );
    }

    #[test]
    fn test_parse_buyin_bounty() {
        let input = "0.60€ + 0.30€ + 0.10€";
        let (_, buy_in) = BuyIn::parse(input).unwrap();
        assert_eq!(
            buy_in,
            BuyIn {
                buy_in: 0.60,
                rake: 0.30,
                bounty: Some(0.10),
            }
        );
    }

    #[test]
    fn test_parse_tournament_type() {
        assert_eq!(
            TournamentType::parse("sitngo"),
            Ok(("", TournamentType::Sitngo))
        );
        assert_eq!(
            TournamentType::parse("normal"),
            Ok(("", TournamentType::Normal))
        );
    }

    #[test]
    fn test_parse_tournament_summary() {
        let input = include_str!("../samples/tournament_summary.txt");
        let (_, tournament_summary) = TournamentSummary::parse(input).unwrap();
        let expected = TournamentSummary {
            name: String::from("MYSTERY KO"),
            id: 669464094,
            hero: String::from("WinterSound"),
            buy_in: BuyIn {
                buy_in: 0.60,
                rake: 0.30,
                bounty: Some(0.10),
            },
            entries: 160,
            mode: String::from("tt"),
            tournament_type: TournamentType::Knockout,
            speed: String::from("normal"),
            flight_id: 0,
            levels: vec![
                Level {
                    ante: 25,
                    small_blind: 100,
                    big_blind: 200,
                    seconds: 2100,
                    poker_type: PokerType::HoldemNoLimit,
                },
                Level {
                    ante: 30,
                    small_blind: 125,
                    big_blind: 250,
                    seconds: 420,
                    poker_type: PokerType::HoldemNoLimit,
                },
            ],
            prizepool: 198.70,
            date: Utc.with_ymd_and_hms(2023, 7, 8, 11, 30, 0).unwrap(),
            play_time: String::from("20min 52s "),
            finish_place: 145,
            won: Some(1.0),
        };
        assert_eq!(tournament_summary, expected);
    }
}
