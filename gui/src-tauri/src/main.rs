// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::{fs, thread};

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tauri::{App, AppHandle, CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};

use holdem_suite_db::models::{Hand, Summary};
use holdem_suite_db::{
    establish_connection, get_hands, get_summaries, insert_hands, insert_summary,
};
use holdem_suite_parser::parser::parse_hands;
use holdem_suite_parser::summary_parser;

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
fn load_hands(state: tauri::State<Settings>, app_handle: tauri::AppHandle) -> Vec<Hand> {
    app_handle
        .emit_all(
            "watcher",
            Payload {
                message: "coucou".into(),
            },
        )
        .unwrap();
    get_hands(state.database_url)
}

#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    window.get_window("main").unwrap().show().unwrap();
}

fn parse_file(path: PathBuf) {
    let connection =
        &mut establish_connection("sqlite:///home/clemux/dev/holdem-suite/parser/test.db");
    if path.clone().to_str().unwrap().contains("summary") {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let parse_result = summary_parser::TournamentSummary::parse(&data);
        let (_, summary) = parse_result.unwrap();
        insert_summary(connection, summary);
    } else {
        let data = fs::read_to_string(path).expect("Unable to read file");
        let parse_result = parse_hands(&data);
        match parse_result {
            Ok((_, hands)) => insert_hands(connection, hands),
            Err(e) => println!("{}", e),
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
        database_url: "sqlite:///home/clemux/dev/holdem-suite/parser/test.db",
        watch_folder: Path::new(
            "/home/clemux/.config/winamax/documents/accounts/WinterSound/history/",
        ),
    };
    let settings2 = settings.clone();
    tauri::Builder::default()
        .setup(move |app| setup_app(app, settings2))
        .invoke_handler(tauri::generate_handler![
            close_splashscreen,
            load_summaries,
            load_hands
        ])
        .manage(settings)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
