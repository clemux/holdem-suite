// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{App, AppHandle, CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};
use uuid::Uuid;

use gui::errors::ApplicationError;
use gui::window_management::{TableWindow, WindowGeometry, WindowManager};
use gui::{compute_hand_metrics, parse_file, Table};
use holdem_suite_db::models::{Action, Hand, Seat, Summary};
use holdem_suite_db::{
    establish_connection, get_actions, get_actions_for_hand, get_hands, get_hands_for_player,
    get_latest_hand, get_players, get_players_for_table, get_seats, get_summaries, Player,
    TablePlayer,
};

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
fn open_popup(player: Player, handle: AppHandle) -> Result<(), ApplicationError> {
    let window = WindowBuilder::new(
        &handle,
        "popup",
        tauri::WindowUrl::App("hud-popup.html".into()),
    )
    .decorations(false)
    .inner_size(200.0, 50.0)
    .resizable(false)
    .build()?;
    let label = window.label().to_owned();
    window.once("popupReady", move |_msg| {
        let window = handle.get_window(&label).unwrap(); // TODO: how to handle errors here?
        window.emit("popup", player).unwrap();
    });

    Ok(())
}

#[tauri::command]
fn open_hud_command(
    player: Player,
    position: WindowGeometry,
    handle: AppHandle,
) -> Result<(), &'static str> {
    let window_label = Uuid::new_v4().to_string();
    let hud_x = position.x as f64 + position.width as f64 / 2.0;
    let hud_y = position.y as f64 + position.height as f64 / 2.0;
    let window = WindowBuilder::new(
        &handle,
        window_label.to_owned(),
        tauri::WindowUrl::App("hud.html".into()),
    )
    .decorations(false)
    .inner_size(160.0, 50.0)
    .position(hud_x, hud_y)
    .resizable(true)
    .always_on_top(true)
    .build()
    .map_err(|e| {
        println!("Error creating popup {:?}", e);
        "Error creating popup"
    })?;
    let label = window_label.to_owned();
    window.once("hudReady", move |_| {
        let window = handle.get_window(&label).unwrap();
        window.emit("hud", player).unwrap();
    });

    Ok(())
}

#[tauri::command]
fn load_summaries(state: tauri::State<Settings>) -> Result<Vec<Summary>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    Ok(get_summaries(&mut conn)?)
}

#[tauri::command]
fn load_hands(state: tauri::State<Settings>) -> Result<Vec<Hand>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    Ok(get_hands(&mut conn)?)
}

#[tauri::command]
fn load_seats(hand_id: &str, state: tauri::State<Settings>) -> Result<Vec<Seat>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    let seats = get_seats(&mut conn, hand_id)?;
    Ok(seats)
}

#[tauri::command]
fn load_actions(
    hand_id: &str,
    state: tauri::State<Settings>,
) -> Result<Vec<Action>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    Ok(get_actions_for_hand(&mut conn, hand_id)?)
}

#[tauri::command]
fn load_players(state: tauri::State<Settings>) -> Result<Vec<Player>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    Ok(get_players(&mut conn)?)
}

#[derive(Serialize)]
struct PlayerStats {
    vpip: f32,
    pfr: f32,
    three_bet: f32,
    open_limp: f32,
    nb_hands: u32,
}

#[tauri::command]
fn load_player_stats(
    player_name: String,
    state: tauri::State<Settings>,
) -> Result<PlayerStats, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    let mut stats = PlayerStats {
        vpip: 0.0,
        pfr: 0.0,
        three_bet: 0.0,
        open_limp: 0.0,
        nb_hands: 0,
    };
    let hands_actions = get_hands_for_player(&mut conn, player_name.as_str())?;
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
        nb_hands: nb_hands as u32,
    })
}

#[tauri::command]
fn load_players_for_table(
    state: tauri::State<Settings>,
    table: Table,
) -> Result<Vec<TablePlayer>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    match table {
        Table::CashGame(name) => Ok(get_players_for_table(&mut conn, None, Some(name.clone()))?),
        Table::Tournament { id, .. } => {
            Ok(get_players_for_table(&mut conn, Some(id as i32), None)?)
        }
        _ => Err(ApplicationError::LoadPlayersForTable),
    }
}

