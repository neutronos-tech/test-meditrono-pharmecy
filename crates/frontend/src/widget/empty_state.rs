use crate::{
    theme::{self, Tone, font, typography},
    widget::icon::chip,
};
use iced::{
    Alignment, Element,
    widget::{Space, column, container, text},
};
use lucide_icons::Icon;

pub fn empty_state<'a, Message: 'a>(
    glyph: Icon,
    title: &'a str,
    description: &'a str,
    action: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = column![
        chip(glyph, Tone::Primary, 56.0),
        Space::new().height(theme::spacing::XS),
        text(title).size(typography::H2).font(font::SEMIBOLD),
        text(description)
            .size(typography::BODY)
            .style(theme::text::muted)
    ]
    .spacing(theme::spacing::SM)
    .align_x(Alignment::Center);
    if let Some(action) = action {
        content = content
            .push(Space::new().height(theme::spacing::SM))
            .push(action);
    }
    container(content)
        .padding([theme::spacing::XXL + 16.0, theme::spacing::XXL])
        .center_x(iced::Fill)
        .style(theme::container::card)
        .into()
}
