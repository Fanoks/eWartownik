use std::fs::create_dir_all;
use std::path::PathBuf;

use cfg_if::cfg_if;

pub(super) fn db_path() -> PathBuf {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            // NOTE: This is currently hardcoded for the author's machine.
            // Consider switching back to `dirs::data_dir()` (commented below) for portability.
            // let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            // let dir = base.join("eWartownik");
            let dir = PathBuf::from("C:/Users/fanok/Desktop/programowanie/Rust/eWartownik/db");
        } else if #[cfg(target_os = "linux")] {
            let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let dir = base.join("eWartownik");
        } else if #[cfg(target_os = "macos")] {
            let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let dir = base.join("eWartownik");
        } else if #[cfg(target_os = "android")] {
            let dir = PathBuf::from("/data/data/com.ewartownik.app/files/eWartownik");
        } else if #[cfg(target_os = "ios")] {
            // I'm not sure if it will work
            let dir = PathBuf::from("/var/mobile/Containers/Data/Application/Documents/eWartownik");
        } else {
            let dir = PathBuf::from("eWartownik");
        }
    }

    if let Err(e) = create_dir_all(&dir) {
        eprintln!("Couldn't create directory: {e}");
    }

    dir.join("database.db")
}
