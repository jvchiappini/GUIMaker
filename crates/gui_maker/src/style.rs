//! Design tokens. Every colour is stored in linear space after a one-time
//! sRGB conversion, matching the engine's colour contract.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self::linear(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::linear(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::linear(1.0, 1.0, 1.0, 1.0);

    pub const fn linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_srgb_hex(hex: &str) -> Self {
        let t = hex.trim().trim_start_matches('#');
        let expanded: String = if t.len() == 3 {
            t.chars().flat_map(|c| [c, c]).collect()
        } else {
            t.to_string()
        };
        let value = u32::from_str_radix(&expanded, 16).unwrap_or(0);
        let channel = |shift: u32| {
            srgb_to_linear(((value >> shift) & 0xFF) as f32 / 255.0)
        };
        Self::linear(channel(16), channel(8), channel(0), 1.0)
    }

    pub fn from_srgb(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::linear(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a)
    }

    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Self::linear(value[0], value[1], value[2], value[3])
    }
}

impl From<Color> for [f32; 4] {
    fn from(value: Color) -> Self {
        value.to_array()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub bg: Color,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub focus_ring: Color,
    pub selection: Color,
}

#[derive(Clone, Copy, Debug)]
pub struct Spacing {
    pub xxs: f32,
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Radius {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct TypeScale {
    pub caption: f32,
    pub body: f32,
    pub label: f32,
    pub title: f32,
    pub mono: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlHeights {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub palette: Palette,
    pub spacing: Spacing,
    pub radius: Radius,
    pub type_scale: TypeScale,
    pub control: ControlHeights,
}

impl Default for Style {
    fn default() -> Self {
        Self::koncreta_dark()
    }
}

impl Style {
    /// Tokens mirroring the current `core_viewer::theme::Theme` values so the
    /// first migrations keep the existing look.
    pub fn koncreta_dark() -> Self {
        Self {
            palette: Palette {
                bg: Color::from_srgb_hex("#111318"),
                panel_bg: Color::linear(0.06, 0.06, 0.06, 0.98),
                panel_border: Color::linear(1.0, 1.0, 1.0, 0.07),
                surface: Color::from_srgb_hex("#2a2a2a"),
                surface_raised: Color::from_srgb_hex("#333333"),
                surface_hover: Color::from_srgb_hex("#3d3d3d"),
                surface_active: Color::from_srgb_hex("#454545"),
                border: Color::linear(1.0, 1.0, 1.0, 0.08),
                border_strong: Color::linear(1.0, 1.0, 1.0, 0.16),
                text: Color::linear(1.0, 1.0, 1.0, 1.0),
                text_dim: Color::from_srgb_hex("#a3a3a3"),
                text_muted: Color::from_srgb_hex("#525252"),
                accent: Color::from_srgb_hex("#dc2626"),
                accent_hover: Color::from_srgb_hex("#ef4444"),
                info: Color::from_srgb_hex("#4a7fa5"),
                success: Color::from_srgb_hex("#22c55e"),
                warning: Color::from_srgb_hex("#f59e0b"),
                danger: Color::from_srgb_hex("#ef4444"),
                focus_ring: Color::from_srgb_hex("#4a7fa5"),
                selection: Color::linear(0.86, 0.15, 0.15, 0.12),
            },
            spacing: Spacing { xxs: 2.0, xs: 4.0, sm: 6.0, md: 8.0, lg: 12.0, xl: 16.0, xxl: 24.0 },
            radius: Radius { sm: 4.0, md: 6.0, lg: 8.0, xl: 12.0 },
            type_scale: TypeScale { caption: 10.0, body: 11.0, label: 12.0, title: 14.0, mono: 11.0 },
            control: ControlHeights { sm: 24.0, md: 28.0, lg: 34.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_is_converted_to_linear() {
        let black = Color::from_srgb_hex("#000000");
        assert_eq!(black.to_array(), [0.0, 0.0, 0.0, 1.0]);
        let white = Color::from_srgb_hex("#ffffff");
        assert!((white.r - 1.0).abs() < 1e-6);
        assert!((white.g - 1.0).abs() < 1e-6);
        assert!((white.b - 1.0).abs() < 1e-6);
    }

    #[test]
    fn accent_matches_the_koncreta_red() {
        let accent = Style::koncreta_dark().palette.accent;
        assert!((accent.r - 0.7157).abs() < 1e-3);
        assert!((accent.g - 0.0194).abs() < 1e-3);
        assert!((accent.b - 0.0194).abs() < 1e-3);
    }
}
