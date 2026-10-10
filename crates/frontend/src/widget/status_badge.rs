use crate::theme;
use contracts::inventory::InventoryStatus;
use iced::{
    Element,
    widget::{container, text},
};
pub fn inventory<'a, Message: 'a>(status: InventoryStatus) -> Element<'a, Message> {
    let (label, style): (&str, fn(&iced::Theme) -> iced::widget::text::Style) = match status {
        InventoryStatus::InStock => ("In stock", theme::text::success),
        InventoryStatus::LowStock => ("Low stock", theme::text::warning),
        InventoryStatus::NearExpiry => ("Near expiry", theme::text::warning),
        InventoryStatus::ExpiredStock => ("Expired stock", theme::text::danger),
    };
    container(text(label).size(12).style(style))
        .padding([theme::spacing::XS, theme::spacing::SM])
        .style(theme::container::surface)
        .into()
}
pub fn active<'a, Message: 'a>(active: bool) -> Element<'a, Message> {
    let (label, style): (&str, fn(&iced::Theme) -> iced::widget::text::Style) = if active {
        ("Active", theme::text::success)
    } else {
        ("Disabled", theme::text::muted)
    };
    container(text(label).style(style))
        .padding(theme::spacing::XS)
        .into()
}
