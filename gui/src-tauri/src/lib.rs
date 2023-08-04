use std::str::FromStr;

use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, tuple};
use nom::{
    bytes::complete::{tag, take_while},
    error::Error,
    IResult, Parser,
};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        _NET_WM_NAME,
        _NET_CLIENT_LIST,
        UTF8_STRING,
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum Table {
    CashGame(String),
    Tournament { name: String, id: u32, table: u32 },
    PendingTournament { name: String, id: u32 },
}

fn parse_tournament(input: &str) -> IResult<&str, Table> {
    let (input, (name, tournament_id, table_id)) = tuple((
        take_while(|c| c != '('),
        delimited(tag("("), nom::character::complete::u32, tag(")")),
        opt(delimited(
            tag("(#"),
            nom::character::complete::u32,
            tag(")"),
        )),
    ))
    .parse(input)?;
    match table_id {
        Some(table_id) => Ok((
            input,
            Table::Tournament {
                name: name.to_owned(),
                id: tournament_id,
                table: table_id,
            },
        )),
        None => Ok((
            input,
            Table::PendingTournament {
                name: name.to_owned(),
                id: tournament_id,
            },
        )),
    }
}

impl FromStr for Table {
    type Err = Error<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_cash_game = map(take_while(|c: char| c != '\n'), |name: &str| {
            Table::CashGame(name.to_string())
        });
        let mut table = preceded(tag("Winamax "), alt((parse_tournament, parse_cash_game)));
        let (_, table) = table.parse(s).unwrap();
        Ok(table)
    }
}

#[derive(Debug)]
pub struct TableWindow {
    pub window: Window,
    pub table: Table,
}

pub struct WindowManager {
    conn: RustConnection,
    atoms: Atoms,
    root: Window,
}

impl WindowManager {
    pub fn connect() -> Result<Self, &'static str> {
        let (conn, screen_num) = x11rb::connect(None).unwrap();
        let root = conn.setup().roots[screen_num].root;
        let atoms = Atoms::new(&conn).unwrap().reply().unwrap();
        Ok(WindowManager { conn, atoms, root })
    }
    fn windows(&self) -> Result<Vec<u32>, &'static str> {
        let mut windows = vec![];
        let reply = self
            .conn
            .get_property(
                false,
                self.root,
                self.atoms._NET_CLIENT_LIST,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )
            .unwrap()
            .reply()
            .unwrap();
        for window in reply.value32().unwrap() {
            windows.push(window);
        }
        Ok(windows)
    }

    fn win_name(&self, win: Window) -> Result<String, &'static str> {
        let reply = self
            .conn
            .get_property(
                false,
                win,
                self.atoms._NET_WM_NAME,
                self.atoms.UTF8_STRING,
                0,
                u32::MAX,
            )
            .unwrap()
            .reply()
            .unwrap();
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = std::str::from_utf8(&reply.value) {
                if !value.is_empty() {
                    Ok(value.to_owned())
                } else {
                    Err("win_name: _NET_WM_NAME is empty")
                }
            } else {
                Err("win_name: _NET_WM_NAME is not UTF8")
            }
        } else {
            Err("win_name: _NET_WM_NAME is NONE")
        }
    }

    pub fn table_windows(&self) -> Result<Vec<TableWindow>, &'static str> {
        let mut table_windows = vec![];
        for win in self.windows()? {
            let name = self.win_name(win)?;
            if name.starts_with("Winamax ") {
                table_windows.push(TableWindow {
                    window: win,
                    table: Table::from_str(&name).unwrap(),
                });
            }
        }
        Ok(table_windows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tournament() {
        let input = "Winamax Monster Stack(676539671)(#0001)";
        let table = Table::from_str(input).unwrap();
        assert_eq!(
            table,
            Table::Tournament {
                name: "Monster Stack".to_owned(),
                id: 676539671,
                table: 1
            }
        );
    }

    #[test]
    fn test_parse_cash_game() {
        let input = "Winamax Wichita 05";
        let table = Table::from_str(input).unwrap();
        assert_eq!(table, Table::CashGame("Wichita 05".to_owned()));
    }
}
