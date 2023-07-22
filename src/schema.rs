// @generated automatically by Diesel CLI.

diesel::table! {
    summaries (id) {
        id -> Nullable<Integer>,
        name -> Nullable<Text>,
        finish_place -> Nullable<Integer>,
    }
}
