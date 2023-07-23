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
