#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error(transparent)]
    X11Connection(#[from] x11rb::errors::ConnectionError),
    #[error(transparent)]
    X11Reply(#[from] x11rb::errors::ReplyError),
    #[error("Error getting window position")]
    WinPosition,
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
