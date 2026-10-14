use crate::theme::{self, font, typography};
use iced::{
    Element, Length,
    widget::{Space, button, column, container, row, scrollable, text},
};

/// A card-framed table. Rows scroll horizontally when the window is narrow.
pub fn data_table<'a, Message: Clone + 'a>(
    headers: &[&str],
    rows: Vec<Vec<Element<'a, Message>>>,
    on_row: Option<fn(usize) -> Message>,
) -> Element<'a, Message> {
    let cell_padding = [theme::spacing::MD, theme::spacing::LG];
    let header = row(headers.iter().map(|h| {
        container(
            text(h.to_uppercase())
                .size(typography::CAPTION)
                .font(font::SEMIBOLD)
                .style(theme::text::muted),
        )
        .width(Length::FillPortion(1))
        .into()
    }))
    .spacing(theme::spacing::SM);
    let mut content = column![
        container(header)
            .padding(cell_padding)
            .width(Length::Fill)
            .style(theme::container::table_header)
    ];
    for (index, cells) in rows.into_iter().enumerate() {
        content = content.push(
            container(Space::new().height(1))
                .width(Length::Fill)
                .style(theme::container::divider),
        );
        let line = row(cells
            .into_iter()
            .map(|cell| container(cell).width(Length::FillPortion(1)).into()))
        .spacing(theme::spacing::SM)
        .align_y(iced::Alignment::Center);
        let line: Element<'a, Message> = if let Some(on_row) = on_row {
            button(line)
                .style(theme::button::row)
                .padding(cell_padding)
                .width(Length::Fill)
                .on_press(on_row(index))
                .into()
        } else {
            container(line)
                .padding(cell_padding)
                .width(Length::Fill)
                .style(theme::container::table_row)
                .into()
        };
        content = content.push(line);
    }
    let min_width = (headers.len() as f32 * 160.0).max(700.0);
    container(
        scrollable(content.width(Length::Fixed(min_width)))
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default(),
            ))
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .clip(true)
    .style(theme::container::card)
    .into()
}
