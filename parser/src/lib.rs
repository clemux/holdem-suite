use std::env;

use self::schema::summaries::dsl::*;
use crate::models::Summary;
use diesel::prelude::*;
use diesel::SqliteConnection;

pub mod models;
pub mod parser;
pub mod schema;
pub mod summary_parser;

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    println!("Connecting to {}", database_url);
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_summary(conn: &mut SqliteConnection, summary: summary_parser::TournamentSummary) {
    let new_summary = models::NewSummary {
        id: summary.id as i32,
        name: summary.name,
        finish_place: summary.finish_place as i32,
    };
    diesel::insert_into(schema::summaries::table)
        .values(&new_summary)
        .on_conflict_do_nothing()
        .execute(conn)
        .expect("Error saving new summary");
}

pub fn get_summaries(url: &str) -> Vec<Summary> {
    let mut connection = establish_connection(url);
    let results = summaries
        .select(models::Summary::as_select())
        .load(&mut connection)
        .expect("Error loading summaries");
    results
}
