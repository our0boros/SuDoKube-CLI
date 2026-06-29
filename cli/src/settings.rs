//! 设置相关类型：AppSettings, SettingsState, SettingsField

use crate::save;
use crate::types::{KeymapEditState, SettingsArrow};

// ── AppSettings ──

/// 可配置项
#[derive(Debug, Clone)]
pub struct AppSettings {
    pub standard_cell_width: usize, // 奇数
    pub bg_color: String,           // "black", "darkgray"
    pub border_color: String,       // "cyan", "white", "green"
    pub guide_group_color: String,  // 同宫/同行/同列高亮背景色 "green","blue","magenta"
    pub guide_group_fg: String,     // 同宫/同行/同列高亮字体色 "white","black"
    pub guide_same_color: String,   // 同数字高亮背景色 "blue","magenta","red"
    pub guide_same_fg: String,      // 同数字高亮字体色 "white","black"
    pub cube_scale: String,         // "0.3", "0.35", "0.4", "0.45", "0.5"
    pub show_cube: String,          // "yes", "no"
    pub cube_width: String,         // "16", "18", "20", "22", "24"
    pub cube_height: String,        // "14", "16", "18", "20", "22"
    pub cube_aspect: String,        // "0.8", "1.0", "1.2", "1.5"
    pub debug_mode: String,         // "off", "on"
    pub language: String,           // "zh", "en", "ja"
    pub naming_mode: String,        // "vivid", "numeric"
    pub blink_highlight: String,    // "off", "on"
    pub user_value_color: String,   // "white", "cyan", "magenta", "blue", "green", "gray"
    pub error_value_color: String,  // "red", "yellow", "magenta"
    pub error_bold: String,         // "off", "on"
    pub guide_owned: String,        // "off"=Lock(未购), "on"=已购
    pub guide_enabled: String,      // "off"=Disable, "on"=Enable;仅在 guide_owned="on" 时可改
}

impl Default for AppSettings {
    fn default() -> Self {
        let lang = detect_language();
        Self {
            standard_cell_width: 7,
            bg_color: "black".into(),
            border_color: "cyan".into(),
            guide_group_color: "green".into(),
            guide_group_fg: "white".into(),
            guide_same_color: "blue".into(),
            guide_same_fg: "white".into(),
            cube_scale: "0.38".into(),
            show_cube: "yes".into(),
            cube_width: "20".into(),
            cube_height: "18".into(),
            cube_aspect: "1.0".into(),
            debug_mode: "off".into(),
            language: lang,
            naming_mode: "vivid".into(),
            blink_highlight: "off".into(),
            user_value_color: "white".into(),
            error_value_color: "red".into(),
            error_bold: "on".into(),
            guide_owned: "off".into(),
            guide_enabled: "off".into(),
        }
    }
}

