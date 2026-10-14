use crate::{
    theme::{self, Tone, font, typography},
    widget::icon::chip,
};
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, container, row, text},
};
use lucide_icons::Icon;

pub fn metric_card<'a, Message: 'a>(
    glyph: Icon,
    label: &'a str,
    value: String,
    note: &'a str,
) -> Element<'a, Message> {
    toned_metric_card(glyph, Tone::Primary, label, value, note)
}

/// A metric card whose icon chip uses `tone`, e.g. accent for expiry.
pub fn toned_metric_card<'a, Message: 'a>(
    glyph: Icon,
    tone: Tone,
    label: &'a str,
    value: String,
    note: &'a str,
) -> Element<'a, Message> {
    container(
        column![
            row![
                text(label)
                    .size(typography::SMALL)
                    .font(font::MEDIUM)
                    .style(theme::text::muted),
                Space::new().width(Length::Fill),
                chip(glyph, tone, 36.0),
            ]
            .align_y(Alignment::Center),
            text(value).size(typography::DISPLAY - 4.0).font(font::BOLD),
            text(note)
                .size(typography::CAPTION)
                .style(theme::text::muted)
        ]
        .spacing(theme::spacing::SM),
    )
    .padding(theme::spacing::LG + 4.0)
    .width(Length::Fill)
    .style(theme::container::tile)
    .into()
}
