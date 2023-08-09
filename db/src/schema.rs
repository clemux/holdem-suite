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
    }
}

diesel::table! {
    seats (hand_id, seat_number) {
        hand_id -> Text,
        player_name -> Text,
        seat_number -> Integer,
    }
}

diesel::table! {
    summaries (id) {
        id -> Integer,
        name -> Text,
        finish_place -> Integer,
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
