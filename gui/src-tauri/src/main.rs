// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{fs, thread};

use anyhow::Result;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{App, AppHandle, CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};

use gui::{compute_hand_metrics, Table, WindowGeometry, WindowManager};
use holdem_suite_db::models::{Action, Hand, Seat, Summary};
use holdem_suite_db::{
    establish_connection, get_actions, get_actions_for_hand, get_hands, get_hands_for_player,
    get_latest_hand, get_players, get_players_for_table, get_seats, get_summaries, insert_hands,
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
fn open_popup(player: Player, handle: AppHandle) -> Result<(), &'static str> {
    let window = WindowBuilder::new(
        &handle,
        "popup",
        tauri::WindowUrl::App("hud-popup.html".into()),
    )
    .decorations(false)
    .inner_size(200.0, 50.0)
    .resizable(false)
    .build()
    .map_err(|_| "Error creating popup")?;
    let label = window.label().to_owned();
    window.once("popupReady", move |msg| {
        let window = handle.get_window(&label).unwrap();
        window.emit("popup", player).unwrap();
        println!("Received {:?}", msg);
    });

    Ok(())
}

#[tauri::command]
fn open_hud(
    player: Player,
    table: Table,
    position: WindowGeometry,
    handle: AppHandle,
) -> Result<(), &'static str> {
    println!("Opening HUD for {:?}", player);
    let table_name = match table {
        Table::CashGame(name) => name,
        Table::Tournament { name, .. } => name,
        Table::PendingTournament { name, .. } => name,
    };
    let window_label = format!("hud_{}_{}", table_name, player.name).replace(' ', "_");
    let hud_x = position.x as f64 + position.width as f64 / 2.0;
    let hud_y = position.y as f64 + position.height as f64 / 2.0;
    println!("Window label: {}", window_label);
    let window = WindowBuilder::new(
        &handle,
        window_label.to_owned(),
        tauri::WindowUrl::App("hud.html".into()),
    )
    .decorations(false)
    .inner_size(200.0, 70.0)
    .position(hud_x, hud_y)
    .resizable(true)
    .always_on_top(true)
    .build()
    .map_err(|e| {
        println!("Error creating popup {:?}", e);
        "Error creating popup"
    })?;
    println!("Opened window {:?}", window.label());
    let label = window_label.to_owned();
    window.once("hudReady", move |msg| {
        let window = handle.get_window(&label).unwrap();
        window.emit("hud", player).unwrap();
        println!("Received {:?}", msg);
    });

    Ok(())
}

#[tauri::command]
fn load_summaries(state: tauri::State<Settings>) -> Vec<Summary> {
    let mut conn = establish_connection(state.database_url);
    get_summaries(&mut conn)
}

#[tauri::command]
fn load_hands(state: tauri::State<Settings>) -> Vec<Hand> {
    let mut conn = establish_connection(state.database_url);
    get_hands(&mut conn)
}

#[tauri::command]
fn load_seats(hand_id: &str, state: tauri::State<Settings>) -> Result<Vec<Seat>, &'static str> {
    let mut conn = establish_connection(state.database_url);
    let seats = get_seats(&mut conn, hand_id).map_err(|_| "Error loading seats")?;
    Ok(seats)
}

#[tauri::command]
fn load_actions(hand_id: &str, state: tauri::State<Settings>) -> Result<Vec<Action>, &'static str> {
    let mut conn = establish_connection(state.database_url);
    let actions = get_actions_for_hand(&mut conn, hand_id).map_err(|_| "Error loading actions")?;
    Ok(actions)
}

#[tauri::command]
fn load_players(state: tauri::State<Settings>) -> Result<Vec<Player>, &'static str> {
    let mut conn = establish_connection(state.database_url);
    get_players(&mut conn).map_err(|e| "Error while loading players")
}

#[derive(Serialize)]
struct PlayerStats {
    vpip: f32,
    pfr: f32,
    three_bet: f32,
    open_limp: f32,
}

#[tauri::command]
fn load_player_stats(
    player_name: String,
    state: tauri::State<Settings>,
) -> Result<PlayerStats, &'static str> {
    let mut conn = establish_connection(state.database_url);
    let mut stats = PlayerStats {
        vpip: 0.0,
        pfr: 0.0,
        three_bet: 0.0,
        open_limp: 0.0,
    };
    let hands_actions = get_hands_for_player(&mut conn, player_name.as_str())
        .map_err(|e| "Error while loading hands for player")?;
    let nb_hands = hands_actions.len();
    for (_, actions) in hands_actions {
        let metrics = compute_hand_metrics(actions);
        let player_metrics = metrics.get(&player_name);
        match player_metrics {
            Some(metrics) => {
                stats.vpip += metrics.vpip as u32 as f32;
                stats.pfr += metrics.pfr as u32 as f32;
                stats.three_bet += metrics.three_bet as u32 as f32;
                stats.open_limp += metrics.open_limp as u32 as f32;
            }
            None => continue,
        }
    }

    Ok(PlayerStats {
        vpip: stats.vpip / nb_hands as f32,
        pfr: stats.pfr / nb_hands as f32,
        three_bet: stats.three_bet / nb_hands as f32,
        open_limp: stats.open_limp / nb_hands as f32,
    })
}

#[tauri::command]
fn load_players_for_table(
    state: tauri::State<Settings>,
    table: Table,
) -> Result<Vec<Player>, &'static str> {
    let mut conn = establish_connection(state.database_url);
    match table {
        Table::CashGame(name) => get_players_for_table(&mut conn, None, Some(name.clone()))
            .map_err(|e| {
                eprintln!("Error while loading players for table {}: {}", name, e);
                "Error while loading players for table"
            }),
        Table::Tournament { id, .. } => get_players_for_table(&mut conn, Some(id as i32), None)
            .map_err(|e| {
                eprintln!("Error while loading players for tournament {}: {}", id, e);
                "Error while loading players for tournament"
            }),
        _ => Err("Invalid table type"),
    }
}

#[tauri::command]
fn get_latest_actions(table: Table, state: tauri::State<Settings>) -> Vec<Action> {
    let mut conn = establish_connection(state.database_url);
    match table {
        Table::CashGame(name) => match get_latest_hand(&mut conn, None, Some(name)) {
            Some(hand) => return get_actions(&mut conn, hand.id),
            None => return vec![],
        },
        Table::Tournament { id, .. } => match get_latest_hand(&mut conn, Some(id), None) {
            Some(hand) => return get_actions(&mut conn, hand.id),
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
    window_position: WindowGeometry,
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
            window_position: t.position.clone(),
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
                let nb_hands = insert_hands(connection, hands).unwrap();
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
            load_player_stats,
            load_seats,
            load_actions,
            open_popup,
            open_hud,
        ])
        .manage(settings)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
