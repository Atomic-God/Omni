use std::env;

pub enum OsType {
    Linux,
    Windows,
    MacOs,
    Unknown,
}

pub struct OsLayer;

impl OsLayer {
    pub fn detect() -> OsType {
        match env::consts::OS {
            "linux" => OsType::Linux,
            "windows" => OsType::Windows,
            "macos" => OsType::MacOs,
            _ => OsType::Unknown,
        }
    }

    pub fn config_root() -> std::path::PathBuf {
        directories::ProjectDirs::from("com", "omni-forge", "omni-forge")
            .map(|p| p.config_dir().to_path_buf())
            .unwrap_or_else(|| std::path::PathBuf::from("."))
    }
}