#[tauri::command]
fn get_latest_actions(
    table: Table,
    state: tauri::State<Settings>,
) -> Result<Vec<Action>, ApplicationError> {
    let mut conn = establish_connection(state.database_url);
    match table {
        Table::CashGame(name) => match get_latest_hand(&mut conn, None, Some(name))? {
            Some(hand) => Ok(get_actions(&mut conn, hand.id)?),
            None => Ok(vec![]),
        },
        Table::Tournament { id, .. } => match get_latest_hand(&mut conn, Some(id), None).unwrap() {
            Some(hand) => Ok(get_actions(&mut conn, hand.id)?),
            None => Ok(vec![]),
        },
        Table::PendingTournament { .. } => Ok(vec![]),
    }
}

#[derive(Serialize)]
struct TsTable {
    rs_table: Table,
    window_position: WindowGeometry,
    name: String,
}

#[tauri::command]
fn detect_tables() -> Result<Vec<TsTable>, ApplicationError> {
    let wm = WindowManager::connect()?;
    let table_windows = wm.table_windows()?;
    Ok(table_windows
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
        .collect())
}

#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    window.get_window("main").unwrap().show().unwrap();
}

fn watch<P: AsRef<Path>>(
    path: P,
    database_url: String,
    app_handle: AppHandle,
    event_tx: mpsc::Sender<()>,
) {
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    let _ = watcher.watch(path.as_ref(), RecursiveMode::Recursive);
    let mut connection = establish_connection(&database_url);
    for res in rx {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    println!("created file: {:?}", event.paths[0]);
                    let _ = parse_file(event.paths[0].clone(), &mut connection).unwrap();
                    app_handle
                        .emit_all(
                            "watcher",
                            Payload {
                                message: "new file".into(),
                            },
                        )
                        .unwrap();
                    event_tx.send(()).unwrap();
                }
                EventKind::Modify(_) => {
                    let path = event.paths[0].clone();
                    println!("modified file: {:?}", path);
                    let _ = parse_file(path.clone(), &mut connection).unwrap();
                    app_handle
                        .emit_all(
                            "watcher",
                            Payload {
                                message: format!("{}", path.clone().display()),
                            },
                        )
                        .unwrap();
                    event_tx.send(()).unwrap();
                }
                _ => {}
            },
            Err(error) => println!("watch error: {:?}", error),
        }
    }
}

fn get_table_players(
    table: Table,
    database_url: &str,
) -> Result<Vec<TablePlayer>, ApplicationError> {
    let mut conn = establish_connection(database_url);
    let players = match table {
        Table::CashGame(name) => get_players_for_table(&mut conn, None, Some(name.clone())),
        Table::Tournament { id, .. } => get_players_for_table(&mut conn, Some(id as i32), None),
        _ => Ok(vec![]),
    };
    match players {
        Ok(players) => Ok(players),
        Err(_) => {
            println!("Error getting players");
            Ok(vec![])
        }
    }
}

fn compute_hud_position(
    table: TableWindow,
    max_players: i32,
    hero_seat: i32,
    player: TablePlayer,
) -> (f64, f64) {
    let seat_index = (player.seat_number + (max_players - hero_seat)) % max_players;
    let (x_multiplier, y_multiplier) = match max_players {
        3 => match seat_index {
            0 => (0.57, 0.7),
            1 => (0.1, 0.1),
            2 => (0.8, 0.1),
            _ => (0.5, 0.5),
        },
        5 => match seat_index {
            0 => (0.57, 0.7),
            1 => (0.07, 0.45),
            2 => (0.35, 0.1),
            3 => (0.8, 0.1),
            4 => (0.8, 0.45),
            _ => (0.5, 0.5),
        },
        6 => match seat_index {
            0 => (0.57, 0.7),
            1 => (0.07, 0.52),
            2 => (0.07, 0.1),
            3 => (0.57, 0.1),
            4 => (0.8, 0.33),
            5 => (0.8, 0.52),
            _ => (0.5, 0.5),
        },
        7 => match seat_index {
            0 => (0.57, 0.7),
            1 => (0.02, 0.7),
            2 => (0.07, 0.3),
            3 => (0.3, 0.1),
            4 => (0.57, 0.1),
            5 => (0.85, 0.1),
            6 => (0.85, 0.3),
            _ => (0.5, 0.5),
        },
        8 => match seat_index {
            0 => (0.57, 0.7),
            1 => (0.02, 0.7),
            2 => (0.07, 0.3),
            3 => (0.3, 0.1),
            4 => (0.57, 0.1),
            5 => (0.85, 0.1),
            6 => (0.85, 0.3),
            7 => (0.85, 0.7),
            _ => (0.5, 0.5),
        },
        _ => (0.5, 0.5),
    };
    (
        table.position.x as f64 + table.position.width as f64 * x_multiplier,
        table.position.y as f64 + table.position.height as f64 * y_multiplier,
    )
}

