use std::fs;
use std::path::{Path, PathBuf};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeName {
    Dayroll,
    Nord,
    Gruvbox,
    TokyoNight,
}

impl ThemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeName::Dayroll => "dayroll",
            ThemeName::Nord => "nord",
            ThemeName::Gruvbox => "gruvbox",
            ThemeName::TokyoNight => "tokyo-night",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ThemeName::Dayroll => ThemeName::Nord,
            ThemeName::Nord => ThemeName::Gruvbox,
            ThemeName::Gruvbox => ThemeName::TokyoNight,
            ThemeName::TokyoNight => ThemeName::Dayroll,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            ThemeName::Dayroll => ThemeName::TokyoNight,
            ThemeName::Nord => ThemeName::Dayroll,
            ThemeName::Gruvbox => ThemeName::Nord,
            ThemeName::TokyoNight => ThemeName::Gruvbox,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Color,
    pub panel: Color,
    pub bar: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub info: Color,
    pub ok: Color,
    pub warn: Color,
    pub danger: Color,
}

pub fn theme_by_name(name: ThemeName) -> Theme {
    match name {
        ThemeName::Dayroll => Theme {
            bg: Color::Rgb(26, 31, 38),
            panel: Color::Rgb(34, 41, 50),
            bar: Color::Rgb(30, 36, 44),
            border: Color::Rgb(95, 110, 126),
            text: Color::Rgb(232, 238, 244),
            muted: Color::Rgb(176, 187, 198),
            accent: Color::Rgb(233, 165, 89),
            info: Color::Rgb(142, 177, 222),
            ok: Color::Rgb(132, 225, 164),
            warn: Color::Rgb(242, 197, 107),
            danger: Color::Rgb(236, 120, 92),
        },
        ThemeName::Nord => Theme {
            bg: Color::Rgb(46, 52, 64),
            panel: Color::Rgb(59, 66, 82),
            bar: Color::Rgb(67, 76, 94),
            border: Color::Rgb(129, 161, 193),
            text: Color::Rgb(236, 239, 244),
            muted: Color::Rgb(180, 188, 204),
            accent: Color::Rgb(136, 192, 208),
            info: Color::Rgb(143, 188, 187),
            ok: Color::Rgb(163, 190, 140),
            warn: Color::Rgb(235, 203, 139),
            danger: Color::Rgb(191, 97, 106),
        },
        ThemeName::Gruvbox => Theme {
            bg: Color::Rgb(40, 40, 40),
            panel: Color::Rgb(50, 48, 47),
            bar: Color::Rgb(60, 56, 54),
            border: Color::Rgb(146, 131, 116),
            text: Color::Rgb(235, 219, 178),
            muted: Color::Rgb(168, 153, 132),
            accent: Color::Rgb(250, 189, 47),
            info: Color::Rgb(131, 165, 152),
            ok: Color::Rgb(184, 187, 38),
            warn: Color::Rgb(250, 189, 47),
            danger: Color::Rgb(251, 73, 52),
        },
        ThemeName::TokyoNight => Theme {
            bg: Color::Rgb(26, 27, 38),
            panel: Color::Rgb(36, 40, 59),
            bar: Color::Rgb(41, 46, 66),
            border: Color::Rgb(122, 162, 247),
            text: Color::Rgb(192, 202, 245),
            muted: Color::Rgb(161, 168, 204),
            accent: Color::Rgb(187, 154, 247),
            info: Color::Rgb(125, 207, 255),
            ok: Color::Rgb(158, 206, 106),
            warn: Color::Rgb(224, 175, 104),
            danger: Color::Rgb(247, 118, 142),
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeName,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeName::Dayroll,
        }
    }
}

pub fn load_config() -> Result<AppConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("failed reading config {}: {error}", path.display()))?;
    toml::from_str::<AppConfig>(&raw)
        .map_err(|error| format!("failed parsing config {}: {error}", path.display()))
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed creating config dir {}: {error}", parent.display()))?;
    }
    let encoded = toml::to_string(config)
        .map_err(|error| format!("failed encoding config {}: {error}", path.display()))?;
    fs::write(&path, encoded)
        .map_err(|error| format!("failed writing config {}: {error}", path.display()))
}

pub fn config_path() -> PathBuf {
    match std::env::var("HOME") {
        Ok(home) => Path::new(&home)
            .join(".config")
            .join("dayroll")
            .join("config.toml"),
        Err(_) => Path::new(".")
            .join(".config")
            .join("dayroll")
            .join("config.toml"),
    }
}
