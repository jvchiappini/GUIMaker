use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ferrous_engine::Color as AppColor;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub typography: ThemeTypography,
    pub spacing: ThemeSpacing,
    pub borders: ThemeBorders,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeColors {
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub background: String,
    pub surface: String,
    pub border: String,
    pub grid: String,
    pub top_bar: String,
    pub text: String,
    pub text_dim: String,
    pub error: String,
    pub success: String,
    pub warning: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeTypography {
    pub font_name: String,
    pub font_size_base: f32,
    pub font_size_small: f32,
    pub font_size_large: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeSpacing {
    pub padding: f32,
    pub margin: f32,
    pub gap: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeBorders {
    pub radius: f32,
    pub thickness: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors {
                primary: "#007ACC".to_string(),
                secondary: "#333333".to_string(),
                accent: "#569CD6".to_string(),
                background: "#1E1E1E".to_string(),
                surface: "#252526".to_string(),
                border: "#333333".to_string(),
                grid: "#2D2D2D".to_string(),
                top_bar: "#3C3C3C".to_string(),
                text: "#CCCCCC".to_string(),
                text_dim: "#888888".to_string(),
                error: "#F44336".to_string(),
                success: "#4CAF50".to_string(),
                warning: "#FFEB3B".to_string(),
            },
            typography: ThemeTypography {
                font_name: "Roboto".to_string(),
                font_size_base: 14.0,
                font_size_small: 12.0,
                font_size_large: 18.0,
            },
            spacing: ThemeSpacing {
                padding: 8.0,
                margin: 4.0,
                gap: 4.0,
            },
            borders: ThemeBorders {
                radius: 4.0,
                thickness: 1.0,
            },
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ThemeColors {
                primary: "#007ACC".to_string(),
                secondary: "#E1E1E1".to_string(),
                accent: "#005A9E".to_string(),
                background: "#F3F3F3".to_string(),
                surface: "#FFFFFF".to_string(),
                border: "#D1D1D1".to_string(),
                grid: "#E5E5E5".to_string(),
                top_bar: "#DDDDDD".to_string(),
                text: "#333333".to_string(),
                text_dim: "#666666".to_string(),
                error: "#D32F2F".to_string(),
                success: "#388E3C".to_string(),
                warning: "#FBC02D".to_string(),
            },
            typography: ThemeTypography {
                font_name: "Roboto".to_string(),
                font_size_base: 14.0,
                font_size_small: 12.0,
                font_size_large: 18.0,
            },
            spacing: ThemeSpacing {
                padding: 8.0,
                margin: 4.0,
                gap: 4.0,
            },
            borders: ThemeBorders {
                radius: 4.0,
                thickness: 1.0,
            },
        }
    }

    pub fn ocean() -> Self {
        Self {
            name: "Ocean".to_string(),
            colors: ThemeColors {
                primary: "#0D47A1".to_string(),
                secondary: "#1A237E".to_string(),
                accent: "#1976D2".to_string(),
                background: "#01579B".to_string(),
                surface: "#0277BD".to_string(),
                border: "#01579B".to_string(),
                grid: "#0288D1".to_string(),
                top_bar: "#01579B".to_string(),
                text: "#E1F5FE".to_string(),
                text_dim: "#B3E5FC".to_string(),
                error: "#FF1744".to_string(),
                success: "#00E676".to_string(),
                warning: "#FFEA00".to_string(),
            },
            typography: ThemeTypography {
                font_name: "Roboto".to_string(),
                font_size_base: 14.0,
                font_size_small: 12.0,
                font_size_large: 18.0,
            },
            spacing: ThemeSpacing {
                padding: 8.0,
                margin: 4.0,
                gap: 4.0,
            },
            borders: ThemeBorders {
                radius: 4.0,
                thickness: 1.0,
            },
        }
    }
}

pub struct ThemeRegistry {
    pub themes: HashMap<String, Theme>,
    pub current: String,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), Theme::dark());
        themes.insert("light".to_string(), Theme::light());
        themes.insert("ocean".to_string(), Theme::ocean());
        
        Self {
            themes,
            current: "dark".to_string(),
        }
    }

    pub fn get_current(&self) -> &Theme {
        self.themes.get(&self.current).unwrap_or_else(|| {
            self.themes.get("dark").unwrap()
        })
    }

    pub fn set_current(&mut self, name: &str) {
        if self.themes.contains_key(name) {
            self.current = name.to_string();
        }
    }

    pub fn hex_to_f32(hex: &str) -> [f32; 4] {
        AppColor::hex(hex).to_linear_f32()
    }
}
