use super::{palette, radius};
use iced::{Theme, widget::button};

pub fn primary(theme: &Theme, status: button::Status) -> button::Style {
    button::primary(theme, status)
}
pub fn secondary(theme: &Theme, status: button::Status) -> button::Style {
    button::secondary(theme, status)
}
pub fn danger(theme: &Theme, status: button::Status) -> button::Style {
    button::danger(theme, status)
}
pub fn ghost(theme: &Theme, status: button::Status) -> button::Style {
    let mut s = button::text(theme, status);
    s.border.radius = radius::SM.into();
    s
}
pub fn sidebar(theme: &Theme, status: button::Status) -> button::Style {
    let mut s = ghost(theme, status);
    if matches!(status, button::Status::Hovered) {
        s.background = Some(palette::surface(theme).into());
    }
    s
}
pub fn sidebar_active(theme: &Theme, status: button::Status) -> button::Style {
    let mut s = sidebar(theme, status);
    s.background = Some(theme.extended_palette().primary.weak.color.into());
    s.text_color = theme.palette().primary;
    s
}
