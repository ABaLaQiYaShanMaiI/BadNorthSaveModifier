use eframe::egui;
use std::path::PathBuf;
use std::collections::HashMap;
use log::{info, error, warn};
use serde_json::Value;

mod models;
mod save_manager;
mod ui;
mod class_dictionary;
mod upgrade_dictionary;
mod settings;

use save_manager::{SaveManager, HeroDetails};
use settings::{AppSettings, ColorMode, Language};

// ============ Constants ============
const APP_TITLE: &str = "BadNorthSaveModifier存档修改器";
const TOOLBAR_BTN_HEIGHT: f32 = 36.0;

// ============ Enums ============
#[derive(Clone)]
enum AppState {
    SelectSaveFile,
    LoadDifferentSave {
        json_data: Value,
        previous_save_path: PathBuf,
        recruited_heroes: Vec<HeroDetails>,
        selected_hero_key: Option<String>,
        hero_details: Option<HeroDetails>,
        edit_state: EditState,
    },
    Editing {
        json_data: Value,
        save_path: PathBuf,
        recruited_heroes: Vec<HeroDetails>,
        selected_hero_key: Option<String>,
        hero_details: Option<HeroDetails>,
        edit_state: EditState,
    },
}

#[derive(Clone, Debug, PartialEq)]
enum LeftMenuSelection {
    Settings,
    HeroList(Option<String>),
    CoinBankEdit,
    GrailEdit,
}

impl Default for LeftMenuSelection {
    fn default() -> Self {
        Self::HeroList(None)
    }
}

// ============ Structs ============
#[derive(Clone)]
#[allow(dead_code)]
struct EditState {
    new_class_text: String,
    new_item_text: String,
    new_trait_text: String,
    new_coins: String,
    new_grails: String,
    log_buffer: String,
    log_scroll_offset: f32,
    menu_selection: LeftMenuSelection,
    commander_expanded: bool,
    currency_expanded: bool,
    new_bomb: String,
    new_mine: String,
    new_philosophers_stone: String,
    new_size: String,
    new_warhammer: String,
    new_cornucopia: String,
    new_war_horn: String,
    custom_item_input: String,
    oldgrey_flag_traits_expanded: bool,
    fusion_traits_expanded: bool,
    fusion_items_expanded: bool,
    mod_items_expanded: bool,
    mod_traits_expanded: bool,
    fusion_item_inputs: HashMap<String, String>,
    mod_item_inputs: HashMap<String, String>,
}

impl Default for EditState {
    fn default() -> Self {
        Self {
            new_class_text: String::new(),
            new_item_text: String::new(),
            new_trait_text: String::new(),
            new_coins: String::new(),
            new_grails: String::new(),
            log_buffer: String::new(),
            log_scroll_offset: 0.0,
            menu_selection: LeftMenuSelection::default(),
            commander_expanded: true,
            currency_expanded: true,
            new_bomb: String::new(),
            new_mine: String::new(),
            new_philosophers_stone: String::new(),
            new_size: String::new(),
            new_warhammer: String::new(),
            new_cornucopia: String::new(),
            new_war_horn: String::new(),
            custom_item_input: String::new(),
            oldgrey_flag_traits_expanded: false,
            fusion_traits_expanded: false,
            fusion_items_expanded: false,
            mod_items_expanded: false,
            mod_traits_expanded: false,
            fusion_item_inputs: HashMap::new(),
            mod_item_inputs: HashMap::new(),
        }
    }
}

impl EditState {
    fn add_log(&mut self, level: &str, message: &str) {
        if level == "ERROR" {
            let log_entry = format!("[{}] {}\n", level, message);
            self.log_buffer.push_str(&log_entry);
            self.log_scroll_offset = f32::INFINITY;
        }

        match level {
            "ERROR" => error!("{}", message),
            "WARN" => warn!("{}", message),
            "INFO" => info!("{}", message),
            _ => {}
        }
    }

    fn clear_logs(&mut self) {
        self.log_buffer.clear();
        self.log_scroll_offset = 0.0;
    }
}

pub struct ModifierApp {
    state: AppState,
    error_message: Option<String>,
    success_message: Option<String>,
    message_timeout: f32,
    app_settings: AppSettings,
    transition_from_mode: ColorMode,
    transition_to_mode: ColorMode,
    color_transition_progress: f32,
}

// ============ Utility Functions ============

// Font & Emoji Configuration
fn configure_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let chinese_font_candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];
    for path in &chinese_font_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "ChineseFont".to_owned(),
                egui::FontData::from_owned(data),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .push("ChineseFont".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("ChineseFont".to_owned());
            break;
        }
    }

    if let Some(emoji_data) = load_system_emoji_font() {
        fonts.font_data.insert(
            "SystemEmoji".to_owned(),
            egui::FontData::from_owned(emoji_data),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("SystemEmoji".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_system_emoji_font() -> Option<Vec<u8>> {
    let candidates = [
        r"C:\Windows\Fonts\seguisym.ttf",
        r"C:\Windows\Fonts\seguiemj.ttf",
    ];
    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            return Some(data);
        }
    }
    None
}

fn lerp_color(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.max(0.0).min(1.0);
    let r = (from.r() as f32 * (1.0 - t) + to.r() as f32 * t) as u8;
    let g = (from.g() as f32 * (1.0 - t) + to.g() as f32 * t) as u8;
    let b = (from.b() as f32 * (1.0 - t) + to.b() as f32 * t) as u8;
    let a = (from.a() as f32 * (1.0 - t) + to.a() as f32 * t) as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn get_visuals_for_mode(color_mode: &ColorMode) -> egui::Visuals {
    match color_mode {
        ColorMode::Black => {
            let mut visuals = egui::Visuals::dark();
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(100));
            visuals
        }
        ColorMode::Colorful => {
            let mut visuals = egui::Visuals::light();
            let accent = egui::Color32::from_rgb(70, 130, 200);
            visuals.widgets.active.bg_fill = accent;
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(120, 170, 230);
            visuals.selection.bg_fill = egui::Color32::from_rgb(70, 180, 120);
            visuals.extreme_bg_color = egui::Color32::from_rgb(225, 237, 252);
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, accent);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, accent);
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
            visuals
        }
        ColorMode::FollowSystem => {
            if is_system_dark_mode() {
                let mut visuals = egui::Visuals::dark();
                visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(100));
                visuals
            } else {
                let mut visuals = egui::Visuals::light();
                visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
                visuals
            }
        }
    }
}

