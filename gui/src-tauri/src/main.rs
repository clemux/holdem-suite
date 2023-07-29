// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use holdem_suite_db::{get_hands, get_summaries};
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowBuilder};

use holdem_suite_db::models::{Hand, Summary};

struct Settings<'a> {
    database_url: &'a str,
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
async fn close_splashscreen(window: tauri::Window) {
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    window.get_window("main").unwrap().show().unwrap();
}

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let submenu = Submenu::new("File", Menu::new().add_item(quit));
    let menu = Menu::new().add_submenu(submenu);

    tauri::Builder::default()
        .setup(|app| {
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            close_splashscreen,
            load_summaries,
            load_hands
        ])
        .manage(Settings {
            database_url: "sqlite:///home/clemux/dev/holdem-suite/parser/test.db",
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
