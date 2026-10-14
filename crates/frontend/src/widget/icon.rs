use crate::theme::{self, Tone};
use iced::{
    Element, Font,
    widget::{Text, container},
};
use lucide_icons::Icon;

pub const FONT: Font = Font::with_name("lucide");

pub fn icon<'a>(icon: Icon, size: f32) -> Text<'a> {
    icon.widget::<iced::Theme>().size(size)
}

/// An icon on a soft, rounded tone background.
pub fn chip<'a, Message: 'a>(glyph: Icon, tone: Tone, size: f32) -> Element<'a, Message> {
    container(icon(glyph, size * 0.5))
        .center(size)
        .style(theme::container::tone(tone))
        .into()
}