impl AppSettings {
    pub fn load_from_db() -> Self {
        let def = Self::default();
        Self {
            standard_cell_width: save::load_setting("standard_cell_width")
                .ok()
                .flatten()
                .and_then(|v| v.parse().ok())
                .unwrap_or(def.standard_cell_width),
            bg_color: save::load_setting("bg_color")
                .ok()
                .flatten()
                .unwrap_or(def.bg_color),
            border_color: save::load_setting("border_color")
                .ok()
                .flatten()
                .unwrap_or(def.border_color),
            guide_group_color: save::load_setting("guide_group_color")
                .ok()
                .flatten()
                .unwrap_or(def.guide_group_color),
            guide_group_fg: save::load_setting("guide_group_fg")
                .ok()
                .flatten()
                .unwrap_or(def.guide_group_fg),
            guide_same_color: save::load_setting("guide_same_color")
                .ok()
                .flatten()
                .unwrap_or(def.guide_same_color),
            guide_same_fg: save::load_setting("guide_same_fg")
                .ok()
                .flatten()
                .unwrap_or(def.guide_same_fg),
            cube_scale: save::load_setting("cube_scale")
                .ok()
                .flatten()
                .unwrap_or(def.cube_scale),
            show_cube: save::load_setting("show_cube")
                .ok()
                .flatten()
                .unwrap_or(def.show_cube),
            cube_width: save::load_setting("cube_width")
                .ok()
                .flatten()
                .unwrap_or(def.cube_width),
            cube_height: save::load_setting("cube_height")
                .ok()
                .flatten()
                .unwrap_or(def.cube_height),
            cube_aspect: save::load_setting("cube_aspect")
                .ok()
                .flatten()
                .unwrap_or(def.cube_aspect),
            debug_mode: save::load_setting("debug_mode")
                .ok()
                .flatten()
                .unwrap_or(def.debug_mode),
            language: save::load_setting("language")
                .ok()
                .flatten()
                .unwrap_or(def.language),
            naming_mode: save::load_setting("naming_mode")
                .ok()
                .flatten()
                .unwrap_or(def.naming_mode),
            blink_highlight: save::load_setting("blink_highlight")
                .ok()
                .flatten()
                .unwrap_or(def.blink_highlight),
            user_value_color: save::load_setting("user_value_color")
                .ok()
                .flatten()
                .unwrap_or(def.user_value_color),
            error_value_color: save::load_setting("error_value_color")
                .ok()
                .flatten()
                .unwrap_or(def.error_value_color),
            error_bold: save::load_setting("error_bold")
                .ok()
                .flatten()
                .unwrap_or(def.error_bold),
            guide_owned: save::load_setting("guide_owned")
                .ok()
                .flatten()
                .unwrap_or(def.guide_owned),
            guide_enabled: save::load_setting("guide_enabled")
                .ok()
                .flatten()
                .unwrap_or(def.guide_enabled),
        }
    }

    pub fn save_to_db(&self) {
        let _ = save::save_setting("standard_cell_width", &self.standard_cell_width.to_string());
        let _ = save::save_setting("bg_color", &self.bg_color);
        let _ = save::save_setting("border_color", &self.border_color);
        let _ = save::save_setting("guide_group_color", &self.guide_group_color);
        let _ = save::save_setting("guide_group_fg", &self.guide_group_fg);
        let _ = save::save_setting("guide_same_color", &self.guide_same_color);
        let _ = save::save_setting("guide_same_fg", &self.guide_same_fg);
        let _ = save::save_setting("cube_scale", &self.cube_scale);
        let _ = save::save_setting("show_cube", &self.show_cube);
        let _ = save::save_setting("cube_width", &self.cube_width);
        let _ = save::save_setting("cube_height", &self.cube_height);
        let _ = save::save_setting("cube_aspect", &self.cube_aspect);
        let _ = save::save_setting("debug_mode", &self.debug_mode);
        let _ = save::save_setting("language", &self.language);
        let _ = save::save_setting("naming_mode", &self.naming_mode);
        let _ = save::save_setting("blink_highlight", &self.blink_highlight);
        let _ = save::save_setting("user_value_color", &self.user_value_color);
        let _ = save::save_setting("error_value_color", &self.error_value_color);
        let _ = save::save_setting("error_bold", &self.error_bold);
        let _ = save::save_setting("guide_owned", &self.guide_owned);
        let _ = save::save_setting("guide_enabled", &self.guide_enabled);
    }
}

/// Guide 功能状态。共三种:
/// - `Locked`   未购买(新存档默认),商店提供 Guide 商品(100 金币)
/// - `Disabled` 已购但用户关闭
/// - `Enabled`  已购并启用,棋盘高亮同宫/同数
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuideState {
    Locked,
    Disabled,
    Enabled,
}

impl AppSettings {
    pub fn guide_state(&self) -> GuideState {
        if self.guide_owned != "on" {
            GuideState::Locked
        } else if self.guide_enabled == "on" {
            GuideState::Enabled
        } else {
            GuideState::Disabled
        }
    }

    pub fn guide_active(&self) -> bool {
        self.guide_state() == GuideState::Enabled
    }
}