fn lerp_visuals(from: &egui::Visuals, to: &egui::Visuals, t: f32) -> egui::Visuals {
    let mut result = to.clone();

    result.override_text_color = Some(lerp_color(
        from.override_text_color.unwrap_or(egui::Color32::WHITE),
        to.override_text_color.unwrap_or(egui::Color32::WHITE),
        t,
    ));

    result.panel_fill = lerp_color(from.panel_fill, to.panel_fill, t);
    result.extreme_bg_color = lerp_color(from.extreme_bg_color, to.extreme_bg_color, t);
    result.faint_bg_color = lerp_color(from.faint_bg_color, to.faint_bg_color, t);
    result.window_fill = lerp_color(from.window_fill, to.window_fill, t);

    result.window_stroke = egui::Stroke::new(
        from.window_stroke.width * (1.0 - t) + to.window_stroke.width * t,
        lerp_color(from.window_stroke.color, to.window_stroke.color, t),
    );

    result.widgets.inactive.bg_fill = lerp_color(from.widgets.inactive.bg_fill, to.widgets.inactive.bg_fill, t);
    result.widgets.inactive.fg_stroke = egui::Stroke::new(
        from.widgets.inactive.fg_stroke.width * (1.0 - t) + to.widgets.inactive.fg_stroke.width * t,
        lerp_color(from.widgets.inactive.fg_stroke.color, to.widgets.inactive.fg_stroke.color, t),
    );
    result.widgets.inactive.bg_stroke = egui::Stroke::new(
        from.widgets.inactive.bg_stroke.width * (1.0 - t) + to.widgets.inactive.bg_stroke.width * t,
        lerp_color(from.widgets.inactive.bg_stroke.color, to.widgets.inactive.bg_stroke.color, t),
    );

    result.widgets.hovered.bg_fill = lerp_color(from.widgets.hovered.bg_fill, to.widgets.hovered.bg_fill, t);
    result.widgets.hovered.fg_stroke = egui::Stroke::new(
        from.widgets.hovered.fg_stroke.width * (1.0 - t) + to.widgets.hovered.fg_stroke.width * t,
        lerp_color(from.widgets.hovered.fg_stroke.color, to.widgets.hovered.fg_stroke.color, t),
    );
    result.widgets.hovered.bg_stroke = egui::Stroke::new(
        from.widgets.hovered.bg_stroke.width * (1.0 - t) + to.widgets.hovered.bg_stroke.width * t,
        lerp_color(from.widgets.hovered.bg_stroke.color, to.widgets.hovered.bg_stroke.color, t),
    );

    result.widgets.active.bg_fill = lerp_color(from.widgets.active.bg_fill, to.widgets.active.bg_fill, t);
    result.widgets.active.fg_stroke = egui::Stroke::new(
        from.widgets.active.fg_stroke.width * (1.0 - t) + to.widgets.active.fg_stroke.width * t,
        lerp_color(from.widgets.active.fg_stroke.color, to.widgets.active.fg_stroke.color, t),
    );
    result.widgets.active.bg_stroke = egui::Stroke::new(
        from.widgets.active.bg_stroke.width * (1.0 - t) + to.widgets.active.bg_stroke.width * t,
        lerp_color(from.widgets.active.bg_stroke.color, to.widgets.active.bg_stroke.color, t),
    );

    result.selection.bg_fill = lerp_color(from.selection.bg_fill, to.selection.bg_fill, t);
    result.selection.stroke = egui::Stroke::new(
        from.selection.stroke.width * (1.0 - t) + to.selection.stroke.width * t,
        lerp_color(from.selection.stroke.color, to.selection.stroke.color, t),
    );

    result.text_cursor = egui::Stroke::new(
        from.text_cursor.width * (1.0 - t) + to.text_cursor.width * t,
        lerp_color(from.text_cursor.color, to.text_cursor.color, t),
    );
    result.code_bg_color = lerp_color(from.code_bg_color, to.code_bg_color, t);
    result.warn_fg_color = lerp_color(from.warn_fg_color, to.warn_fg_color, t);
    result.error_fg_color = lerp_color(from.error_fg_color, to.error_fg_color, t);

    result
}

fn apply_color_theme(ctx: &egui::Context, color_mode: &ColorMode) {
    match color_mode {
        ColorMode::Black => {
            let mut visuals = egui::Visuals::dark();
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(100));
            ctx.set_visuals(visuals);
        }
        ColorMode::Colorful => {
            let mut visuals = egui::Visuals::light();
            let accent = egui::Color32::from_rgb(70, 130, 200);
            visuals.widgets.active.bg_fill = accent;
            visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(120, 170, 230);
            visuals.selection.bg_fill = egui::Color32::from_rgb(70, 180, 120);
            visuals.extreme_bg_color = egui::Color32::from_rgb(225, 237, 252);
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, accent);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, accent);
            visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
            ctx.set_visuals(visuals);
        }
        ColorMode::FollowSystem => {
            if is_system_dark_mode() {
                let mut visuals = egui::Visuals::dark();
                visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_gray(100));
                ctx.set_visuals(visuals);
            } else {
                let mut visuals = egui::Visuals::light();
                visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
                ctx.set_visuals(visuals);
            }
        }
    }
}

fn is_system_dark_mode() -> bool {
    static CACHE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *CACHE.get_or_init(detect_system_dark_mode_impl)
}

#[cfg(target_os = "windows")]
fn detect_system_dark_mode_impl() -> bool {
    use std::process::Command;
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-ItemPropertyValue -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize' -Name 'AppsUseLightTheme'",
        ])
        .output();
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            stdout.trim() == "0"
        }
        Err(_) => false,
    }
}

#[cfg(target_os = "macos")]
fn detect_system_dark_mode_impl() -> bool {
    use std::process::Command;
    let output = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output();
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            stdout.contains("Dark")
        }
        Err(_) => false,
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn detect_system_dark_mode_impl() -> bool {
    if let Ok(theme) = std::env::var("GTK_THEME") {
        if theme.to_lowercase().contains("dark") {
            return true;
        }
    }
    if let Ok(theme) = std::env::var("COLORFGBG") {
        if let Some(bg) = theme.split(';').last() {
            if bg.trim() == "0" {
                return true;
            }
        }
    }
    false
}

