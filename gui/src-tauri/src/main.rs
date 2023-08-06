// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, thread};

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{App, AppHandle, CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};

use gui::{Table, WindowManager};
use holdem_suite_db::models::{Action, Hand, Summary};
use holdem_suite_db::{
    establish_connection, get_actions, get_hands, get_latest_hand, get_summaries, insert_hands,
    insert_summary, Player,
};
use holdem_suite_parser::parser::parse_hands;
use holdem_suite_parser::summary_parser;

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        _NET_CLIENT_LIST,
        UTF8_STRING,
    }
}

#[derive(Clone)]
struct Settings<'a> {
    database_url: &'a str,
    watch_folder: &'a Path,
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
}

#[tauri::command]
fn load_summaries(state: tauri::State<Settings>) -> Vec<Summary> {
    get_summaries(state.database_url)
}

#[tauri::command]
fn load_hands(state: tauri::State<Settings>) -> Vec<Hand> {
    get_hands(state.database_url)
}

#[tauri::command]
fn load_players(state: tauri::State<Settings>) -> Result<Vec<Player>, &'static str> {
    holdem_suite_db::get_players(state.database_url)
}

#[tauri::command]
fn load_players_for_table(
    state: tauri::State<Settings>,
    table: Table,
) -> Result<Vec<Player>, &'static str> {
    match table {
        Table::CashGame(name) => {
            holdem_suite_db::get_players_for_table(state.database_url, None, Some(name))
        }
        Table::Tournament { id, .. } => {
            holdem_suite_db::get_players_for_table(state.database_url, Some(id as i32), None)
        }
        _ => Err("Invalid table type"),
    }
}

#[tauri::command]
fn get_latest_actions(table: Table, state: tauri::State<Settings>) -> Vec<Action> {
    println!("{:?}", table);
    match table {
        Table::CashGame(name) => match get_latest_hand(state.database_url, None, Some(name)) {
            Some(hand) => return get_actions(state.database_url, hand.id),
            None => return vec![],
        },
        Table::Tournament { id, .. } => match get_latest_hand(state.database_url, Some(id), None) {
            Some(hand) => return get_actions(state.database_url, hand.id),
            None => return vec![],
        },
        Table::PendingTournament { id, .. } => {
            println!("Ignoring pending tournament{}", id);
            vec![]
        }
    }
}

#[derive(Serialize)]
struct TsTable {
    rs_table: Table,
    name: String,
}

#[tauri::command]
fn detect_tables() -> Vec<TsTable> {
    let wm = WindowManager::connect().unwrap();
    let table_windows = wm.table_windows().unwrap();
    println!("{:?}", table_windows);
    table_windows
        .iter()
        .map(|t| TsTable {
            rs_table: t.table.clone(),
            name: match &t.table {
                Table::CashGame(name) => name.to_owned(),
                Table::Tournament { name, id, .. } => format!("{} ({})", name.to_owned(), id),
                Table::PendingTournament { name, .. } => name.to_owned(),
            },
        })
        .collect()
}

#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    window.get_window("main").unwrap().show().unwrap();
}

fn parse_file(path: PathBuf) {
    let connection = &mut establish_connection("sqlite:///home/clemux/dev/holdem-suite/test.db");
    let path_cloned = path.clone();
    let path_str = path_cloned.to_str().unwrap();
    if path.clone().to_str().unwrap().contains("summary") {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let parse_result = summary_parser::TournamentSummary::parse(&data);
        match parse_result {
            Ok((_, summary)) => insert_summary(connection, summary),
            Err(_) => println!("Error parsing {}", path_str),
        }
    } else {
        println!("Parsing {}", path_str);
        let data = fs::read_to_string(path).expect("Unable to read file");
        let start = Instant::now();
        let parse_result = parse_hands(&data);
        match parse_result {
            Ok((_, hands)) => {
                let nb_hands = insert_hands(connection, hands);
                println!("Parsed {} hands in {:?}", nb_hands, start.elapsed());
            }
            Err(_) => println!("Error parsing {}", path_str),
        }
    }
}

fn watch<P: AsRef<Path>>(path: P, app_handle: &AppHandle) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    let _ = watcher.watch(path.as_ref(), RecursiveMode::Recursive);
    for res in rx {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    println!("created file: {:?}", event.paths[0]);
                    parse_file(event.paths[0].clone());
                    app_handle
                        .emit_all(
                            "watcher",
                            Payload {
                                message: "new file".into(),
                            },
                        )
                        .unwrap();
                }
                EventKind::Modify(_) => {
                    let path = event.paths[0].clone();
                    println!("modified file: {:?}", path);
                    parse_file(path.clone());
                    app_handle
                        .emit_all(
                            "watcher",
                            Payload {
                                message: format!("{}", path.clone().display()),
                            },
                        )
                        .unwrap();
                }
                _ => {}
            },
            Err(error) => println!("watch error: {:?}", error),
        }
    }
}

fn setup_app(app: &App, settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let submenu = Submenu::new("File", Menu::new().add_item(quit));
    let menu = Menu::new().add_submenu(submenu);
    let window = WindowBuilder::new(
        app,
        "main".to_string(),
        tauri::WindowUrl::App("index.html".into()),
    )
    .menu(menu)
    .visible(false)
    .title("Holdem Suite Tracker")
    .build()?;
    let window_ = window.clone();
    window_.on_menu_event(move |event| {
        if event.menu_item_id() == "quit" {
            std::process::exit(0);
        }
    });
    let app_handle = app.handle();
    let watch_folder = settings.watch_folder.to_owned();
    thread::spawn(move || watch(watch_folder, &app_handle));
    Ok(())
}

fn main() {
    let settings = Settings {
        database_url: "sqlite:///home/clemux/dev/holdem-suite/test.db",
        watch_folder: Path::new(
            "/home/clemux/.config/winamax/documents/accounts/WinterSound/history/",
        ),
    };
    let settings2 = settings.clone();
    tauri::Builder::default()
        .setup(move |app| setup_app(app, settings2))
        .invoke_handler(tauri::generate_handler![
            close_splashscreen,
            detect_tables,
            load_summaries,
            load_hands,
            get_latest_actions,
            load_players,
            load_players_for_table,
        ])
        .manage(settings)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
