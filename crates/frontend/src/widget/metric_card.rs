use crate::{theme, widget::icon::icon};
use iced::{
    Element, Length,
    widget::{column, container, row, text},
};
use lucide_icons::Icon;
pub fn metric_card<'a, Message: 'a>(
    glyph: Icon,
    label: &'a str,
    value: String,
    note: &'a str,
) -> Element<'a, Message> {
    container(
        column![
            row![
                icon(glyph, 20.0),
                text(label).size(14).style(theme::text::muted)
            ]
            .spacing(theme::spacing::SM),
            text(value).size(27),
            text(note).size(12).style(theme::text::muted)
        ]
        .spacing(theme::spacing::SM),
    )
    .padding(theme::spacing::LG)
    .width(Length::Fill)
    .style(theme::container::card)
    .into()
}
