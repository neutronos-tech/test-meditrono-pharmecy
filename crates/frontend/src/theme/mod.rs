pub mod button;
pub mod container;
pub mod palette;
pub mod text;

pub use palette::Tone;

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

/// Inter, bundled in `assets/inter`, in the weights the app uses.
pub mod font {
    use iced::{Font, font::Weight};

    pub const REGULAR: Font = Font::with_name("Inter");
    pub const MEDIUM: Font = Font {
        weight: Weight::Medium,
        ..REGULAR
    };
    pub const SEMIBOLD: Font = Font {
        weight: Weight::Semibold,
        ..REGULAR
    };
    pub const BOLD: Font = Font {
        weight: Weight::Bold,
        ..REGULAR
    };

    pub const FILES: [&[u8]; 4] = [
        include_bytes!("../../assets/inter/Inter-Regular.ttf"),
        include_bytes!("../../assets/inter/Inter-Medium.ttf"),
        include_bytes!("../../assets/inter/Inter-SemiBold.ttf"),
        include_bytes!("../../assets/inter/Inter-Bold.ttf"),
    ];
}

/// Text sizes, largest first.
pub mod typography {
    pub const DISPLAY: f32 = 32.0;
    pub const H1: f32 = 26.0;
    pub const H2: f32 = 18.0;
    pub const H3: f32 = 15.0;
    pub const BODY: f32 = 14.0;
    pub const SMALL: f32 = 13.0;
    pub const CAPTION: f32 = 12.0;
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
    pub const MD: f32 = 8.0;
    pub const XL: f32 = 16.0;
    pub const PANEL: f32 = 28.0;
    pub const FULL: f32 = 999.0;
}
pub mod size {
    pub const CONTROL_HEIGHT: f32 = 40.0;
    pub const SIDEBAR_WIDTH: f32 = 240.0;
    pub const SIDEBAR_COMPACT_WIDTH: f32 = 76.0;
    /// Vertical and horizontal padding inside standard buttons.
    pub const BUTTON_PADDING: [f32; 2] = [9.0, 16.0];
}
