use diesel::prelude::*;
use serde::Serialize;

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::summaries)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Summary {
    pub id: i32,
    pub name: String,
    pub finish_place: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::summaries)]
pub struct NewSummary {
    pub id: i32,
    pub name: String,
    pub finish_place: i32,
}

#[derive(Insertable, Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::hands)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_default_value = false)]
pub struct Hand {
    pub id: String,
    pub hole_card_1: String,
    pub hole_card_2: String,
    pub tournament_id: Option<i32>,
    pub datetime: String,
}

#[derive(Queryable, Selectable, Debug, Serialize)]
#[diesel(table_name = crate::schema::actions)]
#[diesel(belongs_to(Hand))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_default_value = false)]
pub struct Action {
    pub id: i32,
    pub hand_id: String,
    pub player_name: String,
    pub action_type: String,
    pub amount: Option<f64>,
    pub is_all_in: i32,
    pub street: String,
}

#[derive(Insertable, Debug, Serialize)]
#[diesel(table_name = crate::schema::actions)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_default_value = false)]
pub struct NewAction {
    pub hand_id: String,
    pub player_name: String,
    pub action_type: String,
    pub amount: Option<f64>,
    pub is_all_in: i32,
    pub street: String,
}
