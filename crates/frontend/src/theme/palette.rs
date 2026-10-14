use super::ThemeMode;
use iced::{Color, Theme, theme::Palette};

/// The colours iced derives its built-in widget styles from.
pub fn palette(mode: ThemeMode) -> Palette {
    let t = Tokens::for_mode(mode);
    Palette {
        background: t.canvas,
        text: t.text,
        primary: t.primary,
        success: t.success,
        warning: t.warning,
        danger: t.danger,
    }
}

/// Design tokens: every colour the app's own styles use.
#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    /// The content panel behind cards.
    pub canvas: Color,
    /// Cards, inputs and table bodies.
    pub surface: Color,
    /// Tiles, table headers, hovered rows, secondary fills.
    pub surface_alt: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_muted: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_soft: Color,
    pub on_primary: Color,
    /// Warm gold highlight, used sparingly (charts, secondary call to action).
    pub accent: Color,
    pub accent_soft: Color,
    pub success: Color,
    pub success_soft: Color,
    pub warning: Color,
    pub warning_soft: Color,
    pub danger: Color,
    pub danger_soft: Color,
    pub info: Color,
    pub info_soft: Color,
    /// The coloured frame around the content panel.
    pub frame: Color,
    pub sidebar: Color,
    pub sidebar_hover: Color,
    pub sidebar_active: Color,
    pub sidebar_text: Color,
    pub sidebar_text_muted: Color,
    pub shadow: Color,
}

const fn rgb(hex: u32) -> Color {
    Color::from_rgb8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const fn rgba(hex: u32, alpha: f32) -> Color {
    Color::from_rgba8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8, alpha)
}

const LIGHT: Tokens = Tokens {
    canvas: rgb(0xFFFFFF),
    surface: rgb(0xFFFFFF),
    surface_alt: rgb(0xFBF7EF),
    border: rgb(0xEFE8DA),
    border_strong: rgb(0xE0D6C2),
    text: rgb(0x1F2A26),
    text_muted: rgb(0x8B8F86),
    primary: rgb(0x2F8A63),
    primary_hover: rgb(0x26724F),
    primary_soft: rgb(0xE4F2EA),
    on_primary: rgb(0xFFFFFF),
    accent: rgb(0xD8BB7E),
    accent_soft: rgb(0xF7EFDC),
    success: rgb(0x2F8A63),
    success_soft: rgb(0xE4F2EA),
    warning: rgb(0xC08A2E),
    warning_soft: rgb(0xFBF0DA),
    danger: rgb(0xD9534F),
    danger_soft: rgb(0xFCEBEA),
    info: rgb(0x3B9CD9),
    info_soft: rgb(0xE6F4FC),
    frame: rgb(0x2F8A63),
    sidebar: rgb(0x2F8A63),
    sidebar_hover: rgba(0xFFFFFF, 0.10),
    sidebar_active: rgb(0xFFFFFF),
    sidebar_text: rgb(0xFFFFFF),
    sidebar_text_muted: rgb(0xD2EADF),
    shadow: rgba(0x7A5A28, 0.08),
};

const DARK: Tokens = Tokens {
    canvas: rgb(0x111A16),
    surface: rgb(0x16211C),
    surface_alt: rgb(0x1B2822),
    border: rgb(0x26352E),
    border_strong: rgb(0x34463D),
    text: rgb(0xE8EEEA),
    text_muted: rgb(0x93A39B),
    primary: rgb(0x4CC08E),
    primary_hover: rgb(0x6FD3A6),
    primary_soft: rgba(0x4CC08E, 0.14),
    on_primary: rgb(0x08221A),
    accent: rgb(0xD8BB7E),
    accent_soft: rgba(0xD8BB7E, 0.14),
    success: rgb(0x4CC08E),
    success_soft: rgba(0x4CC08E, 0.14),
    warning: rgb(0xE5B45A),
    warning_soft: rgba(0xE5B45A, 0.14),
    danger: rgb(0xF07C78),
    danger_soft: rgba(0xF07C78, 0.14),
    info: rgb(0x7DC2EE),
    info_soft: rgba(0x7DC2EE, 0.14),
    frame: rgb(0x0B1310),
    sidebar: rgb(0x0B1310),
    sidebar_hover: rgba(0xFFFFFF, 0.06),
    sidebar_active: rgb(0x111A16),
    sidebar_text: rgb(0xE8EEEA),
    sidebar_text_muted: rgb(0x8FA59A),
    shadow: rgba(0x000000, 0.35),
};

impl Tokens {
    pub const fn for_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => LIGHT,
            ThemeMode::Dark => DARK,
        }
    }
}

pub fn tokens(theme: &Theme) -> Tokens {
    if theme.extended_palette().is_dark {
        DARK
    } else {
        LIGHT
    }
}

/// A semantic colour for badges, banners and icon chips.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Accent,
}

impl Tone {
    /// Foreground and soft background colours.
    pub fn colors(self, t: &Tokens) -> (Color, Color) {
        match self {
            Tone::Neutral => (t.text_muted, t.surface_alt),
            Tone::Primary => (t.primary, t.primary_soft),
            Tone::Success => (t.success, t.success_soft),
            Tone::Warning => (t.warning, t.warning_soft),
            Tone::Danger => (t.danger, t.danger_soft),
            Tone::Info => (t.info, t.info_soft),
            Tone::Accent => (t.accent, t.accent_soft),
        }
    }
}
