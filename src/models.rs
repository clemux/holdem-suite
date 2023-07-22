use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::summaries)]
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
