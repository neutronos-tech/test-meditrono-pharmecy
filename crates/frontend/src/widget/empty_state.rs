use crate::{theme, widget::icon::icon};
use iced::{
    Alignment, Element,
    widget::{column, container, text},
};
use lucide_icons::Icon;
pub fn empty_state<'a, Message: 'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = column![
        icon(glyph, 28.0),
        text(title).size(18),
        text(description).style(theme::text::muted)
    ]
    .spacing(theme::spacing::SM)
    .align_x(Alignment::Center);
    if let Some(action) = action {
        content = content.push(action);
    }
    container(content)
        .padding(theme::spacing::XXL)
        .center_x(iced::Fill)
        .style(theme::container::card)
        .into()
}