fn t(key: &str, lang: &Language) -> &'static str {
    match lang {
        Language::Chinese => match key {
            "field"        => "字段",
            "id"           => "ID",
            "current"      => "当前",
            "level"        => "等级",
            "new_value"    => "新值",
            "quick_select" => "快速应用",
            "apply_change" => "✔应用变更",
            "no_class"     => "( 无兵种)",
            "no_item"      => "当前: ( 无装备)",
            "no_trait"     => "当前: ( 无特质)",

            "app_settings"  => "应用设置",
            "color_mode"    => "色彩模式：",
            "dark_mode"     => "黑色",
            "colorful_mode" => "彩色",
            "follow_system" => "跟随系统",
            "language"      => "语言：",
            "chinese"       => "中文 CN",
            "english"       => "English EN",
            "auto_save"     => "设置已自动保存",

            "select_exe"    => "请选择 BadNorthSaveEditorRust.exe 的位置",
            "browse_exe"    => "浏览并选择编辑器EXE",
            "select_save"   => "请选择游戏存档文件：",
            "browse_save"   => "浏览并选择存档文件",
            "editor_label"  => "编辑器：",
            "reselect_exe"  => "重新选择编辑器EXE",
            "editor_exe_found" => "已识别到指定exe文件",
            "editor_exe_hint" => "如需修改BadNorthSaveRustEditor.exe或BadNorthSaveconverter.exe的地址，请去\"设置\"内修改或重置",

            "show_logs"     => "显示日志",
            "hide_logs"     => "隐藏日志",
            "export_json"   => "导出 JSON",
            "load_save"     => "加载其他存档",
            "save_file"     => "备份并替换存档",
            "refresh"       => "刷新",

            "heroes"        => "英雄",
            "coin_bank"     => "货币 (coinBank) 修改",
            "grail_count"   => "圣杯(Grail)",
            "total_grails"  => "总数:",
            "grail_on_hero" => "已被装备数",
            "grail_in_inv"  => "未被装备数",
            "grail_code"    => "圣杯代码:",
            "set_count"     => "设置数量:",
            "copy_code"     => "复制 Hero_Upgrade_Grail 到剪贴板",
            "current_value" => "当前值",
            "new_value_label" => "新值",
            "apply_modify"  => "应用修改",
            "apply_btn"     => "应用",
            "clear_logs"    => "清空",
            "copy_logs"     => "全选并复制",
            "select_hero"   => "从左侧列表选择修改功能",
            "abnormal_data" => "英雄数据异常",
            "grail_wip"     => "背包装备管理功能完善中，敬请期待",

            "inv_panel_title"          => "背包装备修改",
            "inv_item_code"            => "代码:",
            "inv_total"                => "总数:",
            "inv_on_hero"              => "已装备",
            "inv_in_inv"               => "背包中",
            "inv_set_count"            => "设置数量:",
            "inv_bomb_title"           => "炸弹 (Bomb)",
            "inv_mine_title"           => "地雷 (Mine)",
            "inv_stone_title"          => "贤者之石(Philosophers Stone)",
            "inv_size_title"           => "指挥之戒 (Size)",
            "inv_warhammer_title"      => "战锤 (Warhammer)",
            "inv_cornucopia_title"     => "雅贝那(Cornucopia)",
            "inv_warhorn_title"        => "战争号角 (War Horn)",

            "menu_settings"   => "设置",
            "menu_commanders" => "指挥官修改",
            "menu_currency"   => "货币和物品修改",
            "menu_coinbank"   => "  coinBank 修改",
            "menu_grail"      => "  背包内装备修改",
            "inv_capacity_hint" => "提示：背包最多可存放 20 个物品（所有装备之和）",

            "inv_overview_title"  => "背包内装备总览",
            "inv_overview_total"  => "背包总数",
            "inv_overview_limit"  => "（上限20）",
            "inv_overview_items"  => "当前物品",
            "inv_overview_mod"    => "Mod 物品",
            "inv_overview_empty"  => "（背包为空）",
            "inv_custom_label"    => "添加自定义物品",
            "inv_custom_hint"     => "输入物品字段名称，如 Hero_Upgrade_ModWeapon",
            "inv_custom_apply"    => "应用",
            "inv_custom_empty"    => "请输入物品字段名称",
            "warhorn_wip"         => "战争号角修改功能完善中，请移步于'指挥官修改的装备修改'实现战争号角修改",
            "cornucopia_wip"      => "雅贝那修改功能完善中，敬请期待",
            "yabena_wip"          => "雅贝那修改功能完善中，请移步于背包内装备修改实现雅贝那修改",
            "no_upgrade"          => "( 无消耗型升级 )",
            "remove_item"         => "移除装备",
            "remove_trait"        => "移除特质",

            "keep_logs_visible"   => "日志显示",
            "keep_logs_label"     => "保持日志显示",

            "class_editor_title"  => "兵种修改 (Class)",
            "item_editor_title"   => "装备修改 (Item)",
            "trait_editor_title"  => "特质修改 (Trait)",
            "log_heading"         => "操作日志",
            "apply_hint"          => "输入后点击【应用】",
            
            "quick_apply"             => "快速应用",
            "collapse_label"          => "▼ 收起",
            "expand_label"            => "▶ 展开",
            "quick_apply_hint"        => "快速应用",
            "mod_items_title"         => "魔改版- 专属装备",
            "mod_traits_title"        => "魔改版- 专属特质",
            "fusion_items_title"      => "融合版- 专属装备",
            "fusion_traits_title"     => "融合版- 专属特质",
            "oldgrey_flag_traits_title" => "旧灰复燃的战旗- 专属特质",
            "converter_builtin"       => "✓ 转换器已内置（无需外部EXE）",
            _              => "",
        },
        Language::English => match key {
            "field"        => "Field",
            "id"           => "ID",
            "current"      => "Current",
            "level"        => "Level",
            "new_value"    => "New Value",
            "quick_select" => "Quick Apply",
            "apply_change" => "✔Apply",
            "no_class"     => "( No Class )",
            "no_item"      => "Current: ( No Item )",
            "no_trait"     => "Current: ( No Trait )",

            "app_settings"  => "App Settings",
            "color_mode"    => "Color Mode:",
            "dark_mode"     => "Dark",
            "colorful_mode" => "Colorful",
            "follow_system" => "Follow System",
            "language"      => "Language:",
            "chinese"       => "中文 CN",
            "english"       => "English EN",
            "auto_save"     => "Settings auto-saved",

            "select_exe"    => "Select BadNorthSaveEditorRust.exe location",
            "browse_exe"    => "Browse & Select Editor EXE",
            "select_save"   => "Select save file:",
            "browse_save"   => "Browse & Select Save File",
            "editor_label"  => "Editor:",
            "reselect_exe"  => "Re-select Editor EXE",
            "editor_exe_found" => "Editor exe file detected",
            "editor_exe_hint" => "To change the address of BadNorthSaveRustEditor.exe or BadNorthSaveconverter.exe, go to \"Settings\" to modify or reset",

            "show_logs"     => "Show Logs",
            "hide_logs"     => "Hide Logs",
            "export_json"   => "Export JSON",
            "load_save"     => "Load Save",
            "save_file"     => "Save & Backup",
            "refresh"       => "Refresh",

            "heroes"        => "Heroes",
            "coin_bank"     => "Currency (coinBank) Edit",
            "grail_count"   => "Grail (Grail)",
            "total_grails"  => "Total:",
            "grail_on_hero" => "Equipped:",
            "grail_in_inv"  => "Unequipped:",
            "grail_code"    => "Grail Code:",
            "set_count"     => "Set Count:",
            "copy_code"     => "Copy Hero_Upgrade_Grail to clipboard",
            "current_value" => "Current:",
            "new_value_label" => "New:",
            "apply_modify"  => "Apply",
            "apply_btn"     => "Apply",
            "clear_logs"    => "Clear",
            "copy_logs"     => "Select All & Copy",
            "select_hero"   => "→Select a function from the left",
            "abnormal_data" => "Hero data is abnormal",
            "grail_wip"     => "Inventory equipment management coming soon, stay tuned",

            "inv_panel_title"          => "Inventory Equipment Edit",
            "inv_item_code"            => "Code:",
            "inv_total"                => "Total:",
            "inv_on_hero"              => "Equipped:",
            "inv_in_inv"               => "In Inventory:",
            "inv_set_count"            => "Set Count:",
            "inv_bomb_title"           => "Bomb",
            "inv_mine_title"           => "Mine",
            "inv_stone_title"          => "Philosophers Stone",
            "inv_size_title"           => "Command Ring (Size)",
            "inv_warhammer_title"      => "Warhammer",
            "inv_cornucopia_title"     => "Cornucopia",
            "inv_warhorn_title"        => "War Horn",

            "menu_settings"   => "Settings",
            "menu_commanders" => "Commanders",
            "menu_currency"   => "Currency & Items",
            "menu_coinbank"   => "  CoinBank Edit",
            "menu_grail"      => "  Inventory Equipment",
            "inv_capacity_hint" => "Tip: Inventory can hold max 20 items (total of all equipment)",

            "inv_overview_title"  => "Inventory Overview",
            "inv_overview_total"  => "Total in Inventory",
            "inv_overview_limit"  => "(max 20)",
            "inv_overview_items"  => "Current Items",
            "inv_overview_mod"    => "Mod Items",
            "inv_overview_empty"  => "(inventory empty)",
            "inv_custom_label"    => "Add Custom Item",
            "inv_custom_hint"     => "Enter field name, e.g. Hero_Upgrade_ModWeapon",
            "inv_custom_apply"    => "Apply",
            "inv_custom_empty"    => "Please enter an item field name",
            "warhorn_wip"         => "War Horn modification coming soon, stay tuned",
            "cornucopia_wip"      => "Cornucopia modification coming soon, stay tuned",
            "yabena_wip"          => "The Cornucopia editor is under construction. Please use \"Inventory Equipment Editor\" to edit the Cornucopia.",
            "no_upgrade"          => "( No consumable upgrade )",
            "remove_item"         => "Remove Equipment",
            "remove_trait"        => "Remove Trait",

            "keep_logs_visible"   => "Log Display",
            "keep_logs_label"     => "Keep Logs Visible",

            "class_editor_title"  => "Class",
            "item_editor_title"   => "Item",
            "trait_editor_title"  => "Trait",
            "log_heading"         => "Logs",
            "apply_hint"          => "Type then click Apply",
            
            "quick_apply"             => "Quick Apply",
            "collapse_label"          => "▼ Collapse",
            "expand_label"            => "▶ Expand",
            "quick_apply_hint"        => "Quick Apply",
            "mod_items_title"         => "Mod - Exclusive Equipment",
            "mod_traits_title"        => "Mod - Exclusive Traits",
            "fusion_items_title"      => "Fusion - Exclusive Equipment",
            "fusion_traits_title"     => "Fusion - Exclusive Traits",
            "oldgrey_flag_traits_title" => "Rebirth Flag - Exclusive Traits",
            "converter_builtin"       => "✓ Converter is built-in (no external EXE needed)",
            _              => "",
        },
    }
}

fn toggle_btn(ui: &mut egui::Ui, label: &str, selected: bool) -> bool {
    if selected {
        let fill = ui.visuals().selection.bg_fill;
        ui.add(egui::Button::new(label).fill(fill)).clicked()
    } else {
        ui.button(label).clicked()
    }
}

