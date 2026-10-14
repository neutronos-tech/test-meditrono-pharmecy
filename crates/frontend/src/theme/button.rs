use super::{
    palette::{Tokens, tokens},
    radius,
};
use iced::{
    Background, Border, Color, Shadow, Theme, Vector,
    widget::button::{Status, Style},
};

fn base(background: Option<Color>, text: Color, border: Color) -> Style {
    Style {
        background: background.map(Background::Color),
        text_color: text,
        border: Border {
            color: border,
            width: if border.a > 0.0 { 1.0 } else { 0.0 },
            radius: radius::MD.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

fn disabled(mut style: Style) -> Style {
    let fade = |c: Color| Color { a: c.a * 0.45, ..c };
    style.background = style.background.map(|b| match b {
        Background::Color(c) => Background::Color(fade(c)),
        other => other,
    });
    style.text_color = fade(style.text_color);
    style.border.color = fade(style.border.color);
    style.shadow = Shadow::default();
    style
}

fn lift(t: &Tokens) -> Shadow {
    Shadow {
        color: t.shadow,
        offset: Vector::new(0.0, 1.0),
        blur_radius: 2.0,
    }
}

pub fn primary(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let mut s = base(Some(t.primary), t.on_primary, Color::TRANSPARENT);
    s.shadow = lift(&t);
    match status {
        Status::Active => s,
        Status::Hovered | Status::Pressed => Style {
            background: Some(t.primary_hover.into()),
            ..s
        },
        Status::Disabled => disabled(s),
    }
}

pub fn secondary(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let s = base(Some(t.surface), t.text, t.border_strong);
    match status {
        Status::Active => Style {
            shadow: lift(&t),
            ..s
        },
        Status::Hovered | Status::Pressed => Style {
            background: Some(t.surface_alt.into()),
            ..s
        },
        Status::Disabled => disabled(s),
    }
}

pub fn danger(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let s = base(Some(t.danger_soft), t.danger, Color::TRANSPARENT);
    match status {
        Status::Active => s,
        Status::Hovered | Status::Pressed => base(Some(t.danger), Color::WHITE, Color::TRANSPARENT),
        Status::Disabled => disabled(s),
    }
}

pub fn ghost(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let s = base(None, t.text, Color::TRANSPARENT);
    match status {
        Status::Active => s,
        Status::Hovered | Status::Pressed => Style {
            background: Some(t.surface_alt.into()),
            ..s
        },
        Status::Disabled => disabled(s),
    }
}

/// A clickable table row: flat, highlighted on hover.
pub fn row(theme: &Theme, status: Status) -> Style {
    let mut s = ghost(theme, status);
    s.border.radius = 0.0.into();
    s
}

pub fn sidebar(theme: &Theme, status: Status) -> Style {
    let t = tokens(theme);
    let s = base(None, t.sidebar_text_muted, Color::TRANSPARENT);
    match status {
        Status::Active | Status::Disabled => s,
        Status::Hovered | Status::Pressed => Style {
            background: Some(t.sidebar_hover.into()),
            text_color: t.sidebar_text,
            ..s
        },
    }
}

/// The current page: a tab in the panel colour that runs into the panel.
pub fn sidebar_active(theme: &Theme, _status: Status) -> Style {
    let t = tokens(theme);
    let mut s = base(Some(t.sidebar_active), t.primary, Color::TRANSPARENT);
    s.border.radius = iced::border::Radius::default()
        .top_left(radius::PANEL)
        .bottom_left(radius::PANEL);
    s
}

/// One option of a segmented control on the sidebar (light/dark switch).
pub fn segment(theme: &Theme, status: Status) -> Style {
    let mut s = sidebar(theme, status);
    s.border.radius = radius::SM.into();
    s
}

pub fn segment_active(theme: &Theme, _status: Status) -> Style {
    let t = tokens(theme);
    let mut s = base(Some(t.sidebar_active), t.primary, Color::TRANSPARENT);
    s.border.radius = radius::SM.into();
    s
}
