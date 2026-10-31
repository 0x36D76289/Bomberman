use std::{env, path::PathBuf, sync::OnceLock};

fn data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        return PathBuf::from(env::var("APPDATA").unwrap_or_else(|_| ".".to_string()))
            .join("Bomberman");
    }
    if let Ok(xdg) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg);
    }
    env::home_dir().map_or_else(
        || PathBuf::from("."),
        |home| {
            home.join(if cfg!(target_os = "macos") {
                "Library/Application Support"
            } else {
                ".local/share"
            })
        },
    )
}

pub fn get_settings_path() -> &'static PathBuf {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| data_dir().join("bomberman.save"))
}
