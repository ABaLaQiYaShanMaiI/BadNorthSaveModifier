use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ColorMode {
    Black,
    Colorful,
    FollowSystem,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Black
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Language {
    Chinese,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Language::Chinese
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    pub color_mode: ColorMode,
    pub language: Language,
    #[serde(default)]
    pub editor_exe_path: Option<PathBuf>,
    #[serde(default)]
    pub keep_logs_visible: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::default(),
            language: Language::default(),
            editor_exe_path: None,
            keep_logs_visible: false,
        }
    }
}

impl AppSettings {

    #[allow(dead_code)]
    pub fn is_editor_exe_valid(&self) -> bool {
        if let Some(ref path) = self.editor_exe_path {
            path.is_file()
        } else {
            false
        }
    }
}

impl AppSettings {

    pub fn get_settings_path() -> PathBuf {

        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("settings.json");
            }
        }

        PathBuf::from("settings.json")
    }

    pub fn load() -> Self {
        let path = Self::get_settings_path();
        
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<AppSettings>(&content) {
                        Ok(settings) => return settings,
                        Err(_) => {
                            log::warn!("Failed to parse settings file, using defaults");
                        }
                    }
                }
                Err(_) => {
                    log::warn!("Failed to read settings file, using defaults");
                }
            }
        }
        
        AppSettings::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_settings_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}