fn detect_language() -> String {
    let tz = std::env::var("TZ").unwrap_or_default();
    let lang = std::env::var("LANG").unwrap_or_default();
    let locale =
        std::env::var("LC_ALL").unwrap_or_else(|_| std::env::var("LC_CTYPE").unwrap_or_default());

    let check = format!("{}{}{}", tz, lang, locale).to_lowercase();
    if check.contains("cn") || check.contains("zh") {
        "zh".into()
    } else if check.contains("jp") || check.contains("ja") {
        "ja".into()
    } else {
        "en".into()
    }
}

// ── SettingsState ──

pub struct SettingsState {
    pub fields: Vec<SettingsField>,
    pub selected: usize,
    /// 弹窗中可见区域的滚动偏移（用于内容过多时滚动）
    pub scroll: u16,
    /// 鼠标悬停的字段下标
    pub hover_field: Option<usize>,
    /// 鼠标悬停的方向（None / Left / Right）
    pub hover_arrow: Option<SettingsArrow>,
    /// 弹窗是否可见（默认隐藏,需用户主动打开）
    pub visible: bool,
    /// 键位映射编辑模式
    pub keymap_edit: Option<KeymapEditState>,
    /// 键位配置错误消息（红色显示）
    pub keymap_error: Option<String>,
    /// 上次的 debug_mode 状态（用于检测变化）
    pub keymap_debug_mode: String,
}

impl SettingsState {
    pub fn from_settings(s: &AppSettings) -> Self {
        let widths: Vec<String> = (3..=15).step_by(2).map(|n| n.to_string()).collect();
        let colors = vec!["black".into(), "darkgray".into()];
        let border_colors = vec![
            "cyan".into(),
            "white".into(),
            "green".into(),
            "yellow".into(),
        ];
        let guide_colors = vec![
            "green".into(),
            "blue".into(),
            "magenta".into(),
            "red".into(),
        ];
        let cube_scales = vec![
            "0.3".into(),
            "0.35".into(),
            "0.38".into(),
            "0.4".into(),
            "0.45".into(),
            "0.5".into(),
        ];
        let yes_no = vec!["yes".into(), "no".into()];
        let cube_widths = vec![
            "16".into(),
            "18".into(),
            "20".into(),
            "22".into(),
            "24".into(),
            "26".into(),
            "28".into(),
            "30".into(),
        ];
        let cube_heights = vec![
            "14".into(),
            "16".into(),
            "18".into(),
            "20".into(),
            "22".into(),
            "24".into(),
            "26".into(),
            "28".into(),
            "30".into(),
        ];
        let cube_aspects = vec![
            "0.7".into(),
            "0.85".into(),
            "1.0".into(),
            "1.2".into(),
            "1.4".into(),
            "1.6".into(),
        ];
        let debug_modes = vec!["off".into(), "on".into()];
        let languages = vec!["zh".into(), "en".into(), "ja".into()];
        let naming_modes = vec!["vivid".into(), "numeric".into()];
        let blink_modes = vec!["off".into(), "on".into()];
        let user_value_colors = vec![
            "white".into(),
            "cyan".into(),
            "magenta".into(),
            "blue".into(),
            "green".into(),
            "yellow".into(),
            "gray".into(),
        ];
        let error_value_colors = vec![
            "red".into(),
            "yellow".into(),
            "magenta".into(),
        ];
        let bold_modes = vec!["off".into(), "on".into()];
        let fg_colors = vec![
            "white".into(),
            "black".into(),
            "gray".into(),
            "cyan".into(),
            "green".into(),
            "yellow".into(),
            "magenta".into(),
            "red".into(),
            "blue".into(),
        ];

        let fields = vec![
            SettingsField::new("Cell Width", &s.standard_cell_width.to_string(), widths),
            SettingsField::new("BG Color", &s.bg_color, colors),
            SettingsField::new("Border Color", &s.border_color, border_colors.clone()),
            SettingsField::new("Guide-Group BG", &s.guide_group_color, guide_colors.clone()),
            SettingsField::new("Guide-Group FG", &s.guide_group_fg, fg_colors.clone()),
            SettingsField::new("Guide-Same BG", &s.guide_same_color, guide_colors),
            SettingsField::new("Guide-Same FG", &s.guide_same_fg, fg_colors),
            SettingsField::new("Input Color", &s.user_value_color, user_value_colors),
            SettingsField::new("Error Color", &s.error_value_color, error_value_colors),
            SettingsField::new("Error Bold", &s.error_bold, bold_modes),
            SettingsField::new("Guide", &s.guide_enabled, vec!["off".into(), "on".into()]),
            SettingsField::new("Cube Scale", &s.cube_scale, cube_scales),
            SettingsField::new("Show Cube", &s.show_cube, yes_no),
            SettingsField::new("Cube Width", &s.cube_width, cube_widths),
            SettingsField::new("Cube Height", &s.cube_height, cube_heights),
            SettingsField::new("Cube Aspect", &s.cube_aspect, cube_aspects),
            SettingsField::new("Debug Mode", &s.debug_mode, debug_modes),
            SettingsField::new("Language", &s.language, languages),
            SettingsField::new("Naming Mode", &s.naming_mode, naming_modes),
            SettingsField::new("Blink Highlight", &s.blink_highlight, blink_modes),
            SettingsField::new("Keymap", "Configure...", vec!["Configure...".into()]),
        ];
        Self {
            fields,
            selected: 0,
            scroll: 0,
            hover_field: None,
            hover_arrow: None,
            visible: false,
            keymap_edit: None,
            keymap_error: None,
            keymap_debug_mode: s.debug_mode.clone(),
        }
    }