#[derive(Debug)]
struct HudWindow {
    label: String,
    table: Table,
    player: TablePlayer,
    max_players: i32,
    hero_seat: i32,
}

impl HudWindow {
    fn new(
        table: TableWindow,
        player: TablePlayer,
        max_players: i32,
        hero_seat: i32,
        app_handle: AppHandle,
    ) -> Result<HudWindow, ApplicationError> {
        let window_label = Uuid::new_v4().to_string();

        let (hud_x, hud_y) =
            compute_hud_position(table.to_owned(), max_players, hero_seat, player.to_owned());

        let window = WindowBuilder::new(
            &app_handle,
            window_label.to_owned(),
            tauri::WindowUrl::App("hud.html".into()),
        )
        .decorations(false)
        .inner_size(120.0, 50.0)
        .position(hud_x, hud_y)
        .resizable(true)
        .always_on_top(true)
        .build()?;

        let label = window_label.to_owned();
        let player_copy = player.to_owned();
        window.once("hudReady", move |_msg| {
            let window = app_handle.get_window(&label).expect("Error getting window");
            window
                .emit("hud", player_copy)
                .expect("Error emitting hud event");
        });
        Ok(HudWindow {
            label: window_label,
            table: table.table,
            player,
            max_players,
            hero_seat,
        })
    }
}

struct TableHuds {
    table_window: TableWindow,
    huds: Vec<HudWindow>,
    players: HashSet<TablePlayer>,
}

