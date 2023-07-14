use nom::branch::alt;
use nom::bytes::complete::{tag, take_until, take_while};
use nom::character::complete::{alpha1, anychar, char, line_ending, not_line_ending};
use nom::combinator::{map, opt, rest};
use nom::multi::{many1, many_till, separated_list0};
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

        let level_parser = preceded(tag("level: "), nom::character::complete::u32);

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
        terminated(nom::character::complete::u32, tag(")#")),
        nom::character::complete::u32,
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
        let (input, (table_name, _, max_players, currency, _, button, _)) = tuple((
            preceded(tag("Table: "), TableName::parse),
            tag(" "),
            terminated(nom::character::complete::u32, tag("-max ")),
            delimited(tag("("), MoneyType::parse, tag(")")),
            tag(" "),
            preceded(tag("Seat #"), nom::character::complete::u32),
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

#[derive(Debug, PartialEq)]
enum Stack {
    Chips(u32),
    Money(f32),
}

impl Stack {
    fn parse(input: &str) -> IResult<&str, Stack> {
        let stack_chips = map(nom::character::complete::u32, Stack::Chips);
        let stack_money = map(terminated(float, tag("€")), Stack::Money);
        let (input, stack) = alt((stack_money, stack_chips)).parse(input)?;
        Ok((input, stack))
    }
}

#[derive(Debug, PartialEq)]
struct Seat {
    seat_number: u32,
    player_name: String,
    stack: Stack,
    bounty: Option<f32>,
}

// input: Seat 5: WinterSound (20000, 0.45€ bounty)
impl Seat {
    fn parse(input: &str) -> IResult<&str, Seat> {
        let stack_bounty = tuple((
            Stack::parse,
            opt(preceded(tag(", "), terminated(float, tag("€ bounty")))),
        ));
        let (input, (seat_number, _, player_name, _, (stack, bounty))) = tuple((
            preceded(tag("Seat "), nom::character::complete::u32),
            tag(": "),
            take_until(" ("),
            tag(" "),
            delimited(tag("("), stack_bounty, tag(")")),
        ))
        .parse(input)?;
        Ok((
            input,
            Seat {
                seat_number,
                player_name: player_name.to_owned(),
                stack,
                bounty,
            },
        ))
    }
}

fn parse_seats(input: &str) -> IResult<&str, Vec<Seat>> {
    let (input, seats) = many1(terminated(Seat::parse, line_ending)).parse(input)?;
    Ok((input, seats))
}

#[derive(Debug, PartialEq)]
enum AmountType {
    Chips(u32),
    Money(f32),
}

impl AmountType {
    fn parse(input: &str) -> IResult<&str, AmountType> {
        let amount_chips = map(nom::character::complete::u32, AmountType::Chips);
        let amount_money = map(terminated(float, tag("€")), AmountType::Money);
        let (input, amount) = alt((amount_money, amount_chips)).parse(input)?;
        Ok((input, amount))
    }
}

#[derive(Debug, PartialEq)]
enum PostType {
    BigBlind(AmountType),
    SmallBlind(AmountType),
    Ante(AmountType),
}

impl PostType {
    fn parse(input: &str) -> IResult<&str, PostType> {
        let small_blind = map(
            preceded(tag("small blind "), AmountType::parse),
            PostType::SmallBlind,
        );
        let big_blind = map(
            preceded(tag("big blind "), AmountType::parse),
            PostType::BigBlind,
        );
        let ante = map(preceded(tag("ante "), AmountType::parse), PostType::Ante);
        let (input, post_type) = alt((small_blind, big_blind, ante)).parse(input)?;
        Ok((input, post_type))
    }
}

#[derive(Debug, PartialEq)]
enum ActionType {
    Bet {
        amount: AmountType,
    },
    Call {
        amount: AmountType,
    },
    Check,
    Fold,
    Post(PostType),
    Raise {
        to_call: AmountType,
        amount: AmountType,
    },
}

impl ActionType {
    fn parse(input: &str) -> IResult<&str, ActionType> {
        let (input, action_type) = terminated(
            alt((
                map(preceded(tag("posts "), PostType::parse), ActionType::Post),
                map(tag("checks"), |_| ActionType::Check),
                map(tag("folds"), |_| ActionType::Fold),
                map(preceded(tag("calls "), AmountType::parse), |x| {
                    ActionType::Call { amount: x }
                }),
                map(preceded(tag("bets "), AmountType::parse), |x| {
                    ActionType::Bet { amount: x }
                }),
                map(
                    preceded(
                        tag("raises "),
                        tuple((AmountType::parse, tag(" to "), AmountType::parse)),
                    ),
                    |(to_call, _, amount)| ActionType::Raise { to_call, amount },
                ),
            )),
            tag("\n"),
        )(input)?;
        Ok((input, action_type))
    }
}

#[derive(Debug, PartialEq)]
struct Action {
    player_name: String,
    action: ActionType,
    is_all_in: bool,
}

impl Action {
    fn parse(input: &str) -> IResult<&str, Action> {
        let (input, (player_name_vec, action_type)) =
            many_till(anychar, preceded(tag(" "), ActionType::parse))(input)?;
        Ok((
            input,
            Action {
                player_name: player_name_vec.into_iter().collect(),
                action: action_type,
                is_all_in: false,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    fn parse(input: &str) -> IResult<&str, Rank> {
        let (input, rank) = alt((
            map(tag("2"), |_| Rank::Two),
            map(tag("3"), |_| Rank::Three),
            map(tag("4"), |_| Rank::Four),
            map(tag("5"), |_| Rank::Five),
            map(tag("6"), |_| Rank::Six),
            map(tag("7"), |_| Rank::Seven),
            map(tag("8"), |_| Rank::Eight),
            map(tag("9"), |_| Rank::Nine),
            map(tag("T"), |_| Rank::Ten),
            map(tag("J"), |_| Rank::Jack),
            map(tag("Q"), |_| Rank::Queen),
            map(tag("K"), |_| Rank::King),
            map(tag("A"), |_| Rank::Ace),
        ))(input)?;
        Ok((input, rank))
    }

    fn parse2(input: &str) -> IResult<&str, Rank> {
        let (input, rank) = alt((
            map(tag("2"), |_| Rank::Two),
            map(tag("3"), |_| Rank::Three),
            map(tag("4"), |_| Rank::Four),
            map(tag("5"), |_| Rank::Five),
            map(tag("6"), |_| Rank::Six),
            map(tag("7"), |_| Rank::Seven),
            map(tag("8"), |_| Rank::Eight),
            map(tag("9"), |_| Rank::Nine),
            map(tag("Tens"), |_| Rank::Ten),
            map(tag("Ten"), |_| Rank::Ten),
            map(tag("Jacks"), |_| Rank::Jack),
            map(tag("Jack"), |_| Rank::Jack),
            map(tag("Queens"), |_| Rank::Queen),
            map(tag("Queen"), |_| Rank::Queen),
            map(tag("Kings"), |_| Rank::King),
            map(tag("King"), |_| Rank::King),
            map(tag("Aces"), |_| Rank::Ace),
            map(tag("Ace"), |_| Rank::Ace),
        ))(input)?;
        Ok((input, rank))
    }
}

#[derive(Debug, PartialEq)]
enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    fn parse(input: &str) -> IResult<&str, Suit> {
        let (input, suit) = alt((
            map(tag("s"), |_| Suit::Spades),
            map(tag("h"), |_| Suit::Hearts),
            map(tag("d"), |_| Suit::Diamonds),
            map(tag("c"), |_| Suit::Clubs),
        ))(input)?;
        Ok((input, suit))
    }
}

#[derive(Debug, PartialEq)]
struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn parse(input: &str) -> IResult<&str, Card> {
        let (input, (rank, suit)) = tuple((Rank::parse, Suit::parse))(input)?;
        Ok((input, Card { rank, suit }))
    }
}

#[derive(Debug, PartialEq)]
struct HoleCards {
    card1: Card,
    card2: Card,
}

impl HoleCards {
    fn parse(input: &str) -> IResult<&str, HoleCards> {
        let (input, (card1, card2)) = separated_pair(Card::parse, tag(" "), Card::parse)(input)?;
        Ok((input, HoleCards { card1, card2 }))
    }
}

#[derive(Debug, PartialEq)]
struct DealtToHero {
    player_name: String,
    hole_cards: HoleCards,
}

impl DealtToHero {
    fn parse(input: &str) -> IResult<&str, DealtToHero> {
        let hole_cards = delimited(tag(" ["), HoleCards::parse, tag("]"));
        let (input, (player_name_vec, hole_cards)) =
            preceded(tag("Dealt to "), many_till(anychar, hole_cards))(input)?;
        Ok((
            input,
            DealtToHero {
                player_name: player_name_vec.into_iter().collect(),
                hole_cards,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
enum StreetType {
    Preflop,
    Flop,
    Turn,
    River,
}

#[derive(Debug, PartialEq)]
struct Street {
    street_type: StreetType,
    actions: Vec<Action>,
}

impl Street {
    fn parse(input: &str) -> IResult<&str, Street> {
        let street_type = alt((
            map(tag("*** FLOP ***"), |_| StreetType::Flop),
            map(tag("*** TURN ***"), |_| StreetType::Turn),
            map(tag("**** RIVER ***"), |_| StreetType::River),
        ));

        let (input, (street_type, _, _, (actions, _))) = tuple((
            street_type,
            take_until("\n"),
            line_ending,
            many_till(Action::parse, tag("***")),
        ))(input)?;
        Ok((
            input,
            Street {
                street_type,
                actions,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
struct Board {
    cards: Vec<Card>,
}

impl Board {
    fn parse(input: &str) -> IResult<&str, Board> {
        let (input, (_, cards, _)) = tuple((
            tag("Board: ["),
            terminated(separated_list0(tag(" "), Card::parse), tag("]")),
            line_ending,
        ))(input)?;
        Ok((input, Board { cards }))
    }
}

#[derive(Debug, PartialEq)]
enum SummaryResult {
    Won(AmountType),
    Lost,
}

#[derive(Debug, PartialEq)]
enum HandCategory {
    HighCard(Rank),
    Pair(Rank),
    TwoPair(Rank, Rank),
    ThreeOfAKind(Rank),
    Straight(Rank),
    Flush(Rank),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank),
    StraightFlush(Rank),
}

impl HandCategory {
    fn parse(input: &str) -> IResult<&str, HandCategory> {
        let high_card = preceded(
            tag("High card : "),
            map(Rank::parse2, HandCategory::HighCard),
        );
        let pair = preceded(tag("One pair : "), map(Rank::parse2, HandCategory::Pair));

        let two_pairs = preceded(
            tag("Two pairs : "),
            map(
                separated_pair(Rank::parse2, tag(" and "), Rank::parse2),
                |(rank1, rank2)| HandCategory::TwoPair(rank1, rank2),
            ),
        );

        let three_of_a_kind = preceded(
            tag("Trips of "),
            map(Rank::parse2, HandCategory::ThreeOfAKind),
        );

        let four_of_a_kind = preceded(
            tag("Four of a kind : "),
            map(Rank::parse2, HandCategory::FourOfAKind),
        );

        let straight = preceded(tag("Straight "), map(Rank::parse2, HandCategory::Straight));

        let flush = preceded(tag("Flush "), map(Rank::parse2, HandCategory::Flush));

        let straight_flush = preceded(
            tag("Straight flush "),
            map(Rank::parse2, HandCategory::StraightFlush),
        );

        let (input, result_cards) = alt((
            high_card,
            pair,
            two_pairs,
            three_of_a_kind,
            four_of_a_kind,
            straight,
            flush,
            straight_flush,
        ))(input)?;
        Ok((input, result_cards))
    }
}

#[derive(Debug, PartialEq)]
struct SummaryPlayer {
    name: String,
    seat: u32,
    hole_cards: Option<HoleCards>,
    result: SummaryResult,
    hand_category: Option<HandCategory>,
}

impl SummaryPlayer {
    fn parse(input: &str) -> IResult<&str, SummaryPlayer> {
        let position = delimited(tag(" ("), take_until(")"), tag(")"));
        let showed = delimited(tag(" showed ["), HoleCards::parse, tag("] and"));
        let result = alt((
            map(preceded(tag(" won "), AmountType::parse), |amount| {
                SummaryResult::Won(amount)
            }),
            map(tag(" lost"), |_| SummaryResult::Lost),
        ));
        let position_show_result = tuple((
            opt(position),
            opt(showed),
            result,
            opt(preceded(tag(" with "), HandCategory::parse)),
            take_until("\n"),
            tag("\n"),
        ));

        let winner_seat = preceded(tag("Seat "), nom::character::complete::u32);
        let winner_name_vec = preceded(tag(": "), many_till(anychar, position_show_result));
        let (input, (winner_seat, (winner_name_vec, (_, showed, result, hand_category, _, _)))) =
            tuple((winner_seat, winner_name_vec))(input)?;
        Ok((
            input,
            SummaryPlayer {
                name: winner_name_vec.into_iter().collect(),
                seat: winner_seat,
                hole_cards: showed,
                result: result,
                hand_category,
            },
        ))
    }
}

#[derive(Debug, PartialEq)]
struct Summary {
    pot: AmountType,
    rake: Option<AmountType>,
    players: Vec<SummaryPlayer>,
    board: Option<Board>,
}

impl Summary {
    fn parse(input: &str) -> IResult<&str, Summary> {
        let pot_amount = delimited(tag("Total pot "), AmountType::parse, tag(" | "));
        let rake = alt((
            map(preceded(tag("Rake "), AmountType::parse), Some),
            map(tag("No rake"), |_| None),
        ));
        let (input, (pot_amount, rake, _, board, (players, _))) = tuple((
            pot_amount,
            rake,
            tag("\n"),
            opt(Board::parse),
            many_till(SummaryPlayer::parse, tag("\n")),
        ))(input)?;
        Ok((
            input,
            Summary {
                pot: pot_amount,
                rake: rake,
                players,
                board,
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
    fn test_hand_info_tournament() {
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

    #[test]
    fn test_parse_seat_chips_with_bounty() {
        let input = "Seat 5: WinterSound (20000, 0.45€ bounty)\n";
        let expected = Seat {
            seat_number: 5,
            player_name: String::from("WinterSound"),
            stack: Stack::Chips(20000),
            bounty: Some(0.45),
        };
        let (_, actual) = Seat::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_seat_chips_without_bounty() {
        let input = "Seat 3: WinterSound (18744)\n";
        let expected = Seat {
            seat_number: 3,
            player_name: String::from("WinterSound"),
            stack: Stack::Chips(18744),
            bounty: None,
        };
        let (_, actual) = Seat::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_seat_cashgame() {
        let input = "Seat 3: Winter Sound (0.50€)\n";
        let expected = Seat {
            seat_number: 3,
            player_name: String::from("Winter Sound"),
            stack: Stack::Money(0.50),
            bounty: None,
        };
        let (_, actual) = Seat::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_seats() {
        let input = "Seat 1: WinterSound (20000, 0.45€ bounty)\nSeat 2: Player Two (18744)\n";
        let expected = vec![
            Seat {
                seat_number: 1,
                player_name: String::from("WinterSound"),
                stack: Stack::Chips(20000),
                bounty: Some(0.45),
            },
            Seat {
                seat_number: 2,
                player_name: String::from("Player Two"),
                stack: Stack::Chips(18744),
                bounty: None,
            },
        ];
        let (_, actual) = parse_seats(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_post_type_sb() {
        let input = "small blind 250\n";
        let expected = PostType::SmallBlind(AmountType::Chips(250));
        let (_, actual) = PostType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_post_type_bb_cash() {
        let input = "big blind 0.02€\n";
        let expected = PostType::BigBlind(AmountType::Money(0.02));
        let (_, actual) = PostType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_post_type_ante_chips() {
        let input = "big blind 60\n";
        let expected = PostType::BigBlind(AmountType::Chips(60));
        let (_, actual) = PostType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_type_post_bb() {
        let input = "posts big blind 500\n";
        let expected = ActionType::Post(PostType::BigBlind(AmountType::Chips(500)));
        let (_, actual) = ActionType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_type_check() {
        let input = "checks\n";
        let expected = ActionType::Check;
        let (_, actual) = ActionType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_type_call() {
        let input = "calls 500\n";
        let expected = ActionType::Call {
            amount: AmountType::Chips(500),
        };
        let (_, actual) = ActionType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_type_bet() {
        let input = "bets 500\n";
        let expected = ActionType::Bet {
            amount: AmountType::Chips(500),
        };
        let (_, actual) = ActionType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_type_raise() {
        let input = "raises 500 to 1000\n";
        let expected = ActionType::Raise {
            to_call: AmountType::Chips(500),
            amount: AmountType::Chips(1000),
        };
        let (_, actual) = ActionType::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_fold() {
        let input = "As 2 carrot folds\n";
        let expected = Action {
            player_name: String::from("As 2 carrot"),
            action: ActionType::Fold,
            is_all_in: false,
        };
        let (_, actual) = Action::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_check() {
        let input = "Player One checks\n";
        let expected = Action {
            player_name: String::from("Player One"),
            action: ActionType::Check,
            is_all_in: false,
        };
        let (_, actual) = Action::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_action_raises() {
        let input = "Player One raises 500 to 1000\n";
        let expected = Action {
            player_name: String::from("Player One"),
            action: ActionType::Raise {
                to_call: AmountType::Chips(500),
                amount: AmountType::Chips(1000),
            },
            is_all_in: false,
        };
        let (_, actual) = Action::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_dealt_to() {
        let input = "Dealt to Player One [Ks 9s]\n";
        let expected = DealtToHero {
            player_name: String::from("Player One"),
            hole_cards: HoleCards {
                card1: Card {
                    rank: Rank::King,
                    suit: Suit::Spades,
                },
                card2: Card {
                    rank: Rank::Nine,
                    suit: Suit::Spades,
                },
            },
        };
        let (_, actual) = DealtToHero::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_street() {
        let input =
            "*** FLOP *** [8s 7h 4h]\nPlayer One raises 500 to 1000\nPlayer Two calls 1000\n***";
        let expected = Street {
            street_type: StreetType::Flop,
            actions: vec![
                Action {
                    player_name: String::from("Player One"),
                    action: ActionType::Raise {
                        to_call: AmountType::Chips(500),
                        amount: AmountType::Chips(1000),
                    },
                    is_all_in: false,
                },
                Action {
                    player_name: String::from("Player Two"),
                    action: ActionType::Call {
                        amount: AmountType::Chips(1000),
                    },
                    is_all_in: false,
                },
            ],
        };
        let (_, actual) = Street::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_board() {
        let input = "Board: [8s 7h 4h 3s 2h]\n";
        let expected = Board {
            cards: vec![
                Card {
                    rank: Rank::Eight,
                    suit: Suit::Spades,
                },
                Card {
                    rank: Rank::Seven,
                    suit: Suit::Hearts,
                },
                Card {
                    rank: Rank::Four,
                    suit: Suit::Hearts,
                },
                Card {
                    rank: Rank::Three,
                    suit: Suit::Spades,
                },
                Card {
                    rank: Rank::Two,
                    suit: Suit::Hearts,
                },
            ],
        };
        let (_, actual) = Board::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_hand_category_high() {
        let input = "High card : Ace";
        let expected = HandCategory::HighCard(Rank::Ace);
        let (_, actual) = HandCategory::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_hand_category_pair() {
        let input = "One pair : Aces";
        let expected = HandCategory::Pair(Rank::Ace);
        let (_, actual) = HandCategory::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_hand_category_two_pair() {
        let input = "Two pairs : Queens and 2";
        let expected = HandCategory::TwoPair(Rank::Queen, Rank::Two);
        let (_, actual) = HandCategory::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_hand_category_flush() {
        let input = "Flush Jack high";
        let expected = HandCategory::Flush(Rank::Jack);
        let (_, actual) = HandCategory::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_hand_category_straight() {
        let input = "Straight Ten high";
        let expected = HandCategory::Straight(Rank::Ten);
        let (_, actual) = HandCategory::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_player() {
        let input = "Seat 6: Alexarango (button) won 0.31€\n";
        let expected = SummaryPlayer {
            seat: 6,
            name: String::from("Alexarango"),
            result: SummaryResult::Won(AmountType::Money(0.31)),
            hole_cards: None,
            hand_category: None,
        };
        let (_, actual) = SummaryPlayer::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_player_showdown() {
        let input =
            "Seat 6: Alexarango (button) showed [8d Td] and won 0.36€ with Straight Ten high\n";
        let expected = SummaryPlayer {
            seat: 6,
            name: String::from("Alexarango"),
            result: SummaryResult::Won(AmountType::Money(0.36)),
            hole_cards: Some(HoleCards {
                card1: Card {
                    rank: Rank::Eight,
                    suit: Suit::Diamonds,
                },
                card2: Card {
                    rank: Rank::Ten,
                    suit: Suit::Diamonds,
                },
            }),
            hand_category: Some(HandCategory::Straight(Rank::Ten)),
        };
        let (_, actual) = SummaryPlayer::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_player_showdown_lost() {
        let input =
            "Seat 3: Player Two showed [Qd As] and won 0.36€ with Two pairs : Queens and 2\n";
        let expected = SummaryPlayer {
            seat: 3,
            name: String::from("Player Two"),
            result: SummaryResult::Won(AmountType::Money(0.36)),
            hole_cards: Some(HoleCards {
                card1: Card {
                    rank: Rank::Queen,
                    suit: Suit::Diamonds,
                },
                card2: Card {
                    rank: Rank::Ace,
                    suit: Suit::Spades,
                },
            }),
            hand_category: Some(HandCategory::TwoPair(Rank::Queen, Rank::Two)),
        };
        let (_, actual) = SummaryPlayer::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_no_flop_no_rake() {
        let input = "Total pot 2670 | No rake\nSeat 3: Player One won 2670\n\n";
        let expected = Summary {
            pot: AmountType::Chips(2670),
            rake: None,
            players: vec![SummaryPlayer {
                seat: 3,
                name: String::from("Player One"),
                result: SummaryResult::Won(AmountType::Chips(2670)),
                hole_cards: None,
                hand_category: None,
            }],
            board: None,
        };
        let (_, actual) = Summary::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_with_rake() {
        let input =
            "Total pot 0.79€ | Rake 0.01€\nBoard: [8c 5h Ts Kd Td]\nSeat 3: Player One won 0.79€\n\n";
        let expected = Summary {
            pot: AmountType::Money(0.79),
            rake: Some(AmountType::Money(0.01)),
            players: vec![SummaryPlayer {
                seat: 3,
                name: String::from("Player One"),
                result: SummaryResult::Won(AmountType::Money(0.79)),
                hole_cards: None,
                hand_category: None,
            }],
            board: Some(Board {
                cards: vec![
                    Card {
                        rank: Rank::Eight,
                        suit: Suit::Clubs,
                    },
                    Card {
                        rank: Rank::Five,
                        suit: Suit::Hearts,
                    },
                    Card {
                        rank: Rank::Ten,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::King,
                        suit: Suit::Diamonds,
                    },
                    Card {
                        rank: Rank::Ten,
                        suit: Suit::Diamonds,
                    },
                ],
            }),
        };
        let (_, actual) = Summary::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_with_board() {
        let input =
            "Total pot 2670 | No rake\nBoard: [8s 7h 4h 3s 2h]\nSeat 3: Player One won 2670\n\n";
        let expected = Summary {
            pot: AmountType::Chips(2670),
            rake: None,
            players: vec![SummaryPlayer {
                seat: 3,
                name: String::from("Player One"),
                result: SummaryResult::Won(AmountType::Chips(2670)),
                hole_cards: None,
                hand_category: None,
            }],
            board: Some(Board {
                cards: vec![
                    Card {
                        rank: Rank::Eight,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::Seven,
                        suit: Suit::Hearts,
                    },
                    Card {
                        rank: Rank::Four,
                        suit: Suit::Hearts,
                    },
                    Card {
                        rank: Rank::Three,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::Two,
                        suit: Suit::Hearts,
                    },
                ],
            }),
        };
        let (_, actual) = Summary::parse(input).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_parse_summary_with_showdown() {
        let input = concat!(
            "Total pot 0.30€ | Rake 0.03€\n",
            "Board: [3s Ks Qh 2s 2c]\n",
            "Seat 2: Player One (big blind) showed [9c Kd] and won ",
            "0.30€ with One pair : Kings\n",
            "Seat 3: Player Two showed [Qd As] and lost with Two pairs : Queens and 2\n\n"
        );

        let expected = Summary {
            pot: AmountType::Money(0.30),
            rake: Some(AmountType::Money(0.03)),
            players: vec![
                SummaryPlayer {
                    seat: 2,
                    name: String::from("Player One"),
                    result: SummaryResult::Won(AmountType::Money(0.30)),
                    hole_cards: Some(HoleCards {
                        card1: Card {
                            rank: Rank::Nine,
                            suit: Suit::Clubs,
                        },
                        card2: Card {
                            rank: Rank::King,
                            suit: Suit::Diamonds,
                        },
                    }),
                    hand_category: Some(HandCategory::Pair(Rank::King)),
                },
                SummaryPlayer {
                    seat: 3,
                    name: String::from("Player Two"),
                    result: SummaryResult::Lost,
                    hole_cards: Some(HoleCards {
                        card1: Card {
                            rank: Rank::Queen,
                            suit: Suit::Diamonds,
                        },
                        card2: Card {
                            rank: Rank::Ace,
                            suit: Suit::Spades,
                        },
                    }),
                    hand_category: Some(HandCategory::TwoPair(Rank::Queen, Rank::Two)),
                },
            ],
            board: Some(Board {
                cards: vec![
                    Card {
                        rank: Rank::Three,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::King,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::Queen,
                        suit: Suit::Hearts,
                    },
                    Card {
                        rank: Rank::Two,
                        suit: Suit::Spades,
                    },
                    Card {
                        rank: Rank::Two,
                        suit: Suit::Clubs,
                    },
                ],
            }),
        };
        let (_, actual) = Summary::parse(input).unwrap();
        assert_eq!(expected, actual);
    }
}
