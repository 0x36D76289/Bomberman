use std::sync::Mutex;

fn settings_path(path: Option<String>) -> String {
    static SETTINGS_PATH: Mutex<String> = Mutex::new(String::new());

    if let Some(path) = path {
        *SETTINGS_PATH.lock().unwrap() = path
    }
    SETTINGS_PATH.lock().unwrap().clone()
}

const FILENAME: &str = "bomberman.save";

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

pub fn get_settings_path() -> String {
    settings_path(None)
}
