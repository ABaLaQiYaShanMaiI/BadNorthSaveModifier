use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub folder_name: String,
    pub png_files: Vec<String>,
}

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
    #[serde(default)]
    pub game_folder_path: Option<PathBuf>,
    #[serde(default)]
    pub plugins_info: Vec<PluginInfo>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            color_mode: ColorMode::default(),
            language: Language::default(),
            editor_exe_path: None,
            keep_logs_visible: false,
            game_folder_path: None,
            plugins_info: Vec::new(),
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

    pub fn scan_plugins(game_root: &PathBuf) -> Result<Vec<PluginInfo>, String> {
        let plugins_dir = game_root.join("BepInEx").join("plugins");
        if !plugins_dir.exists() || !plugins_dir.is_dir() {
            return Err(format!(
                "未找到 BepInEx\\plugins 目录: {}",
                plugins_dir.display()
            ));
        }

        let mut result = Vec::new();

        let entries = match std::fs::read_dir(&plugins_dir) {
            Ok(e) => e,
            Err(e) => return Err(format!("无法读取 plugins 目录: {}", e)),
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };

            if !file_type.is_dir() {
                continue;
            }

            let folder_name = entry.file_name().to_string_lossy().to_string();

            let folder_name_lower = folder_name.to_lowercase();
            if !folder_name_lower.contains("trait") {
                continue;
            }

            let mut png_files = Vec::new();
            if let Ok(sub_entries) = std::fs::read_dir(entry.path()) {
                for sub_entry in sub_entries {
                    let sub_entry = match sub_entry {
                        Ok(se) => se,
                        Err(_) => continue,
                    };
                    let sub_name = sub_entry.file_name().to_string_lossy().to_string();
                    let sub_name_lower = sub_name.to_lowercase();
                    if sub_name_lower.ends_with(".png") {
                        png_files.push(sub_name);
                    }
                }
            }

            result.push(PluginInfo {
                folder_name,
                png_files,
            });
        }

        if result.is_empty() {
            return Err("未找到任何含 Trait 的插件文件夹，或文件夹内无 PNG 图片".to_string());
        }

        Ok(result)
    }

    pub fn get_detected_trait_codes(plugins_info: &[PluginInfo]) -> Vec<String> {
        let mut codes = Vec::new();

        let png_map: Vec<(&str, &str)> = vec![
            ("trait_axe.png", "Hero_Trait_AxeThrower"),
            ("trait_cheaperclass.png", "Hero_Trait_CheaperClass"),
            ("trait_regenerative.png", "Hero_Trait_Regenerative"),
            ("trait_thorns.png", "Hero_Trait_Thorns"),
        ];

        for plugin in plugins_info {
            for png_file in &plugin.png_files {
                let lower = png_file.to_lowercase();
                for (png_name, trait_code) in &png_map {
                    if lower == *png_name {
                        if !codes.contains(&trait_code.to_string()) {
                            codes.push(trait_code.to_string());
                        }
                    }
                }
            }
        }

        codes
    }

    pub fn has_png(plugins_info: &[PluginInfo], png_name: &str) -> bool {
        let lower = png_name.to_lowercase();
        for plugin in plugins_info {
            for png_file in &plugin.png_files {
                if png_file.to_lowercase() == lower {
                    return true;
                }
            }
        }
        false
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

