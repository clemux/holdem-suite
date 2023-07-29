use std::fs;

use clap::Parser;

use holdem_suite_parser::parser::parse_hands;
use holdem_suite_parser::{
    establish_connection, insert_hands, insert_summary, parser, summary_parser,
};

#[derive(Parser)]
struct Cli {
    path: Vec<std::path::PathBuf>,
}

fn main() {
    let args = Cli::parse();
    let connection =
        &mut establish_connection("sqlite:///home/clemux/dev/holdem-suite/parser/test.db");
    for path in args.path {
        // println!("{}", path.display());
        if path.clone().to_str().unwrap().contains("summary") {
            let data = fs::read_to_string(path).expect("Unable to read file");
            let parse_result = summary_parser::TournamentSummary::parse(&data);
            // println!("{}", parse_result.is_ok());
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
}
