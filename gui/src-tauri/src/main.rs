// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use holdem_suite_parser::models::Summary;
use holdem_suite_parser::{establish_connection, get_summaries};

struct Settings<'a> {
    database_url: &'a str,
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn load_summaries(state: tauri::State<Settings>) -> Vec<Summary> {
    get_summaries(state.database_url)
}

use tauri::Manager;
// Create the command:
// This command must be async so that it doesn't run on the main thread.
#[tauri::command]
async fn close_splashscreen(window: tauri::Window) {
    // Close splashscreen
    if let Some(splashscreen) = window.get_window("splashscreen") {
        splashscreen.close().unwrap();
    }
    // Show main window
    window.get_window("main").unwrap().show().unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![close_splashscreen, load_summaries])
        .manage(Settings {
            database_url: "sqlite:///home/clemux/dev/holdem-suite/parser/test.db",
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
