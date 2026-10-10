use crate::theme;
use iced::{
    Element, Length,
    widget::{Space, column, row, text},
};
pub fn page_header<'a, Message: 'a>(
    title: &'a str,
    subtitle: &'a str,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut line = row![
        text(title).size(28).style(theme::text::primary),
        Space::new().width(Length::Fill)
    ]
    .align_y(iced::Alignment::Center);
    if let Some(action) = action {
        line = line.push(action);
    }
    column![line, text(subtitle).size(14).style(theme::text::muted)]
        .spacing(theme::spacing::XS)
        .into()
}
