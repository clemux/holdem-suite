// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{mpsc, Mutex};
use std::thread;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{App, AppHandle, CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};
use uuid::Uuid;
use x11rb::protocol::xproto::Window;

use gui::errors::ApplicationError;
use gui::{compute_hand_metrics, parse_file, Table, TableWindow, WindowGeometry, WindowManager};
use holdem_suite_db::models::{Action, Hand, Seat, Summary};
use holdem_suite_db::{
    establish_connection, get_actions, get_actions_for_hand, get_hands, get_hands_for_player,
    get_latest_hand, get_players, get_players_for_table, get_seats, get_summaries, Player,
};
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

#[derive(Debug)]
struct HudWindow {
    label: String,
    table: Table,
    player: Player,
}

struct State {
    hud_windows: Mutex<Vec<HudWindow>>,
    tables: Mutex<Vec<TableWindow>>,
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
    window.once("popupReady", move |msg| {
        let window = handle.get_window(&label).unwrap(); // TODO: how to handle errors here?
        window.emit("popup", player).unwrap();
        println!("Received {:?}", msg);
    });

    Ok(())
}

#[tauri::command]
fn open_hud_command(
    player: Player,
    table: Table,
    position: WindowGeometry,
    handle: AppHandle,
) -> Result<(), &'static str> {
    println!("Opening HUD for {:?}", player);
    let window_label = Uuid::new_v4().to_string();
    let hud_x = position.x as f64 + position.width as f64 / 2.0;
    let hud_y = position.y as f64 + position.height as f64 / 2.0;
    println!("Window label: {}", window_label);
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
) -> Result<Vec<Player>, ApplicationError> {
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
        Table::PendingTournament { id, .. } => {
            println!("Ignoring pending tournament{}", id);
            Ok(vec![])
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

fn watch<P: AsRef<Path>>(path: P, app_handle: AppHandle, event_tx: mpsc::Sender<()>) {
    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    let _ = watcher.watch(path.as_ref(), RecursiveMode::Recursive);
    let database_url = "sqlite:///home/clemux/dev/holdem-suite/test.db";
    let mut connection = establish_connection(database_url);
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

fn get_table_players(table: Table) -> Result<Vec<Player>, ApplicationError> {
    let mut conn = establish_connection("sqlite:///home/clemux/dev/holdem-suite/test.db");
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

fn create_player_hud(
    table: TableWindow,
    player: Player,
    app_handle: AppHandle,
) -> Result<HudWindow, ApplicationError> {
    println!("Creating player hud for {:?}", player);
    let window_label = Uuid::new_v4().to_string();

    let hud_x = table.position.x as f64 + table.position.width as f64 / 2.0;
    let hud_y = table.position.y as f64 + table.position.height as f64 / 2.0;

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

    println!("Opened window {:?}", window.label());
    let label = window_label.to_owned();
    let player_copy = player.to_owned();
    window.once("hudReady", move |msg| {
        let window = app_handle.get_window(&label).expect("Error getting window");
        window
            .emit("hud", player_copy)
            .expect("Error emitting hud event");
        println!("Received {:?}", msg);
    });
    Ok(HudWindow {
        label: window_label,
        table: table.table,
        player: player,
    })
}

struct TableHuds {
    table_window: TableWindow,
    huds: Vec<HudWindow>,
    players: HashSet<Player>,
}

impl TableHuds {
    fn new(
        table_window: TableWindow,
        app_handle: AppHandle,
    ) -> Result<TableHuds, ApplicationError> {
        let players: HashSet<Player> =
            HashSet::from_iter(get_table_players(table_window.table.to_owned())?);

        println!("New HUDs for {:?}", table_window);
        println!("Players: {:?}", players);
        let huds: Vec<HudWindow> = players
            .iter()
            .map(|p| {
                create_player_hud(table_window.to_owned(), p.to_owned(), app_handle.to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TableHuds {
            table_window,
            players,
            huds,
        })
    }

    fn update(&mut self, app_handle: AppHandle) -> Result<(), ApplicationError> {
        let players =
            HashSet::from_iter(get_table_players(self.table_window.table.to_owned()).unwrap());
        println!("Update - Players: {:?}", players);
        let new_players = players.difference(&self.players);
        let players_left = self.players.difference(&players);
        println!("New players: {:?}", new_players);
        println!("Players left: {:?}", players_left);
        new_players.into_iter().for_each(|p| {
            self.huds.push(
                create_player_hud(
                    self.table_window.to_owned(),
                    p.to_owned(),
                    app_handle.to_owned(),
                )
                .unwrap(),
            )
        });
        players_left.into_iter().for_each(|p| {
            for hud in self.huds.iter() {
                if hud.player == *p {
                    println!("Closing HUD for {:?}", p);
                    println!("Window label: {}", hud.label);
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

    fn close(self) {}
}

fn watch_windows(app_handle: AppHandle, event_rx: mpsc::Receiver<()>) {
    let mut tables_windows: HashMap<Window, TableWindow> = HashMap::new();
    let mut tables_huds: HashMap<Window, TableHuds> = HashMap::new();
    loop {
        let mut received_event = false;
        for event in event_rx.try_iter() {
            if !received_event {
                received_event = true;
                for hud in tables_huds.values_mut() {
                    match hud.update(app_handle.to_owned()) {
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
        let new_table_windows: Vec<TableWindow> = match wm.table_windows() {
            Ok(tables) => tables,
            Err(_) => {
                println!("Error getting table windows");
                continue;
            }
        }
        .into_iter()
        .filter(|t| matches!(t.table, Table::CashGame(_) | Table::Tournament { .. }))
        .collect();

        // closed windows
        let new_table_window_ids: Vec<Window> =
            new_table_windows.iter().map(|t| t.window).collect();
        for table_window in tables_windows.keys() {
            if !new_table_window_ids.contains(table_window) {
                //     close_table_hud_windows(
                //         tables_windows.get(table_window).unwrap().clone(),
                //         app_handle.to_owned(),
                //     );
            }
        }

        // new windows
        for new_table_window in new_table_windows.iter() {
            if !tables_windows.contains_key(&new_table_window.window) {
                println!("New table window {:?}", new_table_window);
                match TableHuds::new(new_table_window.clone(), app_handle.to_owned()) {
                    Ok(table_huds) => {
                        tables_huds.insert(new_table_window.window, table_huds);
                    }
                    Err(_) => {
                        println!("Error creating table huds");
                        continue;
                    }
                }
            }
        }

        // update
        tables_windows.clear();
        tables_windows = new_table_windows
            .into_iter()
            .map(|t| (t.window, t))
            .collect();
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
    thread::spawn(move || watch(watch_folder, app_handle, tx));
    let app_handle = app.handle();
    thread::spawn(move || watch_windows(app_handle, rx));
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
            open_hud_command,
        ])
        .manage(settings)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
