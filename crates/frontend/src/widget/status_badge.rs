use crate::theme::{self, Tone, font, typography};
use contracts::inventory::InventoryStatus;
use iced::{
    Element,
    widget::{container, text},
};

/// A small rounded label in a tone colour.
pub fn badge<'a, Message: 'a>(
    label: impl text::IntoFragment<'a>,
    tone: Tone,
) -> Element<'a, Message> {
    container(text(label).size(typography::CAPTION).font(font::MEDIUM))
        .padding([3.0, theme::spacing::SM + 2.0])
        .style(theme::container::pill(tone))
        .into()
}

pub fn inventory<'a, Message: 'a>(status: InventoryStatus) -> Element<'a, Message> {
    match status {
        InventoryStatus::InStock => badge("In stock", Tone::Success),
        InventoryStatus::LowStock => badge("Low stock", Tone::Warning),
        InventoryStatus::NearExpiry => badge("Near expiry", Tone::Warning),
        InventoryStatus::ExpiredStock => badge("Expired stock", Tone::Danger),
    }
}

pub fn active<'a, Message: 'a>(active: bool) -> Element<'a, Message> {
    if active {
        badge("Active", Tone::Success)
    } else {
        badge("Disabled", Tone::Neutral)
    }
}
