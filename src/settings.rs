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

        // 收集所有 entries 以便同时扫描 PNG 和 DLL
        let mut all_entries: Vec<std::fs::DirEntry> = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
            for entry in entries {
                if let Ok(e) = entry {
                    all_entries.push(e);
                }
            }
        }

        // 1. 扫描 root-level PNG 文件（DLL 和 PNG 直接放在 plugins/ 下的情况）
        let mut root_pngs = Vec::new();
        for entry in &all_entries {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_lowercase().ends_with(".png") {
                root_pngs.push(name);
            }
        }
        if !root_pngs.is_empty() {
            result.push(PluginInfo {
                folder_name: "(plugins 根目录)".to_string(),
                png_files: root_pngs,
            });
        }

        // 2. 扫描含 "trait" 的子文件夹
        for entry in &all_entries {
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
                    if let Ok(se) = sub_entry {
                        let sub_name = se.file_name().to_string_lossy().to_string();
                        let sub_name_lower = sub_name.to_lowercase();
                        if sub_name_lower.ends_with(".png") {
                            png_files.push(sub_name);
                        }
                    }
                }
            }

            result.push(PluginInfo {
                folder_name,
                png_files,
            });
        }

        // 3. 扫描 FancyTraits 魔改版（文件夹或 DLL）
        //    FancyTraits 通过 DLL 内嵌精灵图，不依赖外部 PNG 文件
        //    文件名包含 "fancytraits" 即视为 FancyTraits 插件
        let mut found_fancytraits = false;
        for entry in &all_entries {
            let name = entry.file_name().to_string_lossy().to_string();
            let name_lower = name.to_lowercase();
            if name_lower.contains("fancytraits") {
                if !found_fancytraits {
                    // FancyTraits 注册的6个升级：3个特质 + 3个装备
                    result.push(PluginInfo {
                        folder_name: format!("FancyTraits (魔改版: {})", name),
                        png_files: vec![
                            "$$fancytraits_Hero_Trait_UltimateSquad".to_string(),
                            "$$fancytraits_Hero_Trait_Yuri".to_string(),
                            "$$fancytraits_Hero_Trait_Slash".to_string(),
                            "$$fancytraits_Hero_Item_Charge".to_string(),
                            "$$fancytraits_Hero_Item_FrontArmor".to_string(),
                            "$$fancytraits_Hero_Item_SpeedUp".to_string(),
                        ],
                    });
                    found_fancytraits = true;
                }
            }
        }

        if result.is_empty() {
            return Err("未找到任何 PNG 图片或含 Trait 的插件文件夹".to_string());
        }

        Ok(result)
    }

    pub fn get_detected_trait_codes(plugins_info: &[PluginInfo]) -> Vec<String> {
        let mut codes = Vec::new();

        let png_map: Vec<(&str, &str)> = vec![
            // 简单命名（旧版）
            ("trait_axe.png", "Hero_Trait_AxeThrower"),
            ("trait_cheaperclass.png", "Hero_Trait_CheaperClass"),
            ("trait_regenerative.png", "Hero_Trait_Regenerative"),
            ("trait_thorns.png", "Hero_Trait_Thorns"),
            // PlentyTraits 2.0 复杂命名
            ("trueaxe.png", "Hero_Trait_AxeThrower"),
            ("trait_thorns.png", "Hero_Trait_Thorns"),
            ("mesugaki.png", "Hero_Trait_CheaperClass"),
            ("trait_regenerative.png", "Hero_Trait_Regenerative"),
            ("jump.png", "Hero_Trait_Jumper"),
            ("creeper.png", "Hero_Trait_Creeper"),
            ("mystory.png", "Hero_Trait_Flyer"),
            ("titan.png", "Hero_Trait_Titan"),
            ("charge.png", "Hero_Item_Charge"),
        ];

        for plugin in plugins_info {
            for png_file in &plugin.png_files {
                let lower = png_file.to_lowercase();

                // FancyTraits DLL 内嵌升级：直接提取代码名
                if lower.starts_with("$$fancytraits_") {
                    let code = &png_file[14..]; // 去掉 "$$fancytraits_" 前缀
                    if !codes.contains(&code.to_string()) {
                        codes.push(code.to_string());
                    }
                    continue;
                }

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

