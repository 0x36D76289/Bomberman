use std::sync::Mutex;

/// The singleton to get / set the settings path, give a String to set or None to get
fn settings_path(path: Option<String>) -> String {
    static SETTINGS_PATH: Mutex<String> = Mutex::new(String::new());

    if let Some(path) = path {
        *SETTINGS_PATH.lock().unwrap() = path
    }
    SETTINGS_PATH.lock().unwrap().clone()
}

const FILENAME: &str = "bomberman.save";

/// The public setter of the settings path (OS dependent)
/// The linux path tries to query XDG_DATA_HOME, then $HOME/.local/share, then ./
#[cfg(target_os = "linux")]
pub fn init_settings_path() {
    let dir = 'dir: {
        let xdg = std::env::var("XDG_DATA_HOME");
        if xdg.is_ok() {
            break 'dir xdg.unwrap();
        }
        let home = std::env::home_dir();
        if home.is_some() {
            break 'dir home.unwrap().to_string_lossy().to_string() + "/.local/share";
        }
        ".".to_string()
    };
    let path = dir + "/" + FILENAME;
    settings_path(Some(path));
}

/// The public setter of the settings path (OS dependent)
/// The macos path tries to query XDG_DATA_HOME, then $HOME/Library/Application Support, then ./
#[cfg(target_os = "macos")]
pub fn init_settings_path() {
    let dir = 'dir: {
        let xdg = std::env::var("XDG_DATA_HOME");
        if xdg.is_ok() {
            break 'dir xdg.unwrap();
        }
        let home = std::env::home_dir();
        if home.is_some() {
            break 'dir home.unwrap().to_string_lossy().to_string()
                + "/Library/Application Support";
        }
        ".".to_string()
    };
    let path = dir + "/" + FILENAME;
    settings_path(Some(path));
}

/// The public setter of the settings path (OS dependent)
/// The windows path tries to query $APPDATA, then ./
#[cfg(target_os = "windows")]
pub fn init_settings_path() {
    let dir = 'dir: {
        if let Ok(appdata) = std::env::var("APPDATA") {
            break 'dir appdata;
        }
        ".".to_string()
    };
    let path = dir + "\\Bomberman";
    settings_path(Some(path + "\\" + FILENAME));
}

/// The public getter for the path singleton
pub fn get_settings_path() -> String {
    settings_path(None)
}