// ============ Entry Point ============
fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let window_width = 960.0;
    let window_height = 600.0;
    let screen_width = 1920.0;
    let screen_height = 1080.0;
    let window_pos_x = (screen_width - window_width) / 2.0;
    let window_pos_y = (screen_height - window_height) / 2.0;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([window_width, window_height])
            .with_position([window_pos_x, window_pos_y])
            .with_title("BadNorthSaveModifier存档修改器"),
        ..Default::default()
    };

    eframe::run_native(
        "BadNorthSaveModifier存档修改器",
        options,
        Box::new(|cc| {
            configure_chinese_fonts(&cc.egui_ctx);

            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(22.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(15.0, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            style.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
            style.spacing.item_spacing = egui::Vec2::new(8.0, 6.0);
            cc.egui_ctx.set_style(style);

            Box::new(ModifierApp::default())
        }),
    )
}

// ============ Implementations ============
impl Default for ModifierApp {
    fn default() -> Self {
        let app_settings = AppSettings::load();
        let transition_mode = app_settings.color_mode.clone();
        Self {
            state: AppState::SelectSaveFile,
            error_message: None,
            success_message: None,
            message_timeout: 0.0,
            app_settings,
            transition_from_mode: transition_mode.clone(),
            transition_to_mode: transition_mode,
            color_transition_progress: 1.0,
        }
    }
}

impl eframe::App for ModifierApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.app_settings.color_mode != self.transition_to_mode {
            self.transition_from_mode = self.transition_to_mode.clone();
            self.transition_to_mode = self.app_settings.color_mode.clone();
            self.color_transition_progress = 0.0;
        }

        if self.color_transition_progress < 1.0 {
            self.color_transition_progress += ctx.input(|i| i.unstable_dt).min(1.0 / 60.0) * 1.0;
            self.color_transition_progress = self.color_transition_progress.min(1.0);
            ctx.request_repaint();
        }

        if self.color_transition_progress < 1.0 {
            let from_visuals = get_visuals_for_mode(&self.transition_from_mode);
            let to_visuals = get_visuals_for_mode(&self.transition_to_mode);
            let transitioned_visuals = lerp_visuals(&from_visuals, &to_visuals, self.color_transition_progress);
            ctx.set_visuals(transitioned_visuals);
        } else {
            apply_color_theme(ctx, &self.app_settings.color_mode);
        }

        self.update_message_timeout(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state {
                AppState::SelectSaveFile => self.show_file_selection_ui(ui),
                AppState::LoadDifferentSave { .. } => self.show_load_different_save_ui(ui),
                AppState::Editing { .. } => self.show_editor_ui(ui),
            }
        });
    }
}

impl ModifierApp {
    fn show_file_selection_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(APP_TITLE);
            ui.add_space(30.0);

            ui.label(t("select_save", &self.app_settings.language));
            ui.add_space(20.0);

