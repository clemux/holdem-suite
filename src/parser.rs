use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::{alpha1, char, line_ending, not_line_ending, u32};
use nom::combinator::{map, opt, rest};
use nom::number::complete::float;
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple, Tuple};
use nom::{IResult, Parser};

#[derive(Debug, PartialEq)]
struct TournamentInfo {
    name: String,
    buy_in: f32,
    rake: f32,
    level: u32,
}

impl TournamentInfo {
    fn parse(input: &str) -> IResult<&str, TournamentInfo> {
        let name_parser = delimited(tag("\""), take_while(|c: char| c != '"'), tag("\""));

        let buyin_parser = terminated(float, tag("€"));
        let rake_parser = terminated(float, tag("€"));

        let buyin_rake_parser = preceded(
            tag("buyIn: "),
            separated_pair(buyin_parser, tag(" + "), rake_parser),
        );

        let level_parser = preceded(tag("level: "), u32);

        let (input, (name, _, (buy_in, rake), _, level)) = (
            name_parser,
            char(' '),
            buyin_rake_parser,
            char(' '),
            level_parser,
        )
            .parse(input)?;

        Ok((
            input,
            TournamentInfo {
                name: name.to_owned(),
                buy_in,
                rake,
                level,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
enum GameInfo {
    Tournament(TournamentInfo),
    CashGame,
    HoldUp,
}

impl GameInfo {
    fn parse(input: &str) -> IResult<&str, GameInfo> {
        let winamax = tag("Winamax Poker - ");
        let tournament = preceded(
            tag("Tournament "),
            map(TournamentInfo::parse, GameInfo::Tournament),
        );
        let cashgame = map(tag("CashGame"), |_| GameInfo::CashGame);
        let hold_up = map(
            preceded(tag("HOLD-UP"), delimited(tag(" \""), alpha1, char('"'))),
            |_| GameInfo::HoldUp,
        );
        let (input, game_info) =
            preceded(winamax, alt((tournament, cashgame, hold_up))).parse(input)?;
        Ok((input, game_info))
    }
}

#[derive(Debug, PartialEq)]
struct Blinds {
    ante: Option<f32>,
    small_blind: f32,
    big_blind: f32,
}

fn parse_blind(input: &str) -> IResult<&str, f32> {
    let (input, blind) = terminated(float, opt(tag("€"))).parse(input)?;
    Ok((input, blind))
}

// possible inputs are "60/250/500" or "250/500" or "0.01€/0.02€". use parse_blind
impl Blinds {
    fn parse(input: &str) -> IResult<&str, Blinds> {
        let small_big_pair = separated_pair(parse_blind, tag("/"), parse_blind);
        let small_big = map(small_big_pair, |(small_blind, big_blind)| Blinds {
            ante: None,
            small_blind,
            big_blind,
        });

        let ante_blinds_tuple = tuple((
            terminated(parse_blind, tag("/")),
            terminated(parse_blind, tag("/")),
            parse_blind,
        ));

        let ante_blinds = map(ante_blinds_tuple, |(ante, small_blind, big_blind)| Blinds {
            ante: Some(ante),
            small_blind,
            big_blind,
        });
        let (input, blinds) = alt((ante_blinds, small_big)).parse(input)?;
        Ok((input, blinds))
    }
}

#[derive(Debug, PartialEq)]
enum PokerType {
    HoldemNoLimit,
}

// implement PokerType::parse for this input: "Holdem no limit" which is PokerType::HoldemNoLimit
impl PokerType {
    fn parse(input: &str) -> IResult<&str, PokerType> {
        let (input, _) = tag("Holdem no limit").parse(input)?;
        Ok((input, PokerType::HoldemNoLimit))
    }
}

#[derive(Debug, PartialEq)]
struct HandInfo {
    game_info: GameInfo,
    hand_id: String,
    poker_type: PokerType,
    blinds: Blinds,
    datetime: String,
}

// implement HandInfo parse for this input:
// Winamax Poker - Tournament "WESTERN" buyIn: 0.90€ + 0.10€ level: 6 - HandId: #2815488303912976462-15-1684698584 - Holdem no limit (60/250/500) - 2023/05/21 19:49:44 UTC
// use GameInfo::parse, Blinds::parse, PokerType::parse
impl HandInfo {
    fn parse(input: &str) -> IResult<&str, HandInfo> {
        let hand_id = preceded(tag("HandId: #"), take_while(|c: char| c != ' '));
        let datetime = terminated(not_line_ending, line_ending);
        let (input, (game_info, _, hand_id, _, poker_type, _, blinds, _, datetime)) = (
            GameInfo::parse,
            tag(" - "),
            hand_id,
            tag(" - "),
            PokerType::parse,
            char(' '),
            delimited(char('('), Blinds::parse, char(')')),
            tag(" - "),
            datetime,
        )
            .parse(input)?;
        Ok((
            input,
            HandInfo {
                game_info,
                hand_id: hand_id.to_owned(),
                poker_type,
                blinds,
                datetime: datetime.to_owned(),
            },
        ))
    }
}

// input: WESTERN(655531954)#077
fn parse_table_name_tournament(input: &str) -> IResult<&str, TableName> {
    let (input, (name, tournament_id, table_id)) = tuple((
        terminated(take_while(|c| c != '('), tag("(")),
        terminated(u32, tag(")#")),
        u32,
    ))
    .parse(input)?;
    Ok((
        input,
        TableName::Tournament(name.to_owned(), tournament_id, table_id),
    ))
}

#[derive(Debug, PartialEq)]
enum TableName {
    Tournament(String, u32, u32),
    CashGame(String),
}

impl TableName {
    fn parse(input: &str) -> IResult<&str, TableName> {
        let parse_cashgame = map(take_while(|c| c != '\''), |name: &str| {
            TableName::CashGame(name.to_owned())
        });
        let (input, table_name) = delimited(
            tag("'"),
            alt((parse_table_name_tournament, parse_cashgame)),
            tag("'"),
        )
        .parse(input)?;
        Ok((input, table_name))
    }
}

#[derive(Debug, PartialEq)]
enum MoneyType {
    RealMoney,
    PlayMoney,
}

impl MoneyType {
    fn parse(input: &str) -> IResult<&str, MoneyType> {
        let (input, money_type) = alt((tag("real money"), tag("play money"))).parse(input)?;
        Ok((
            input,
            match money_type {
                "real money" => MoneyType::RealMoney,
                "play money" => MoneyType::PlayMoney,
                _ => unreachable!(),
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
struct TableInfo {
    table_name: TableName,
    max_players: u32,
    currency: MoneyType,
    button: u32,
}

impl TableInfo {
    fn parse(input: &str) -> IResult<&str, TableInfo> {
        let (input, (_, table_name, _, max_players, _, currency, _, button, _)) = tuple((
            tag("Table: "),
            TableName::parse,
            tag(" "),
            u32,
            tag("-max "),
            delimited(tag("("), MoneyType::parse, tag(")")),
            tag(" "),
            preceded(tag("Seat #"), u32),
            rest,
        ))
        .parse(input)?;
        Ok((
            input,
            TableInfo {
                table_name,
                max_players,
                currency,
                button,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tournament_info() {
        let input = "\"WESTERN\" buyIn: 0.90€ + 0.10€ level: 6";
        let expected = TournamentInfo {
            name: String::from("WESTERN"),
            buy_in: 0.90,
            rake: 0.10,
            level: 6,
        };
        let (_, actual) = TournamentInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_game_info_tournament() {
        let input = "Winamax Poker - Tournament \"WESTERN\" buyIn: 0.90€ + 0.10€ level: 6";
        let expected = GameInfo::Tournament(TournamentInfo {
            name: String::from("WESTERN"),
            buy_in: 0.90,
            rake: 0.10,
            level: 6,
        });
        let (_, actual) = GameInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_parse_game_info_cashgame() {
        let input = "Winamax Poker - CashGame";
        let expected = GameInfo::CashGame;
        let (_, actual) = GameInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_blind_chips() {
        let input = "60";
        let expected = 60.0;
        assert_eq!(expected, parse_blind(input).unwrap().1);
    }

    #[test]
    fn test_parse_blind_money() {
        let input = "60€";
        let expected = 60.0;
        assert_eq!(expected, parse_blind(input).unwrap().1);
    }

    #[test]
    fn test_parse_blinds_chips() {
        let input = "60/250/500";
        let expected = Blinds {
            ante: Some(60.0),
            small_blind: 250.0,
            big_blind: 500.0,
        };
        let (_, actual) = Blinds::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_blinds_no_ante() {
        let input = "250/500";
        let expected = Blinds {
            ante: None,
            small_blind: 250.0,
            big_blind: 500.0,
        };
        let (_, actual) = Blinds::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hand_info() {
        let input = "Winamax Poker - Tournament \"WESTERN\" buyIn: 0.90€ + 0.10€ level: 6 - HandId: \
        #2815488303912976462-15-1684698584 - Holdem no limit (60/250/500) - 2023/05/21 19:49:44 UTC\n";

        let expected = HandInfo {
            game_info: GameInfo::Tournament(TournamentInfo {
                name: String::from("WESTERN"),
                buy_in: 0.90,
                rake: 0.10,
                level: 6,
            }),
            hand_id: String::from("2815488303912976462-15-1684698584"),
            poker_type: PokerType::HoldemNoLimit,
            blinds: Blinds {
                ante: Some(60.0),
                small_blind: 250.0,
                big_blind: 500.0,
            },
            datetime: String::from("2023/05/21 19:49:44 UTC"),
        };
        let (_, actual) = HandInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hand_info_hold_up() {
        let input = "Winamax Poker - HOLD-UP \"Colorado\" - HandId: #18559747-238220-1687014585 - Holdem no limit (0.01€/0.02€) - 2023/06/17 15:09:45 UTC\n";

        let expected = HandInfo {
            game_info: GameInfo::HoldUp,
            hand_id: String::from("18559747-238220-1687014585"),
            poker_type: PokerType::HoldemNoLimit,
            blinds: Blinds {
                ante: None,
                small_blind: 0.01,
                big_blind: 0.02,
            },
            datetime: String::from("2023/06/17 15:09:45 UTC"),
        };
        let (_, actual) = HandInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_hand_info_cash_game() {
        let input = "Winamax Poker - CashGame - HandId: #18567763-280-1687022958 - Holdem no limit (0.01€/0.02€) - 2023/06/17 17:29:18 UTC\n";
        let expected = HandInfo {
            game_info: GameInfo::CashGame,
            hand_id: String::from("18567763-280-1687022958"),
            poker_type: PokerType::HoldemNoLimit,
            blinds: Blinds {
                ante: None,
                small_blind: 0.01,
                big_blind: 0.02,
            },
            datetime: String::from("2023/06/17 17:29:18 UTC"),
        };

        let (_, actual) = HandInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_table_name_tournament() {
        let input = "'Kill The Fish(651864208)#003'";
        let expected = TableName::Tournament(String::from("Kill The Fish"), 651864208, 3);
        let (_, actual) = TableName::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_table_name_cashgame() {
        let input = "'Nice 17'";
        let expected = TableName::CashGame(String::from("Nice 17"));
        let (_, actual) = TableName::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_table_name_holdup() {
        let input = "'Colorado'";
        let expected = TableName::CashGame(String::from("Colorado"));
        let (_, actual) = TableName::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_table_info_tournament() {
        let input = "Table: 'WESTERN(655531954)#077' 6-max (real money) Seat #3 is the button\n";
        let expected = TableInfo {
            table_name: TableName::Tournament(String::from("WESTERN"), 655531954, 77),
            max_players: 6,
            currency: MoneyType::RealMoney,
            button: 3,
        };
        let (_, actual) = TableInfo::parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
