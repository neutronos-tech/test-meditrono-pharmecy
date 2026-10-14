use crate::{
    theme::{self, font, typography},
    widget::icon,
};
use iced::{
    Element,
    widget::{TextInput, column, text, text_input},
};
use lucide_icons::Icon;

/// A labelled form control with an optional validation message under it.
pub fn field<'a, Message: 'a>(
    label: &'a str,
    control: impl Into<Element<'a, Message>>,
    error: Option<&'a str>,
) -> Element<'a, Message> {
    let mut content = column![
        text(label)
            .size(typography::SMALL)
            .font(font::MEDIUM)
            .style(theme::text::primary),
        control.into()
    ]
    .spacing(theme::spacing::XS + 2.0);
    if let Some(error) = error {
        content = content.push(
            text(error)
                .size(typography::CAPTION)
                .style(theme::text::danger),
        );
    }
    content.into()
}

/// A styled text input at the standard control height.
pub fn input<'a, Message: Clone + 'a>(placeholder: &str, value: &str) -> TextInput<'a, Message> {
    text_input(placeholder, value)
        .style(theme::text::input)
        .padding([theme::spacing::SM + 2.0, theme::spacing::MD])
}

/// A text input with a leading search icon.
pub fn search<'a, Message: Clone + 'a>(placeholder: &str, value: &str) -> TextInput<'a, Message> {
    input(placeholder, value).icon(text_input::Icon {
        font: icon::FONT,
        code_point: Icon::Search.unicode(),
        size: Some(16.into()),
        spacing: theme::spacing::SM,
        side: text_input::Side::Left,
    })
}
