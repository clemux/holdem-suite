use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use notify::EventKind;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

use holdem_suite_db::{establish_connection, insert_hands, insert_summary};
use holdem_suite_parser::parser::parse_hands;
use holdem_suite_parser::summary_parser;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Parse { path: Vec<PathBuf> },

    #[command(arg_required_else_help = true)]
    Watch { path: PathBuf },
}

fn parse(path: Vec<PathBuf>) {
    let mut count = 0;
    for path in path {
        parse_file(path);
        count += 1;
    }
    println!("Parsed {} files", count);
}

fn parse_file(path: PathBuf) {
    let connection =
        &mut establish_connection("sqlite:///home/clemux/dev/holdem-suite/parser/test.db");
    println!("{}", path.display());
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

fn watch<P: AsRef<Path>>(path: P) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    let _ = watcher.watch(path.as_ref(), RecursiveMode::Recursive);
    for res in rx {
        match res {
            Ok(event) => match event.kind {
                EventKind::Create(_) => {
                    println!("created file {:?}", event.paths);
                    parse_file(event.paths[0].clone());
                }
                EventKind::Modify(_) => {
                    println!("modified file {:?}", event.paths);
                    parse_file(event.paths[0].clone());
                }
                _ => {}
            },
            Err(error) => println!("watch error: {:?}", error),
        }
    }
}

fn main() {
    let command = Cli::parse();
    match command.command {
        Commands::Parse { path } => parse(path),
        Commands::Watch { path } => watch(path),
    }
}