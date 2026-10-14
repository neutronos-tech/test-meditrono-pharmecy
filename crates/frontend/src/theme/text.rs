use super::{palette::tokens, radius};
use iced::{
    Background, Border, Theme,
    widget::{
        text::Style,
        text_input::{self, Status},
    },
};

pub fn primary(theme: &Theme) -> Style {
    Style {
        color: Some(tokens(theme).text),
    }
}
pub fn muted(theme: &Theme) -> Style {
    Style {
        color: Some(tokens(theme).text_muted),
    }
}
pub fn danger(theme: &Theme) -> Style {
    Style {
        color: Some(tokens(theme).danger),
    }
}
pub fn sidebar(theme: &Theme) -> Style {
    Style {
        color: Some(tokens(theme).sidebar_text),
    }
}
pub fn sidebar_muted(theme: &Theme) -> Style {
    Style {
        color: Some(tokens(theme).sidebar_text_muted),
    }
}

pub fn input(theme: &Theme, status: Status) -> text_input::Style {
    let t = tokens(theme);
    let border = |color, width| Border {
        color,
        width,
        radius: radius::MD.into(),
    };
    let style = text_input::Style {
        background: Background::Color(t.surface),
        border: border(t.border_strong, 1.0),
        icon: t.text_muted,
        placeholder: t.text_muted,
        value: t.text,
        selection: iced::Color {
            a: 0.3,
            ..t.primary
        },
    };
    match status {
        Status::Active => style,
        Status::Hovered => text_input::Style {
            border: border(t.text_muted, 1.0),
            ..style
        },
        Status::Focused { .. } => text_input::Style {
            border: border(t.primary, 2.0),
            ..style
        },
        Status::Disabled => text_input::Style {
            background: Background::Color(t.surface_alt),
            value: t.text_muted,
            ..style
        },
    }
}
