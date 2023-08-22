use std::str::FromStr;
use serde::{Deserialize, Serialize};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;
use crate::errors::ApplicationError;
use crate::Table;
x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        _NET_WM_NAME,
        _NET_CLIENT_LIST,
        UTF8_STRING,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct TableWindow {
    pub window: Window,
    pub table: Table,
    pub position: WindowGeometry,
}

pub struct WindowManager {
    conn: RustConnection,
    atoms: Atoms,
    root: Window,
}

impl WindowManager {
    pub fn connect() -> Result<Self, ApplicationError> {
        let (conn, screen_num) = x11rb::connect(None)?;
        let root = conn.setup().roots[screen_num].root;
        let atoms = Atoms::new(&conn)?.reply()?;
        Ok(WindowManager { conn, atoms, root })
    }

    fn windows(&self) -> Result<Vec<u32>, ApplicationError> {
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
            )?
            .reply()?;
        for window in reply.value32().ok_or(ApplicationError::Windows)? {
            windows.push(window);
        }
        Ok(windows)
    }

    fn win_name(&self, win: Window) -> Result<String, ApplicationError> {
        let reply = self
            .conn
            .get_property(
                false,
                win,
                self.atoms._NET_WM_NAME,
                self.atoms.UTF8_STRING,
                0,
                u32::MAX,
            )?
            .reply()?;
        if reply.type_ != x11rb::NONE {
            if let Ok(value) = std::str::from_utf8(&reply.value) {
                if !value.is_empty() {
                    Ok(value.to_owned())
                } else {
                    Err(ApplicationError::NetWmNameEmpty)
                }
            } else {
                Err(ApplicationError::NetWmNameNotUtf8)
            }
        } else {
            Err(ApplicationError::NetWmNameNotUtf8)
        }
    }

    fn win_position(&self, win: Window) -> Result<WindowGeometry, ApplicationError> {
        let reply = self
            .conn
            .get_geometry(win)? // MyError::X11Connection
            .reply()?; // MyError::X11Reply
        Ok(WindowGeometry {
            x: reply.x,
            y: reply.y,
            width: reply.width,
            height: reply.height,
        })
    }

    pub fn table_windows(&self) -> Result<Vec<TableWindow>, ApplicationError> {
        let mut table_windows = vec![];
        for win in self.windows()? {
            let name = self.win_name(win)?;
            if name.starts_with("Winamax ") {
                table_windows.push(TableWindow {
                    window: win,
                    table: Table::from_str(&name).map_err(|_| ApplicationError::TableWindows)?,
                    position: self.win_position(win)?,
                });
            }
        }
        Ok(table_windows)
    }
}

