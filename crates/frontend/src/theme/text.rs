use super::palette;
use iced::{Theme, widget::text};
pub fn primary(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text),
    }
}
pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette::muted(theme)),
    }
}
pub fn success(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().success),
    }
}
pub fn warning(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().warning),
    }
}
pub fn danger(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().danger),
    }
}

pub fn input(
    theme: &Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    let mut style = iced::widget::text_input::default(theme, status);
    style.border.radius = super::radius::SM.into();
    style
}
