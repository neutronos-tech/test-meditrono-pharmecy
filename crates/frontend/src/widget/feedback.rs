use crate::{
    theme::{self, Tone},
    widget::icon::icon,
};
use iced::{
    Alignment, Element, Length,
    widget::{container, row, text},
};
use lucide_icons::Icon;

fn banner<'a, Message: 'a>(glyph: Icon, tone: Tone, message: &'a str) -> Element<'a, Message> {
    container(
        row![icon(glyph, 16.0), text(message)]
            .spacing(theme::spacing::SM + 2.0)
            .align_y(Alignment::Center),
    )
    .padding([theme::spacing::MD - 2.0, theme::spacing::LG])
    .width(Length::Fill)
    .style(theme::container::banner(tone))
    .into()
}

pub fn success_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    banner(Icon::CircleCheck, Tone::Success, message)
}
pub fn error_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    banner(Icon::CircleAlert, Tone::Danger, message)
}
pub fn info_banner<'a, Message: 'a>(message: &'a str) -> Element<'a, Message> {
    banner(Icon::Info, Tone::Info, message)
}
