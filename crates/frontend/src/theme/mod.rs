pub mod button;
pub mod container;
pub mod palette;
pub mod text;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

pub fn iced_theme(mode: ThemeMode) -> iced::Theme {
    iced::Theme::custom(
        match mode {
            ThemeMode::Light => "Pharmacy Light",
            ThemeMode::Dark => "Pharmacy Dark",
        },
        palette::palette(mode),
    )
}
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
}
pub mod radius {
    pub const SM: f32 = 6.0;
    pub const MD: f32 = 10.0;
    pub const LG: f32 = 14.0;
}
pub mod size {
    pub const CONTROL_HEIGHT: f32 = 42.0;
    pub const SIDEBAR_WIDTH: f32 = 228.0;
}
