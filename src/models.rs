// #[derive(Debug, PartialEq)]
// pub struct Game {
//     pub name: String,
//     pub buy_in: f32,
//     pub rake: f32,
// }
//
// struct Seat {
//     id: u32,
//     player_name: String,
//     stack: u32,
//     bounty: Option<f32>,
// }
//
// enum RealCurrency {
//     EUR,
// }
//
// enum Currency {
//     RealMoney(RealCurrency),
//     PlayMoney,
// }
//
// struct Table {
//     game: Game,
//     table_name: String,
//     max_players: u8,
//     button_position: u8,
//     small_blind: u32,
//     big_blind: u32,
//     ante: u32,
//     currency: Currency,
//     game_type: String,
//     seats: Vec<Seat>,
// }
//
// enum Street {
//     AnteBlinds,
//     Preflop,
//     Flop,
//     Turn,
//     River,
//     Showdown,
// }
//
// enum Rank {
//     Two,
//     Three,
//     Four,
//     Five,
//     Six,
//     Seven,
//     Eight,
//     Nine,
//     Ten,
//     Jack,
//     Queen,
//     King,
//     Ace,
// }
//
// enum Suit {
//     Clubs,
//     Diamonds,
//     Hearts,
//     Spades,
// }
//
// struct Card {
//     rank: Rank,
//     suit: Suit,
// }
//
// enum ActionType {
//     Dealt {
//         card1: Card,
//         card2: Card,
//     },
//     PostAnte {
//         amount: u32,
//     },
//     PostSmallBlind {
//         amount: u32,
//     },
//     PostBigBlind {
//         amount: u32,
//     },
//     Fold,
//     Check,
//     Call {
//         amount: u32,
//     },
//     Bet {
//         amount: u32,
//     },
//     Raise {
//         to_call_amount: u32,
//         raise_amount: u32,
//         is_all_in: bool,
//     },
// }
//
// struct Action {
//     player_name: String,
//     action: ActionType,
// }
//
// struct Hand {
//     game: Game,
//     table: Table,
//     actions: Vec<Action>,
//     datetime: String,
// }
