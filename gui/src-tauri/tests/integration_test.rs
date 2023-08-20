#[cfg(test)]
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use gui::get_table_max_players_and_hero;
use gui::parse_file;
use gui::Table;
use holdem_suite_db::models::Hand;
use holdem_suite_db::schema::hands;
use holdem_suite_db::{establish_connection, get_hands};
use std::path::PathBuf;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../db/migrations/");

fn run_migrations(conn: &mut impl MigrationHarness<Sqlite>) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

fn establish_test_connection() -> SqliteConnection {
    let mut conn = establish_connection(":memory:");
    run_migrations(&mut conn);
    conn
}

#[test]
fn test_insert_hand() {
    let mut conn = establish_test_connection();
    let hand = Hand {
        id: "whatever".to_owned(),
        hole_card_1: "Ah".to_owned(),
        hole_card_2: "Ks".to_owned(),
        tournament_id: None,
        cash_game_name: None,
        datetime: "2021-01-01 00:00:00".to_owned(),
        button: 1,
        max_players: 9,
        hero: "WinterSound".to_owned(),
    };
    let inserted = diesel::insert_or_ignore_into(hands::table)
        .values(&hand)
        .execute(&mut conn);
    assert_eq!(Ok(1), inserted);
}

#[test]
fn test_parse_file() {
    let mut conn = establish_test_connection();
    let nb_parsed_hands =
        parse_file(PathBuf::from("tests/samples/sample1.txt"), &mut conn).unwrap();
    assert_eq!(3, nb_parsed_hands);

    let hands = get_hands(&mut conn).unwrap();
    assert_eq!(3, hands.len());
    assert_eq!("6s", hands[0].hole_card_1);
}

#[test]
fn test_get_hands_for_player() {
    let mut conn = establish_test_connection();
    parse_file(PathBuf::from("tests/samples/sample1.txt"), &mut conn);
    let hands = get_hands(&mut conn).unwrap();
    assert_eq!(3, hands.len());
    let hands = holdem_suite_db::get_hands_for_player(&mut conn, "WinterSound").unwrap();
    assert_eq!(2, hands.len());
}

#[test]
fn test_get_players() {
    let mut conn = establish_test_connection();
    parse_file(PathBuf::from("tests/samples/sample1.txt"), &mut conn);
    let players = holdem_suite_db::get_players(&mut conn).unwrap();
    assert_eq!(7, players.len());
}

#[test]
fn test_get_max_players_and_hero() {
    let mut conn = establish_test_connection();
    parse_file(PathBuf::from("tests/samples/sample1.txt"), &mut conn);
    let table = Table::Tournament {
        name: String::from("WESTERN"),
        id: 655531954,
        table: 77,
    };
    let (max_players, hero) = get_table_max_players_and_hero(&mut conn, table)
        .unwrap()
        .unwrap();
    assert_eq!(6, max_players);
    assert_eq!("NotWinterSound", hero);
}
