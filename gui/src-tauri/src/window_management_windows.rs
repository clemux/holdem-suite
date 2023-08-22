use crate::errors::ApplicationError;
use crate::Table;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use windows::{Win32::Foundation::*, Win32::UI::WindowsAndMessaging::*};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowGeometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone)]
pub struct TableWindow {
    pub table: Table,
    pub position: WindowGeometry,
}



pub struct WindowManager {}

impl WindowManager {
    pub fn connect() -> Result<Self, ApplicationError> {
        Ok(WindowManager {})
    }

    pub fn table_windows(self) -> Result<Vec<TableWindow>, ApplicationError> {
        let mut table_windows = vec![];
        for window in get_windows()? {
            if window.text.starts_with("Winamax ") {
                table_windows.push ({
                    TableWindow {
                        table: Table::from_str(&window.text).unwrap(),
                        position: WindowGeometry {
                            x: window.x as i16,
                            y: window.y as i16,
                            width: window.width as u16,
                            height: window.height as u16,
                        },
                    }
                });
            }
        }
        Ok(table_windows)
    }


}

#[derive(Debug)]
struct WindowsWindow {
    text: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}


fn get_windows() -> Result<Vec<WindowsWindow>, ApplicationError> {
    let state: Box<Vec<WindowsWindow>> = Box::new(Vec::new());
    let ptr = Box::into_raw(state);
    let state;
    unsafe {
        EnumWindows(Some(enum_window), LPARAM(ptr as isize))?; // unsafe
        state = Box::from_raw(ptr); // unsafe
    };
    Ok(*state)
}


fn get_window_text(window: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetWindowTextW(window, &mut text) };
    let text = String::from_utf16_lossy(&text[..len as usize]);
    text.to_owned()
}

fn get_window_info(window: HWND) -> WINDOWINFO {
    let mut info = WINDOWINFO {
        cbSize: core::mem::size_of::<WINDOWINFO>() as u32,
        ..Default::default()
    };
    unsafe {
        GetWindowInfo(window, &mut info).unwrap();
    }
    info.to_owned()
}


extern "system" fn enum_window(window: HWND, state: LPARAM) -> BOOL {
    let state = state.0 as *mut Vec<WindowsWindow>;

    let window_text = get_window_text(window);
    let info = get_window_info(window);

    let win_x = info.rcWindow.left;
    let win_y = info.rcWindow.top;
    let win_width = info.rcWindow.right - info.rcWindow.left;
    let win_height = info.rcWindow.bottom - info.rcWindow.top;
    if !window_text.is_empty() && info.dwStyle.contains(WS_VISIBLE) {
        unsafe {
            (*state).push(WindowsWindow {
                text: window_text,
                x: win_x,
                y: win_y,
                width: win_width,
                height: win_height,
            });
        }
    }

    true.into()
}