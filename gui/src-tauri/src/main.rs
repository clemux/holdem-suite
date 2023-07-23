// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use holdem_suite_parser::get_summaries;
use holdem_suite_parser::models::Summary;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn load_summaries() -> Vec<Summary> {
    let results = get_summaries();
    println!("load_summaries: {:?}", results);
    results
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, load_summaries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
