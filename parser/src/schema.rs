// @generated automatically by Diesel CLI.

diesel::table! {
    summaries (id) {
        id -> Integer,
        name -> Text,
        finish_place -> Integer,
    }
}