impl TableHuds {
    fn new(
        table_window: TableWindow,
        app_handle: &AppHandle,
        database_url: &str,
    ) -> Result<TableHuds, ApplicationError> {
        let mut conn = establish_connection(database_url);
        let players: HashSet<TablePlayer> = HashSet::from_iter(get_table_players(
            table_window.table.to_owned(),
            database_url,
        )?);
        let max_players_and_hero =
            gui::get_table_max_players_and_hero(&mut conn, table_window.table.to_owned())?;
        match max_players_and_hero {
            Some((max_players, hero)) => {
                let hero_seat = players
                    .iter()
                    .filter(|p| p.name == hero)
                    .map(|p| p.seat_number)
                    .next()
                    .unwrap();
                let huds: Vec<HudWindow> = players
                    .iter()
                    .map(|p| {
                        HudWindow::new(
                            table_window.to_owned(),
                            p.to_owned(),
                            max_players,
                            hero_seat,
                            app_handle.to_owned(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(TableHuds {
                    table_window,
                    players,
                    huds,
                })
            }
            None => Ok(TableHuds {
                table_window,
                players,
                huds: vec![],
            }),
        }
    }

    fn update(
        &mut self,
        app_handle: &AppHandle,
        database_url: &str,
    ) -> Result<(), ApplicationError> {
        let players = HashSet::from_iter(
            get_table_players(self.table_window.table.to_owned(), database_url).unwrap(),
        );
        let mut conn = establish_connection(database_url);
        let max_players =
            gui::get_table_max_players_and_hero(&mut conn, self.table_window.table.to_owned())?;
        match max_players {
            Some((max_players, hero)) => {
                let hero_seat = players
                    .iter()
                    .filter(|p| p.name == hero)
                    .map(|p| p.seat_number)
                    .next()
                    .unwrap();
                let new_players = players.difference(&self.players);
                let players_left = self.players.difference(&players);
                new_players.into_iter().for_each(|p| {
                    self.huds.push(
                        HudWindow::new(
                            self.table_window.to_owned(),
                            p.to_owned(),
                            max_players,
                            hero_seat,
                            app_handle.to_owned(),
                        )
                        .unwrap(),
                    )
                });
                players_left.into_iter().for_each(|p| {
                    for hud in self.huds.iter() {
                        if hud.player == *p {
                            let window = app_handle.get_window(hud.label.as_str());
                            match window {
                                Some(window) => window.close().unwrap(),
                                None => {
                                    println!("Window {} not found", hud.label);
                                }
                            }
                        }
                    }
                });
                self.players = players;
                Ok(())
            }
            None => Err::<(), ApplicationError>(ApplicationError::GetTableMaxPlayers),
        }
    }

    fn close(&self, app_handle: &AppHandle) -> Result<(), ApplicationError> {
        for hud in self.huds.iter() {
            let window = app_handle.get_window(hud.label.as_str());
            match window {
                Some(window) => window.close()?,
                None => {
                    println!("Window {} not found", hud.label);
                }
            }
        }
        Ok(())
    }
}

fn watch_windows(app_handle: AppHandle, database_url: String, event_rx: mpsc::Receiver<()>) {
    let mut tables_windows: HashMap<Table, TableWindow> = HashMap::new();
    let mut tables_huds: HashMap<Table, TableHuds> = HashMap::new();
    loop {
        let mut received_event = false;
        for _event in event_rx.try_iter() {
            if !received_event {
                received_event = true;
                for hud in tables_huds.values_mut() {
                    match hud.update(&app_handle, &database_url) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Error updating huds");
                        }
                    };
                }
            }
        }
        let wm = match WindowManager::connect() {
            Ok(wm) => wm,
            Err(_) => {
                println!("Error connecting to window manager");
                continue;
            }
        };
        let detected_table_windows: HashMap<Table, TableWindow> = match wm.table_windows() {
            Ok(tables) => tables,
            Err(_) => {
                println!("Error getting table windows");
                continue;
            }
        }
        .into_iter()
        .filter(|t| matches!(t.table, Table::CashGame(_) | Table::Tournament { .. }))
        .map(|t| (t.table.to_owned(), t))
        .collect();

        let current_tables: HashSet<Table> = HashSet::from_iter(tables_windows.keys().cloned());

        let detected_tables: HashSet<Table> = HashSet::from_iter(
            detected_table_windows
                .iter()
                .map(|(t, _)| t.to_owned())
                .collect::<Vec<_>>(),
        );
        let new_tables = detected_tables.difference(&current_tables);
        let tables_closed = current_tables.difference(&detected_tables);

        // closed tables
        for table in tables_closed {
            let huds = tables_huds.remove(table);
            if let Some(huds) = huds {
                match huds.close(&app_handle) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Error closing huds");
                    }
                }
            }
        }

        // new windows
        for table in new_tables {
            let new_table_window = detected_table_windows.get(table).unwrap();
            match TableHuds::new(new_table_window.clone(), &app_handle, &database_url) {
                Ok(table_huds) => {
                    tables_huds.insert(table.to_owned(), table_huds);
                }
                Err(_) => {
                    println!("Error creating table huds");
                    continue;
                }
            }
        }
        tables_windows.clear();
        tables_windows = detected_table_windows;
        thread::sleep(std::time::Duration::from_secs(1));
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
    let (tx, rx) = mpsc::channel();
    let database_url = settings.database_url.to_owned();
    thread::spawn(move || watch(watch_folder, database_url, app_handle, tx));
    let app_handle = app.handle();
    let database_url = settings.database_url.to_owned();
    thread::spawn(move || watch_windows(app_handle, database_url, rx));
    Ok(())
}

fn main() {
    let settings = Settings {
        database_url: r#"C:\Users\cleme\PycharmProjects\holdem-suite\db\test.db"#,
        watch_folder: Path::new(
            r#"C:\Users\cleme\AppData\Roaming\winamax\documents\accounts\WinterSound\history\"#,
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
            open_hud_command,
        ])
        .manage(settings)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
