#[derive(Debug, thiserror::Error)]
pub enum MyError {
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
}

impl serde::Serialize for MyError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