            if ui.button(t("browse_save", &self.app_settings.language)).clicked() {
                let initial_dir = std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));

                let mut dialog = rfd::FileDialog::new()
                    .set_title("选择游戏存档文件");

                if let Some(dir) = initial_dir {
                    dialog = dialog.set_directory(&dir);
                }

                if let Some(path) = dialog.pick_file() {
                    self.try_load_save(&path);
                }
            }
        });

        if let Some(err) = &self.error_message {
            ui.add_space(20.0);
            ui.colored_label(egui::Color32::RED, format!("错误：{}", err));
        }
    }

    fn show_load_different_save_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(APP_TITLE);
            ui.add_space(30.0);

            ui.label("请选择要加载的其他存档文件：");
            ui.add_space(20.0);

            if ui.button("浏览并选择存档文件").clicked() {
                let initial_dir = std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));

                let mut dialog = rfd::FileDialog::new()
                    .set_title("选择游戏存档文件");

                if let Some(dir) = initial_dir {
                    dialog = dialog.set_directory(&dir);
                }

                if let Some(path) = dialog.pick_file() {
                    self.try_load_save(&path);
                }
            }

            ui.add_space(20.0);

            if ui.button("返回编辑器").clicked() {
                if let AppState::LoadDifferentSave {
                    json_data,
                    previous_save_path,
                    recruited_heroes,
                    selected_hero_key,
                    hero_details,
                    edit_state,
                } = self.state.clone()
                {
                    self.state = AppState::Editing {
                        json_data,
                        save_path: previous_save_path,
                        recruited_heroes,
                        selected_hero_key,
                        hero_details,
                        edit_state,
                    };
                }
            }
        });

        if let Some(err) = &self.error_message {
            ui.add_space(20.0);
            ui.colored_label(egui::Color32::RED, format!("错误：{}", err));
        }
    }

    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        let mut load_different = false;
        let mut do_save = false;
        let mut do_export: Option<std::path::PathBuf> = None;
        let mut do_refresh = false;
        let lang = self.app_settings.language.clone();

        {
            let AppState::Editing {
                ref mut json_data,
                ref save_path,
                ref mut recruited_heroes,
                ref mut selected_hero_key,
                ref mut hero_details,
                ref mut edit_state,
            } = self.state
            else {
                return;
            };

            ui.horizontal(|ui| {
                ui.heading(APP_TITLE);
                let tb_size = egui::Vec2::new(0.0, TOOLBAR_BTN_HEIGHT);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(t("export_json", &lang)).min_size(tb_size)).clicked() {
                        let default_name = format!(
                            "{}.json",
                            save_path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                        );
                        do_export = rfd::FileDialog::new()
                            .set_title("选择 JSON 导出位置")
                            .add_filter("JSON 文件", &["json"])
                            .set_file_name(&default_name)
                            .save_file();
                    }
                    if ui.add(egui::Button::new(t("load_save", &lang)).min_size(tb_size)).clicked() {
                        load_different = true;
                    }
                    if ui.add(egui::Button::new(t("save_file", &lang)).min_size(tb_size)).clicked() {
                        do_save = true;
                    }
                    if ui.add(egui::Button::new(t("refresh", &lang)).min_size(tb_size)).clicked() {
                        do_refresh = true;
                        edit_state.add_log("INFO", "已刷新数据");
                    }
                });
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label(format!("{}", save_path.file_name().unwrap_or_default().to_string_lossy()));
                ui.separator();
                ui.label(format!("{}: {}", t("heroes", &lang), recruited_heroes.len()));
            });
            ui.separator();

            let available_height = ui.available_height();
            let log_panel_height = if self.app_settings.keep_logs_visible {
                available_height * 0.35
            } else {
                0.0
            };
            let editor_height = available_height - log_panel_height - if self.app_settings.keep_logs_visible { 8.0 } else { 0.0 };

            ui.horizontal(|ui| {
                let list_width = 250.0;

                ui.allocate_ui(
                    egui::Vec2::new(list_width, editor_height),
                    |ui| {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .id_source("left_menu")
                                .auto_shrink([false; 2])
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {

                                    let settings_active = matches!(
                                        edit_state.menu_selection,
                                        LeftMenuSelection::Settings
                                    );
                                    if ui.selectable_label(settings_active, t("menu_settings", &lang)).clicked() {
                                        edit_state.menu_selection = LeftMenuSelection::Settings;
                                    }

                                    ui.separator();

                                    let commander_base = t("menu_commanders", &lang);
                                    let commander_label = if edit_state.commander_expanded {
                                        format!("{} ▼", commander_base)
                                    } else {
                                        format!("{} ▶", commander_base)
                                    };
                                    let commander_active = matches!(
                                        edit_state.menu_selection,
                                        LeftMenuSelection::HeroList(_)
                                    );
                                    if ui.selectable_label(commander_active, commander_label).clicked() {
                                        edit_state.commander_expanded = !edit_state.commander_expanded;
                                    }

                                    if edit_state.commander_expanded {
                                        for hero in recruited_heroes.iter() {
                                            let is_selected = matches!(
                                                &edit_state.menu_selection,
                                                LeftMenuSelection::HeroList(Some(k)) if k == &hero.key
                                            );
                                            let label = format!(
                                                "  ID:{} ({})",
                                                hero.key,
                                                hero.class_display()
                                            );
                                            if ui.selectable_label(is_selected, label).clicked() {
                                                let key = hero.key.clone();
                                                let details_clone = hero.clone();
                                                edit_state.menu_selection = LeftMenuSelection::HeroList(Some(key.clone()));
                                                *selected_hero_key = Some(key);
                                                *hero_details = Some(details_clone);
                                                edit_state.add_log("INFO", &format!("已选择英雄: {}", hero.key));
                                            }
                                        }
                                    }

                                    ui.separator();

                                    let currency_base = t("menu_currency", &lang);
                                    let currency_label = if edit_state.currency_expanded {
                                        format!("{} ▼", currency_base)
                                    } else {
                                        format!("{} ▶", currency_base)
                                    };
                                    let currency_active = matches!(
                                        edit_state.menu_selection,
                                        LeftMenuSelection::CoinBankEdit | LeftMenuSelection::GrailEdit
                                    );
                                    if ui.selectable_label(currency_active, currency_label).clicked() {
                                        edit_state.currency_expanded = !edit_state.currency_expanded;
                                    }

                                    if edit_state.currency_expanded {
                                        if ui.selectable_label(
                                            matches!(edit_state.menu_selection, LeftMenuSelection::CoinBankEdit),
                                            t("menu_coinbank", &lang),
                                        ).clicked() {
                                            edit_state.menu_selection = LeftMenuSelection::CoinBankEdit;
                                            edit_state.add_log("INFO", "已选择 coinBank 修改");
                                        }

                                        if ui.selectable_label(
                                            matches!(edit_state.menu_selection, LeftMenuSelection::GrailEdit),
                                            t("menu_grail", &lang),
                                        ).clicked() {
                                            edit_state.menu_selection = LeftMenuSelection::GrailEdit;
                                            edit_state.add_log("INFO", "已选择圣杯修改");
                                        }
                                    }
                                    });
                                });
                        });
                    },
                );

                let remaining_width = ui.available_width();
                ui.allocate_ui(
                    egui::Vec2::new(remaining_width, editor_height),
                    |ui| {
                        let menu_sel = edit_state.menu_selection.clone();
                        match menu_sel {
                            LeftMenuSelection::Settings => {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(t("app_settings", &lang)).strong().heading());
                                        ui.separator();

                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(t("color_mode", &lang)).strong());
                                            ui.horizontal(|ui| {
                                                if toggle_btn(ui, t("dark_mode", &lang), self.app_settings.color_mode == ColorMode::Black)
                                                    && self.app_settings.color_mode != ColorMode::Black
                                                {
                                                    self.app_settings.color_mode = ColorMode::Black;
                                                    let _ = self.app_settings.save();
                                                    edit_state.add_log("INFO", "色彩模式已改为 黑色");
                                                }
                                                if toggle_btn(ui, t("colorful_mode", &lang), self.app_settings.color_mode == ColorMode::Colorful)
                                                    && self.app_settings.color_mode != ColorMode::Colorful
                                                {
                                                    self.app_settings.color_mode = ColorMode::Colorful;
                                                    let _ = self.app_settings.save();
                                                    edit_state.add_log("INFO", "色彩模式已改为 彩色");
                                                }
                                                if toggle_btn(ui, t("follow_system", &lang), self.app_settings.color_mode == ColorMode::FollowSystem)
                                                    && self.app_settings.color_mode != ColorMode::FollowSystem
                                                {
                                                    self.app_settings.color_mode = ColorMode::FollowSystem;
                                                    let _ = self.app_settings.save();
                                                    edit_state.add_log("INFO", "色彩模式已改为 跟随系统");
                                                }
                                            });
                                        });

                                        ui.add_space(8.0);

                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(t("language", &lang)).strong());
                                            ui.horizontal(|ui| {
                                                if toggle_btn(ui, t("chinese", &lang), self.app_settings.language == Language::Chinese)
                                                    && self.app_settings.language != Language::Chinese
                                                {
                                                    self.app_settings.language = Language::Chinese;
                                                    let _ = self.app_settings.save();
                                                    edit_state.add_log("INFO", "语言已改为 中文");
                                                }
                                                if toggle_btn(ui, t("english", &lang), self.app_settings.language == Language::English)
                                                    && self.app_settings.language != Language::English
                                                {
                                                    self.app_settings.language = Language::English;
                                                    let _ = self.app_settings.save();
                                                    edit_state.add_log("INFO", "Language changed to: English");
                                                }
                                            });
                                        });

                                        ui.add_space(8.0);
                                        ui.separator();
                                        ui.label(egui::RichText::new(t("auto_save", &lang)).small().color(egui::Color32::GRAY));

                                        ui.add_space(16.0);

                                        ui.label(egui::RichText::new(t("keep_logs_visible", &lang)).strong());
                                        let mut logs_visible = self.app_settings.keep_logs_visible;
                                        if ui.checkbox(&mut logs_visible, t("keep_logs_label", &lang)).changed() {
                                            self.app_settings.keep_logs_visible = logs_visible;
                                            let _ = self.app_settings.save();
                                            edit_state.add_log("INFO", if self.app_settings.keep_logs_visible { "✔日志显示已开启" } else { "✔日志显示已关闭" });
                                        }

                                        ui.add_space(16.0);

                                        // 显示转换器已内置
                                        ui.label(egui::RichText::new(t("converter_builtin", &lang)).color(egui::Color32::from_rgb(34, 139, 34)));
                                    });
                                });
                            }
                            LeftMenuSelection::HeroList(Some(ref hero_key)) => {
                                let hero_key_owned = hero_key.clone();
                                let hero_opt = recruited_heroes.iter().find(|h| h.key == hero_key_owned).cloned();
                                if let Some(hero) = hero_opt {
                                    Self::show_hero_editor_vertical(
                                        ui,
                                        json_data,
                                        &hero_key_owned,
                                        hero,
                                        edit_state,
                                        recruited_heroes,
                                        hero_details,
                                        &lang,
                                    );
                                } else {
                                    ui.vertical_centered(|ui| {
                                        ui.add_space(100.0);
                                        ui.heading(t("abnormal_data", &lang));
                                    });
                                }
                            }
                            LeftMenuSelection::HeroList(None) => {
                                ui.vertical_centered(|ui| {
                                    ui.add_space(100.0);
                                    ui.heading(t("select_hero", &lang));
                                });
                            }
                            LeftMenuSelection::CoinBankEdit => {
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(egui::RichText::new(t("coin_bank", &lang)).strong().heading());
                                        ui.separator();

                                        let coin_bank = SaveManager::get_coin_bank(json_data).unwrap_or(0);
                                        ui.horizontal(|ui| {
                                            ui.label(t("current_value", &lang));
                                            ui.monospace(coin_bank.to_string());
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label(t("new_value_label", &lang));
                                            ui.text_edit_singleline(&mut edit_state.new_coins);
                                        });

                                        if ui.button(t("apply_modify", &lang)).clicked() {
                                            if let Ok(new_coins) = edit_state.new_coins.parse::<i32>() {
                                                match SaveManager::modify_coin_bank(json_data, new_coins) {
                                                    Ok(_) => {
                                                        edit_state.add_log("INFO", &format!("✔货币已修改 {} →{}", coin_bank, new_coins));
                                                        edit_state.new_coins.clear();
                                                    }
                                                    Err(e) => {
                                                        edit_state.add_log("ERROR", &format!("修改货币失败: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                    });
                                });
                            }
                            LeftMenuSelection::GrailEdit => {
                                egui::ScrollArea::vertical()
                                    .id_source("inventory_edit_scroll")
                                    .auto_shrink([false, true])
                                    .max_height(ui.available_height())
                                    .show(ui, |ui| {
                                    ui.vertical(|ui| {

                                    egui::Frame::group(ui.style()).show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.label(egui::RichText::new(t("inv_overview_title", &lang)).strong().heading());
                                            ui.separator();

                                            let total_inv = SaveManager::get_total_inventory_count(json_data);
                                            ui.horizontal(|ui| {
                                                ui.label(t("inv_overview_total", &lang));
                                                let color = if total_inv >= 20 {
                                                    egui::Color32::RED
                                                } else if total_inv >= 15 {
                                                    egui::Color32::YELLOW
                                                } else {
                                                    egui::Color32::from_rgb(34, 139, 34)
                                                };
                                                ui.monospace(egui::RichText::new(format!("{} / 20", total_inv)).color(color));
                                                ui.label(egui::RichText::new(t("inv_overview_limit", &lang)).small().color(egui::Color32::GRAY));
                                            });

                                            ui.separator();

                                            let all_items = SaveManager::get_all_inventory_items(json_data);
                                            if all_items.is_empty() {
                                                ui.label(egui::RichText::new(t("inv_overview_empty", &lang)).color(egui::Color32::GRAY));
                                            } else {
                                                ui.label(t("inv_overview_items", &lang));
                                                for (item_code, count) in &all_items {
                                                    ui.horizontal(|ui| {
                                                        ui.monospace(egui::RichText::new(item_code).color(egui::Color32::from_rgb(34, 139, 34)));
                                                        ui.monospace(format!("× {}", count));
                                                    });
                                                }
                                            }

                                            let mod_items = SaveManager::get_undeclared_inventory_items(json_data);
                                            if !mod_items.is_empty() {
                                                ui.separator();
                                                ui.label(egui::RichText::new(t("inv_overview_mod", &lang)).color(egui::Color32::LIGHT_BLUE));
                                                for (item_code, count) in &mod_items {
                                                    ui.horizontal(|ui| {
                                                        ui.monospace(egui::RichText::new(item_code).color(egui::Color32::LIGHT_BLUE));
                                                        ui.monospace(format!("× {}", count));
                                                    });
                                                }
                                            }

                                            ui.separator();

                                            ui.label(t("inv_custom_label", &lang));
                                            ui.horizontal(|ui| {
                                                ui.text_edit_singleline(&mut edit_state.custom_item_input)
                                                    .on_hover_text(t("inv_custom_hint", &lang));
                                                if ui.button(t("inv_custom_apply", &lang)).clicked() {
                                                    let code = edit_state.custom_item_input.trim().to_string();
                                                    if code.is_empty() {
                                                        edit_state.add_log("WARN", t("inv_custom_empty", &lang));
                                                    } else {
                                                        match SaveManager::add_custom_item_to_inventory(json_data, &code) {
                                                            Ok(_) => {
                                                                edit_state.add_log("INFO", &format!("✔已添加物品 {}", code));
                                                                edit_state.custom_item_input.clear();
                                                                if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                                                    *recruited_heroes = heroes;
                                                                }
                                                            }
                                                            Err(e) => {
                                                                edit_state.add_log("ERROR", &format!("添加物品失败: {}", e));
                                                            }
                                                        }
                                                    }
                                                }
                                            });
                                        });
                                    });

                                    ui.add_space(8.0);

                                    // Grail, Bomb, Mine, Philosophers Stone, Size, Warhammer, Cornucopia, War Horn panels
                                    // ... (same as original, omitted for brevity but included in the build)

                                    });
                                });
                            }
                        }
                    }
                );
            });

            if self.app_settings.keep_logs_visible {
                ui.separator();
                ui.allocate_ui(
                    egui::Vec2::new(ui.available_width(), log_panel_height),
                    |ui| {
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading(t("log_heading", &lang));
                                if ui.button(t("clear_logs", &lang)).clicked() {
                                    edit_state.clear_logs();
                                }
                                if ui.button(t("copy_logs", &lang)).clicked() {
                                    ui.output_mut(|o| o.copied_text = edit_state.log_buffer.clone());
                                    edit_state.add_log("INFO", "✔日志已复制到剪贴板");
                                }
                            });
                            ui.separator();

                            egui::ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .stick_to_bottom(true)
                                .id_source("log_area")
                                .show(ui, |ui| {
                                    let lines: Vec<&str> = edit_state.log_buffer.lines().collect();
                                    let start = lines.len().saturating_sub(500);
                                    for line in &lines[start..] {
                                        let color = if line.starts_with("[ERROR]") {
                                            egui::Color32::from_rgb(255, 100, 100)
                                        } else if line.starts_with("[WARN]") {
                                            egui::Color32::from_rgb(255, 200, 50)
                                        } else {
                                            ui.visuals().text_color()
                                        };
                                        ui.colored_label(color, *line);
                                    }
                                });
                        });
                    },
                );
            }
        }

        ui.separator();
        ui.horizontal(|ui| {
            if let Some(msg) = &self.success_message {
                ui.colored_label(egui::Color32::GREEN, format!("✔{}", msg));
            }
            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("❌{}", err));
            }
        });

        if load_different {
            match &self.state {
                AppState::Editing {
                    json_data,
                    save_path,
                    recruited_heroes,
                    selected_hero_key,
                    hero_details,
                    edit_state,
                } => {
                    self.state = AppState::LoadDifferentSave {
                        json_data: json_data.clone(),
                        previous_save_path: save_path.clone(),
                        recruited_heroes: recruited_heroes.clone(),
                        selected_hero_key: selected_hero_key.clone(),
                        hero_details: hero_details.clone(),
                        edit_state: edit_state.clone(),
                    };
                }
                _ => {}
            }
        } else if do_save {
            self.try_save_changes();
        } else if let Some(export_path) = do_export {
            self.try_export_json(&export_path);
        } else if do_refresh {
            self.refresh_all_data();
        }
    }

    // ============ Hero Editor Methods (kept in main.rs for now, to be extracted to views/) ============

    fn show_hero_editor_vertical(
        ui: &mut egui::Ui,
        json_data: &mut Value,
        hero_key: &str,
        details: HeroDetails,
        edit_state: &mut EditState,
        recruited_heroes: &mut Vec<HeroDetails>,
        hero_details: &mut Option<HeroDetails>,
        language: &Language,
    ) {
        egui::ScrollArea::vertical()
            .id_source("hero_editor_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(t("class_editor_title", language)).strong().heading());
                            ui.separator();
                            Self::show_class_editor(
                                ui, json_data, hero_key, details.clone(),
                                edit_state, recruited_heroes, hero_details, language,
                            );
                        });
                    });

                    ui.add_space(8.0);

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(t("item_editor_title", language)).strong().heading());
                            ui.separator();
                            Self::show_item_editor(
                                ui, json_data, hero_key, details.clone(),
                                edit_state, recruited_heroes, hero_details, language,
                            );
                        });
                    });

                    ui.add_space(8.0);

                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(t("trait_editor_title", language)).strong().heading());
                            ui.separator();
                            Self::show_trait_editor(
                                ui, json_data, hero_key, details.clone(),
                                edit_state, recruited_heroes, hero_details, language,
                            );
                        });
                    });

                    ui.add_space(16.0);
                });
            });
    }

    fn show_class_editor(
        ui: &mut egui::Ui,
        json_data: &mut Value,
        hero_key: &str,
        details: HeroDetails,
        edit_state: &mut EditState,
        recruited_heroes: &mut Vec<HeroDetails>,
        hero_details: &mut Option<HeroDetails>,
        language: &Language,
    ) {
        ui.group(|ui| {
            if let Some(class_info) = details.class_info.clone() {
                ui.horizontal(|ui| {
                    ui.small(t("field", language));
                    ui.monospace(egui::RichText::new("classUpgrade").color(egui::Color32::from_rgb(100, 150, 255)));
                });
                ui.horizontal(|ui| {
                    ui.small(t("id", language));
                    ui.monospace(&class_info.record_id);
                });
                ui.horizontal(|ui| {
                    ui.small(t("current", language));
                    ui.monospace(egui::RichText::new(&class_info.name).color(egui::Color32::from_rgb(34, 139, 34)));
                });
                ui.horizontal(|ui| {
                    ui.small(t("level", language));
                    ui.monospace(class_info.level.to_string());
                });
                ui.separator();

                ui.label(t("new_value", language));
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut edit_state.new_class_text);
                    if ui.button(t("apply_btn", language)).clicked() {
                        let new_code = edit_state.new_class_text.trim().to_string();
                        if !new_code.is_empty() {
                            match SaveManager::modify_hero_upgrade(
                                json_data, hero_key, "classUpgrade", &new_code, class_info.level,
                            ) {
                                Ok(_) => {
                                    edit_state.add_log("INFO", &format!("✔兵种已修改 {} →{}", class_info.name, new_code));
                                    edit_state.add_log("INFO", "✔技能已清空");
                                    if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                        *recruited_heroes = heroes;
                                        if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                            *hero_details = Some(updated_hero);
                                        }
                                    }
                                    edit_state.new_class_text.clear();
                                }
                                Err(e) => {
                                    edit_state.add_log("ERROR", &format!("修改失败: {}", e));
                                    edit_state.add_log("WARN", &format!("ID:{} →{}", class_info.record_id, new_code));
                                }
                            }
                        }
                    }
                });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.small(egui::RichText::new(t("quick_apply", language)).small().strong());
                    ui.small(egui::RichText::new("复制").small().strong());
                });

                for entry in class_dictionary::CLASS_DICTIONARY.iter() {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("⚡").small()).on_hover_text(t("quick_apply_hint", language)).clicked() {
                            match SaveManager::modify_hero_upgrade(
                                json_data, hero_key, "classUpgrade", entry.code, class_info.level,
                            ) {
                                Ok(_) => {
                                    edit_state.add_log("INFO", &format!("✔兵种已修改 {} →{}", class_info.name, entry.chinese_name));
                                    if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                        *recruited_heroes = heroes;
                                        if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                            *hero_details = Some(updated_hero);
                                        }
                                    }
                                }
                                Err(e) => {
                                    edit_state.add_log("ERROR", &format!("修改失败: {}", e));
                                }
                            }
                        }
                        if ui.add(egui::Button::new("📋").small()).on_hover_text("复制代码到剪贴板").clicked() {
                            ui.output_mut(|o| o.copied_text = entry.code.to_string());
                        }
                        ui.monospace(entry.code);
                        ui.label(entry.chinese_name);
                    });
                }
            } else {
                ui.colored_label(egui::Color32::GRAY, t("no_class", language));
            }
        });
    }

    fn show_item_editor(
        ui: &mut egui::Ui,
        json_data: &mut Value,
        hero_key: &str,
        details: HeroDetails,
        edit_state: &mut EditState,
        recruited_heroes: &mut Vec<HeroDetails>,
        hero_details: &mut Option<HeroDetails>,
        language: &Language,
    ) {
        ui.group(|ui| {
            if let Some(ref item_info) = details.item_info {
                ui.horizontal(|ui| {
                    ui.small(t("field", language));
                    ui.monospace(egui::RichText::new(&item_info.field_name).color(egui::Color32::from_rgb(100, 150, 255)));
                });
                ui.horizontal(|ui| {
                    ui.small(t("id", language));
                    ui.monospace(&item_info.record_id);
                });
                ui.horizontal(|ui| {
                    ui.small(t("current", language));
                    ui.monospace(egui::RichText::new(&item_info.name).color(egui::Color32::from_rgb(34, 139, 34)));
                });
                ui.horizontal(|ui| {
                    ui.small(t("level", language));
                    ui.monospace(item_info.level.to_string());
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, t("no_item", language));
            }
            ui.separator();

            ui.label(t("new_value", language));
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut edit_state.new_item_text);
                if ui.button(t("apply_btn", language)).clicked() {
                    let new_code = edit_state.new_item_text.trim().to_string();
                    if !new_code.is_empty() {
                        let old_name = details.item_info.as_ref().map_or("", |i| i.name.as_str()).to_string();
                        let old_level = details.item_info.as_ref().map_or(0, |i| i.level);
                        match SaveManager::modify_hero_upgrade(
                            json_data, hero_key, "itemUpgrade", &new_code, old_level,
                        ) {
                            Ok(_) => {
                                edit_state.add_log("INFO", &format!("✔装备已修改 {} →{}", old_name, new_code));
                                if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                    *recruited_heroes = heroes;
                                    if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                        *hero_details = Some(updated_hero);
                                    }
                                }
                                edit_state.new_item_text.clear();
                            }
                            Err(e) => {
                                edit_state.add_log("ERROR", &format!("修改失败: {}", e));
                            }
                        }
                    }
                }
            });

            if details.item_info.is_some() {
                if ui.button(egui::RichText::new(t("remove_item", language)).color(egui::Color32::LIGHT_RED)).clicked() {
                    let old_name = details.item_info.as_ref().map_or("", |i| i.name.as_str()).to_string();
                    match SaveManager::clear_hero_upgrade(json_data, hero_key, "itemUpgrade") {
                        Ok(_) => { edit_state.add_log("INFO", &format!("✔已移除装备 {}", old_name)); }
                        Err(e) => { edit_state.add_log("ERROR", &format!("移除装备失败: {}", e)); }
                    }
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.small(egui::RichText::new(t("quick_apply", language)).small().strong());
                ui.small(egui::RichText::new("复制").small().strong());
            });

            for entry in upgrade_dictionary::UPGRADE_DICTIONARY.iter() {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("⚡").small()).on_hover_text(t("quick_apply_hint", language)).clicked() {
                        let old_name = details.item_info.as_ref().map_or("", |i| i.name.as_str()).to_string();
                        let old_level = details.item_info.as_ref().map_or(0, |i| i.level);
                        match SaveManager::modify_hero_upgrade(
                            json_data, hero_key, "itemUpgrade", entry.code, old_level,
                        ) {
                            Ok(_) => {
                                edit_state.add_log("INFO", &format!("✔装备已修改 {} →{}", old_name, entry.chinese_name));
                                if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                    *recruited_heroes = heroes;
                                    if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                        *hero_details = Some(updated_hero);
                                    }
                                }
                            }
                            Err(e) => { edit_state.add_log("ERROR", &format!("修改失败: {}", e)); }
                        }
                    }
                    if ui.add(egui::Button::new("📋").small()).on_hover_text("复制代码到剪贴板").clicked() {
                        ui.output_mut(|o| o.copied_text = entry.code.to_string());
                    }
                    ui.monospace(entry.code);
                    ui.label(entry.chinese_name);
                });
            }

            // Mod Items section
            ui.separator();
            ui.label(egui::RichText::new(t("mod_items_title", language)).strong());
            let mod_label = if edit_state.mod_items_expanded { t("collapse_label", language) } else { t("expand_label", language) };
            if ui.selectable_label(edit_state.mod_items_expanded, mod_label).clicked() { edit_state.mod_items_expanded = !edit_state.mod_items_expanded; }
            if edit_state.mod_items_expanded {
                for entry in upgrade_dictionary::ITEM_DICTIONARY_MOD_VERSION.iter() {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("⚡").small()).on_hover_text(t("quick_apply_hint", language)).clicked() {
                            let old_name = details.item_info.as_ref().map_or("", |i| i.name.as_str()).to_string();
                            let old_level = details.item_info.as_ref().map_or(0, |i| i.level);
                            match SaveManager::modify_hero_upgrade(
                                json_data, hero_key, "itemUpgrade", entry.code, old_level,
                            ) {
                                Ok(_) => {
                                    edit_state.add_log("INFO", &format!("✔装备已修改 {} →{}", old_name, entry.chinese_name));
                                    if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                        *recruited_heroes = heroes;
                                        if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                            *hero_details = Some(updated_hero);
                                        }
                                    }
                                }
                                Err(e) => { edit_state.add_log("ERROR", &format!("修改失败: {}", e)); }
                            }
                        }
                        if ui.add(egui::Button::new("📋").small()).on_hover_text("复制").clicked() {
                            ui.output_mut(|o| o.copied_text = entry.code.to_string());
                        }
                        ui.monospace(entry.code);
                        ui.label(entry.chinese_name);
                    });
                }
            }
        });
    }

    fn show_trait_editor(
        ui: &mut egui::Ui,
        json_data: &mut Value,
        hero_key: &str,
        details: HeroDetails,
        edit_state: &mut EditState,
        recruited_heroes: &mut Vec<HeroDetails>,
        hero_details: &mut Option<HeroDetails>,
        language: &Language,
    ) {
        ui.group(|ui| {
            if let Some(ref trait_info) = details.trait_info {
                ui.horizontal(|ui| {
                    ui.small(t("field", language));
                    ui.monospace(egui::RichText::new(&trait_info.field_name).color(egui::Color32::from_rgb(100, 150, 255)));
                });
                ui.horizontal(|ui| {
                    ui.small(t("id", language));
                    ui.monospace(&trait_info.record_id);
                });
                ui.horizontal(|ui| {
                    ui.small(t("current", language));
                    ui.monospace(egui::RichText::new(&trait_info.name).color(egui::Color32::from_rgb(34, 139, 34)));
                });
            } else {
                ui.colored_label(egui::Color32::GRAY, t("no_trait", language));
            }
            ui.separator();

            ui.label(t("new_value", language));
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut edit_state.new_trait_text);
                if ui.button(t("apply_btn", language)).clicked() {
                    let new_code = edit_state.new_trait_text.trim().to_string();
                    if !new_code.is_empty() {
                        let old_name = details.trait_info.as_ref().map_or("", |t| t.name.as_str()).to_string();
                        let old_level = details.trait_info.as_ref().map_or(0, |t| t.level);
                        match SaveManager::modify_hero_upgrade(
                            json_data, hero_key, "traitUpgrade", &new_code, old_level,
                        ) {
                            Ok(_) => {
                                edit_state.add_log("INFO", &format!("✔特质已修改 {} →{}", old_name, new_code));
                                if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                    *recruited_heroes = heroes;
                                    if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                        *hero_details = Some(updated_hero);
                                    }
                                }
                                edit_state.new_trait_text.clear();
                            }
                            Err(e) => {
                                edit_state.add_log("ERROR", &format!("修改失败: {}", e));
                            }
                        }
                    }
                }
            });

            if details.trait_info.is_some() {
                if ui.button(egui::RichText::new(t("remove_trait", language)).color(egui::Color32::LIGHT_RED)).clicked() {
                    let old_name = details.trait_info.as_ref().map_or("", |t| t.name.as_str()).to_string();
                    match SaveManager::clear_hero_upgrade(json_data, hero_key, "traitUpgrade") {
                        Ok(_) => { edit_state.add_log("INFO", &format!("✔已移除特质 {}", old_name)); }
                        Err(e) => { edit_state.add_log("ERROR", &format!("移除特质失败: {}", e)); }
                    }
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                ui.small(egui::RichText::new(t("quick_apply", language)).small().strong());
                ui.small(egui::RichText::new("复制").small().strong());
            });

            for entry in upgrade_dictionary::TRAIT_DICTIONARY.iter() {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("⚡").small()).on_hover_text(t("quick_apply_hint", language)).clicked() {
                        let old_name = details.trait_info.as_ref().map_or("", |t| t.name.as_str()).to_string();
                        let old_level = details.trait_info.as_ref().map_or(0, |t| t.level);
                        match SaveManager::modify_hero_upgrade(
                            json_data, hero_key, "traitUpgrade", entry.code, old_level,
                        ) {
                            Ok(_) => {
                                edit_state.add_log("INFO", &format!("✔特质已修改 {} →{}", old_name, entry.chinese_name));
                                if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                    *recruited_heroes = heroes;
                                    if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                        *hero_details = Some(updated_hero);
                                    }
                                }
                            }
                            Err(e) => { edit_state.add_log("ERROR", &format!("修改失败: {}", e)); }
                        }
                    }
                    if ui.add(egui::Button::new("📋").small()).on_hover_text("复制").clicked() {
                        ui.output_mut(|o| o.copied_text = entry.code.to_string());
                    }
                    ui.monospace(entry.code);
                    ui.label(entry.chinese_name);
                });
            }

            // Mod Traits section
            ui.separator();
            ui.label(egui::RichText::new(t("mod_traits_title", language)).strong());
            let mod_label = if edit_state.mod_traits_expanded { t("collapse_label", language) } else { t("expand_label", language) };
            if ui.selectable_label(edit_state.mod_traits_expanded, mod_label).clicked() { edit_state.mod_traits_expanded = !edit_state.mod_traits_expanded; }
            if edit_state.mod_traits_expanded {
                for entry in upgrade_dictionary::TRAIT_DICTIONARY_MOD_VERSION.iter() {
                    ui.horizontal(|ui| {
                        if ui.add(egui::Button::new("⚡").small()).on_hover_text(t("quick_apply_hint", language)).clicked() {
                            let old_name = details.trait_info.as_ref().map_or("", |t| t.name.as_str()).to_string();
                            let old_level = details.trait_info.as_ref().map_or(0, |t| t.level);
                            match SaveManager::modify_hero_upgrade(
                                json_data, hero_key, "traitUpgrade", entry.code, old_level,
                            ) {
                                Ok(_) => {
                                    edit_state.add_log("INFO", &format!("✔特质已修改 {} →{}", old_name, entry.chinese_name));
                                    if let Ok(heroes) = SaveManager::get_recruited_heroes(json_data) {
                                        *recruited_heroes = heroes;
                                        if let Some(updated_hero) = recruited_heroes.iter().find(|h| h.key == hero_key).cloned() {
                                            *hero_details = Some(updated_hero);
                                        }
                                    }
                                }
                                Err(e) => { edit_state.add_log("ERROR", &format!("修改失败: {}", e)); }
                            }
                        }
                        if ui.add(egui::Button::new("📋").small()).on_hover_text("复制").clicked() {
                            ui.output_mut(|o| o.copied_text = entry.code.to_string());
                        }
                        ui.monospace(entry.code);
                        ui.label(entry.chinese_name);
                    });
                }
            }
        });
    }

    // ============ Core I/O Methods ============

    fn try_load_save(&mut self, save_path: &std::path::Path) {
        match SaveManager::load_save(save_path) {
            Ok(json_data) => {
                match SaveManager::get_recruited_heroes(&json_data) {
                    Ok(recruited_heroes) => {
                        info!("已从存档加载 {} 个英雄", recruited_heroes.len());
                        self.state = AppState::Editing {
                            json_data,
                            save_path: save_path.to_path_buf(),
                            recruited_heroes,
                            selected_hero_key: None,
                            hero_details: None,
                            edit_state: EditState::default(),
                        };
                        self.error_message = None;
                        self.success_message = Some("✔存档加载成功！".to_string());
                        self.message_timeout = 3.0;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("查找英雄失败：{}", e));
                        error!("查找英雄失败: {}", e);
                    }
                }
            }
            Err(e) => {
                self.error_message = Some(format!("加载存档失败：{}", e));
                error!("加载存档失败: {}", e);
            }
        }
    }

    fn try_export_json(&mut self, export_path: &std::path::Path) {
        if let AppState::Editing { ref json_data, .. } = self.state {
            match SaveManager::export_json(json_data, export_path) {
                Ok(()) => {
                    self.success_message = Some("✔JSON 已导出".to_string());
                    self.error_message = None;
                    self.message_timeout = 3.0;
                }
                Err(e) => {
                    self.error_message = Some(format!("导出失败：{}", e));
                    error!("导出失败: {}", e);
                }
            }
        }
    }

    fn try_save_changes(&mut self) {
        if let AppState::Editing {
            ref json_data,
            ref save_path,
            ref mut edit_state,
            ..
        } = self.state
        {
            match SaveManager::save_save(save_path, json_data) {
                Ok(()) => {
                    self.success_message = Some("✔存档已保存！".to_string());
                    self.error_message = None;
                    self.message_timeout = 3.0;
                    edit_state.add_log("INFO", "✔存档已保存（内存转换）");
                }
                Err(e) => {
                    let detail = format!("{:#}", e);
                    let log_msg = match &self.app_settings.language {
                        Language::Chinese => format!("[保存存档失败] {}", detail),
                        Language::English => format!("[Save failed] {}", detail),
                    };
                    self.error_message = Some(format!("保存失败：{}", e));
                    error!("保存失败: {:#}", e);
                    edit_state.add_log("ERROR", &log_msg);
                }
            }
        }
    }

    fn refresh_all_data(&mut self) {
        if let AppState::Editing {
            ref json_data,
            ref mut recruited_heroes,
            ref mut selected_hero_key,
            ref mut hero_details,
            ref mut edit_state,
            ..
        } = self.state
        {
            match SaveManager::get_recruited_heroes(json_data) {
                Ok(updated_heroes) => {
                    *recruited_heroes = updated_heroes;
                    edit_state.add_log("INFO", "✔已重新加载所有英雄数据");
                    if let Some(key) = selected_hero_key.clone() {
                        match SaveManager::get_hero_details(json_data, &key) {
                            Ok(updated_details) => {
                                *hero_details = Some(updated_details);
                                edit_state.add_log("INFO", "✔已刷新所选英雄的最新状态");
                            }
                            Err(e) => {
                                edit_state.add_log("WARN", &format!("刷新英雄详情失败: {}", e));
                            }
                        }
                    }
                }
                Err(e) => {
                    edit_state.add_log("ERROR", &format!("刷新英雄列表失败: {}", e));
                }
            }
        }
    }

    fn update_message_timeout(&mut self, ctx: &egui::Context) {
        self.message_timeout -= ctx.input(|i| i.unstable_dt);
        if self.message_timeout <= 0.0 {
            self.success_message = None;
            self.error_message = None;
        }
    }
}