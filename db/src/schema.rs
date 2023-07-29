// @generated automatically by Diesel CLI.

diesel::table! {
    hands (id) {
        id -> Text,
        hole_card_1 -> Text,
        hole_card_2 -> Text,
        tournament_id -> Nullable<Integer>,
        datetime -> Text,
    }
}

diesel::table! {
    summaries (id) {
        id -> Integer,
        name -> Text,
        finish_place -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    hands,
    summaries,
);
