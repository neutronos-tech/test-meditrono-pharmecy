use crate::theme;
use iced::{
    Element,
    widget::{container, text},
};
pub fn success_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).style(theme::text::success))
        .padding(theme::spacing::MD)
        .style(theme::container::card)
        .into()
}
pub fn error_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).style(theme::text::danger))
        .padding(theme::spacing::MD)
        .style(theme::container::card)
        .into()
}
pub fn info_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    container(text(message).style(theme::text::primary))
        .padding(theme::spacing::MD)
        .style(theme::container::card)
        .into()
}
