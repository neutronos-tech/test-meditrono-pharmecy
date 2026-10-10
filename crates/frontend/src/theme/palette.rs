use super::ThemeMode;
use iced::{Color, Theme, theme::Palette};

pub fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Light => Palette {
            background: Color::from_rgb8(245, 247, 246),
            text: Color::from_rgb8(29, 41, 37),
            primary: Color::from_rgb8(15, 118, 110),
            success: Color::from_rgb8(22, 134, 90),
            warning: Color::from_rgb8(196, 119, 16),
            danger: Color::from_rgb8(196, 61, 61),
        },
        ThemeMode::Dark => Palette {
            background: Color::from_rgb8(17, 24, 22),
            text: Color::from_rgb8(230, 236, 233),
            primary: Color::from_rgb8(66, 181, 166),
            success: Color::from_rgb8(74, 197, 139),
            warning: Color::from_rgb8(232, 164, 74),
            danger: Color::from_rgb8(224, 107, 107),
        },
    }
}
pub fn surface(theme: &Theme) -> Color {
    theme.extended_palette().background.weak.color
}
pub fn muted(theme: &Theme) -> Color {
    theme.extended_palette().background.strong.text
}
pub fn border(theme: &Theme) -> Color {
    theme.extended_palette().background.strong.color
}