    pub fn apply_to(&self, s: &mut AppSettings) {
        s.standard_cell_width = self.fields[0].value.parse().unwrap_or(7);
        s.bg_color = self.fields[1].value.clone();
        s.border_color = self.fields[2].value.clone();
        s.guide_group_color = self.fields[3].value.clone();
        s.guide_group_fg = self.fields[4].value.clone();
        s.guide_same_color = self.fields[5].value.clone();
        s.guide_same_fg = self.fields[6].value.clone();
        s.user_value_color = self.fields[7].value.clone();
        s.error_value_color = self.fields[8].value.clone();
        s.error_bold = self.fields[9].value.clone();
        // Guide 字段:仅在已购买时允许修改 enable;否则强制保持 off(Lock 状态)
        if s.guide_owned == "on" {
            s.guide_enabled = self.fields[10].value.clone();
        } else {
            s.guide_enabled = "off".into();
        }
        s.cube_scale = self.fields[11].value.clone();
        s.show_cube = self.fields[12].value.clone();
        s.cube_width = self.fields[13].value.clone();
        s.cube_height = self.fields[14].value.clone();
        s.cube_aspect = self.fields[15].value.clone();
        s.debug_mode = self.fields[16].value.clone();
        s.language = self.fields[17].value.clone();
        s.naming_mode = self.fields[18].value.clone();
        s.blink_highlight = self.fields[19].value.clone();
        // fields[20] = "Keymap" — 不写入 AppSettings
    }
}

// ── SettingsField ──

#[derive(Debug, Clone)]
pub struct SettingsField {
    pub label: String,
    pub value: String,
    pub options: Vec<String>,
    pub option_index: usize,
}

impl SettingsField {
    pub fn new(label: &str, current: &str, options: Vec<String>) -> Self {
        let idx = options.iter().position(|o| o == current).unwrap_or(0);
        Self {
            label: label.into(),
            value: current.into(),
            options,
            option_index: idx,
        }
    }

    pub fn cycle_next(&mut self) {
        self.option_index = (self.option_index + 1) % self.options.len();
        self.value = self.options[self.option_index].clone();
    }

    pub fn cycle_prev(&mut self) {
        if self.option_index == 0 {
            self.option_index = self.options.len() - 1;
        } else {
            self.option_index -= 1;
        }
        self.value = self.options[self.option_index].clone();
    }
}
