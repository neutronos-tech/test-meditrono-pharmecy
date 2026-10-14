use crate::theme::{self, font, typography};
use iced::{
    Alignment, Element, Length,
    widget::{Space, column, row, text},
};

pub fn page_header<'a, Message: 'a>(
    title: &'a str,
    subtitle: &'a str,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![
        column![
            text(title)
                .size(typography::H1)
                .font(font::SEMIBOLD)
                .style(theme::text::primary),
            text(subtitle)
                .size(typography::BODY)
                .style(theme::text::muted)
        ]
        .spacing(theme::spacing::XS),
        Space::new().width(Length::Fill)
    ]
    .spacing(theme::spacing::LG)
    .align_y(Alignment::Center);
    if let Some(action) = action {
        line = line.push(action);
    }
    line.into()
}

/// A titled group inside a page, e.g. "Recent activity".
pub fn section_title<'a, Message: 'a>(title: &'a str) -> Element<'a, Message> {
    text(title)
        .size(typography::H2)
        .font(font::SEMIBOLD)
        .style(theme::text::primary)
        .into()
}
