use holdem_suite_db::errors::DatabaseError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    // x11rb errors
    #[error(transparent)]
    X11Connect(#[from] x11rb::errors::ConnectError),
    #[error(transparent)]
    X11Connection(#[from] x11rb::errors::ConnectionError),
    #[error(transparent)]
    X11Reply(#[from] x11rb::errors::ReplyError),
    #[error("win_name: _NET_WM_NAME is empty")]
    // window manager error
    NetWmNameEmpty,
    #[error("win_name: _NET_WM_NAME is not UTF8")]
    NetWmNameNotUtf8,
    #[error("win_name: _NET_WM_NAME is NONE")]
    NetWmNameNone,
    #[error("Error getting window position")]
    WinPosition,
    #[error("Error getting windows")]
    Windows,
    #[error("Error getting table windows")]
    TableWindows,
    // popup errors
    #[error("Error opening popup")]
    OpenPopup,
    #[error("Error getting popup window")]
    GetPopupWindow,
    // tauri errors
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    // database errors
    #[error(transparent)]
    Database(#[from] DatabaseError),
    // other errors
    #[error("Error loading players for table")]
    LoadPlayersForTable,
    #[error("Error getting table max players")]
    GetTableMaxPlayers,
}

impl serde::Serialize for ApplicationError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
