use super::{
    Tone,
    palette::{Tokens, tokens},
    radius,
};
use iced::{Border, Shadow, Theme, Vector, widget::container::Style};

fn card_shadow(t: &Tokens) -> Shadow {
    Shadow {
        color: t.shadow,
        offset: Vector::new(0.0, 4.0),
        blur_radius: 18.0,
    }
}

pub fn app(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style::default().background(t.canvas).color(t.text)
}

/// The coloured window frame that holds the sidebar and the content panel.
pub fn frame(theme: &Theme) -> Style {
    Style::default().background(tokens(theme).frame)
}

/// The large rounded panel that holds each page.
pub fn panel(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style {
        background: Some(t.canvas.into()),
        text_color: Some(t.text),
        border: Border {
            radius: radius::PANEL.into(),
            ..Border::default()
        },
        ..Style::default()
    }
}

/// A warm, borderless tile for metrics and highlights.
pub fn tile(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style {
        background: Some(t.surface_alt.into()),
        border: Border {
            radius: radius::XL.into(),
            ..Border::default()
        },
        shadow: card_shadow(&t),
        ..Style::default()
    }
}

pub fn sidebar(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style::default().background(t.sidebar).color(t.sidebar_text)
}

pub fn card(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style {
        background: Some(t.surface.into()),
        border: Border {
            color: t.border,
            width: 1.0,
            radius: radius::XL.into(),
        },
        shadow: card_shadow(&t),
        ..Style::default()
    }
}

pub fn table_header(theme: &Theme) -> Style {
    Style::default().background(tokens(theme).surface_alt)
}

pub fn table_row(theme: &Theme) -> Style {
    Style::default().background(tokens(theme).surface)
}

/// A 1px line between table rows or sections.
pub fn divider(theme: &Theme) -> Style {
    Style::default().background(tokens(theme).border)
}

pub fn modal(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style {
        shadow: Shadow {
            color: t.shadow,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 24.0,
        },
        ..card(theme)
    }
}

/// Soft tinted fill for badges, icon chips and banners.
pub fn tone(tone: Tone) -> impl Fn(&Theme) -> Style {
    move |theme| {
        let t = tokens(theme);
        let (fg, bg) = tone.colors(&t);
        Style {
            background: Some(bg.into()),
            text_color: Some(fg),
            border: Border {
                radius: radius::MD.into(),
                ..Border::default()
            },
            ..Style::default()
        }
    }
}

/// A fully rounded `tone` fill.
pub fn pill(tone: Tone) -> impl Fn(&Theme) -> Style {
    move |theme| {
        let mut s = self::tone(tone)(theme);
        s.border.radius = radius::FULL.into();
        s
    }
}

/// A banner: soft tone fill with a tone-coloured border.
pub fn banner(tone: Tone) -> impl Fn(&Theme) -> Style {
    move |theme| {
        let t = tokens(theme);
        let (fg, bg) = tone.colors(&t);
        Style {
            background: Some(bg.into()),
            text_color: Some(fg),
            border: Border {
                color: iced::Color { a: 0.35, ..fg },
                width: 1.0,
                radius: radius::MD.into(),
            },
            ..Style::default()
        }
    }
}

/// A round badge on the sidebar: the logo mark and the account initial.
pub fn avatar(theme: &Theme) -> Style {
    let t = tokens(theme);
    Style {
        background: Some(t.sidebar_active.into()),
        text_color: Some(t.primary),
        border: Border {
            radius: radius::FULL.into(),
            ..Border::default()
        },
        ..Style::default()
    }
}

/// A translucent panel on the sidebar (theme switch track, account card).
pub fn segmented(_theme: &Theme) -> Style {
    Style {
        background: Some(iced::Color::from_rgba(1.0, 1.0, 1.0, 0.12).into()),
        border: Border {
            radius: radius::MD.into(),
            ..Border::default()
        },
        ..Style::default()
    }
}
