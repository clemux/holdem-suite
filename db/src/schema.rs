// @generated automatically by Diesel CLI.

diesel::table! {
    actions (id) {
        id -> Integer,
        hand_id -> Text,
        player_name -> Text,
        action_type -> Text,
        amount -> Nullable<Double>,
        is_all_in -> Integer,
        street -> Text,
    }
}

diesel::table! {
    hands (id) {
        id -> Text,
        hole_card_1 -> Text,
        hole_card_2 -> Text,
        tournament_id -> Nullable<Integer>,
        cash_game_name -> Nullable<Text>,
        datetime -> Text,
        button -> Integer,
        max_players -> Integer,
        hero -> Text,
        ante -> Nullable<Double>,
        small_blind -> Double,
        big_blind -> Double,
        pot -> Double,
        rake -> Nullable<Double>,
        flop1 -> Nullable<Text>,
        flop2 -> Nullable<Text>,
        flop3 -> Nullable<Text>,
        turn -> Nullable<Text>,
        river -> Nullable<Text>,
    }
}

diesel::table! {
    seats (hand_id, seat_number) {
        hand_id -> Text,
        player_name -> Text,
        seat_number -> Integer,
        stack -> Double,
        bounty -> Nullable<Double>,
        card1 -> Nullable<Text>,
        card2 -> Nullable<Text>,
    }
}

diesel::table! {
    summaries (id) {
        id -> Integer,
        name -> Text,
        buyin -> Double,
        date -> Timestamp,
        play_time -> Text,
        entries -> Integer,
        mode -> Text,
        tournament_type -> Text,
        speed -> Text,
        finish_place -> Integer,
        won -> Nullable<Double>,
    }
}

diesel::joinable!(actions -> hands (hand_id));
diesel::joinable!(seats -> hands (hand_id));

diesel::allow_tables_to_appear_in_same_query!(
    actions,
    hands,
    seats,
    summaries,
);
