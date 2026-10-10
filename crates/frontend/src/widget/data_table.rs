use crate::theme;
use iced::{
    Element, Length,
    widget::{button, column, container, row, scrollable, text},
};
pub fn data_table<'a, Message: Clone + 'a>(
    headers: &[&str],
    rows: Vec<Vec<Element<'a, Message>>>,
    on_row: Option<fn(usize) -> Message>,
) -> Element<'a, Message> {
    let header = row(headers.iter().map(|h| {
        container(text((*h).to_string()).size(12).style(theme::text::muted))
            .width(Length::FillPortion(1))
            .into()
    }))
    .spacing(theme::spacing::SM);
    let mut content = column![
        container(header)
            .padding(theme::spacing::MD)
            .style(theme::container::table_header)
    ];
    for (index, cells) in rows.into_iter().enumerate() {
        let line = row(cells
            .into_iter()
            .map(|cell| container(cell).width(Length::FillPortion(1)).into()))
        .spacing(theme::spacing::SM)
        .align_y(iced::Alignment::Center);
        let line: Element<'a, Message> = if let Some(on_row) = on_row {
            button(line)
                .style(theme::button::ghost)
                .width(Length::Fill)
                .on_press(on_row(index))
                .into()
        } else {
            container(line)
                .padding(theme::spacing::MD)
                .width(Length::Fill)
                .style(theme::container::table_row)
                .into()
        };
        content = content.push(line);
    }
    let min_width = (headers.len() as f32 * 160.0).max(700.0);
    scrollable(content.width(Length::Fixed(min_width)))
        .direction(iced::widget::scrollable::Direction::Horizontal(
            iced::widget::scrollable::Scrollbar::default(),
        ))
        .into()
}
