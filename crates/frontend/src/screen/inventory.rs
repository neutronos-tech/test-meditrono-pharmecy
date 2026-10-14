use crate::{
    format, theme,
    widget::{
        data_table::data_table, empty_state::empty_state, feedback::error_banner,
        page::page_header, status_badge,
    },
};
use backend::Backend;
use contracts::inventory::{InventoryDetail, InventoryItem};
use iced::{
    Element, Length,
    widget::{button, column, row, scrollable, text},
};
use lucide_icons::Icon;
#[derive(Default)]
pub struct State {
    pub query: String,
    pub items: Vec<InventoryItem>,
    pub selected: Option<InventoryDetail>,
    pub error: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Message {
    Query(String),
    Select(usize),
    CloseDetail,
    Refresh,
}
pub fn refresh(state: &mut State, backend: &Backend) {
    match backend.inventory(&state.query, 100) {
        Ok(items) => {
            state.items = items;
            state.error = None;
        }
        Err(_) => state.error = Some("Could not load inventory.".into()),
    }
}
pub fn update(state: &mut State, message: Message, backend: &mut Backend) -> bool {
    match message {
        Message::Query(query) => {
            state.query = query;
            match backend.inventory(&state.query, 100) {
                Ok(items) => {
                    state.items = items;
                    state.error = None
                }
                Err(_) => state.error = Some("Could not search inventory.".into()),
            }
        }
        Message::Select(index) => {
            if let Some(item) = state.items.get(index) {
                match backend.inventory_detail(item.medicine_id) {
                    Ok(detail) => {
                        state.selected = Some(detail);
                        state.error = None
                    }
                    Err(_) => state.error = Some("Could not load medicine details.".into()),
                }
            }
        }
        Message::CloseDetail => state.selected = None,
        Message::Refresh => refresh(state, backend),
    }
    false
}
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![
        page_header(
            "Inventory",
            "Physical stock, including expired batches",
            Some(
                button("Refresh")
                    .style(theme::button::secondary)
                    .padding(theme::size::BUTTON_PADDING)
                    .on_press(Message::Refresh)
                    .into()
            )
        ),
        crate::widget::form::search("Search medicines", &state.query).on_input(Message::Query)
    ]
    .spacing(theme::spacing::LG);
    if let Some(e) = &state.error {
        content = content.push(error_banner(e));
    }
    if state.items.is_empty() {
        content = content.push(empty_state(
            Icon::Boxes,
            "No inventory found",
            "Try another search or receive stock.",
            None,
        ));
    } else {
        let rows = state
            .items
            .iter()
            .map(|item| {
                vec![
                    text(item.medicine_name.as_str()).into(),
                    text(format!(
                        "{} {}",
                        item.generic_name.as_deref().unwrap_or(""),
                        item.strength.as_deref().unwrap_or("")
                    ))
                    .into(),
                    text(item.total_quantity.to_string()).into(),
                    text(item.batch_count.to_string()).into(),
                    text(
                        item.nearest_expiry
                            .map(format::date)
                            .unwrap_or_else(|| "—".into()),
                    )
                    .into(),
                    text(
                        item.selling_price
                            .map(format::money)
                            .unwrap_or_else(|| "—".into()),
                    )
                    .into(),
                    status_badge::inventory(item.status),
                ]
            })
            .collect();
        content = content.push(data_table(
            &[
                "Medicine",
                "Generic / strength",
                "Stock",
                "Batches",
                "Nearest expiry",
                "Selling price",
                "Status",
            ],
            rows,
            Some(Message::Select),
        ));
    }
    if let Some(detail) = &state.selected {
        content = content.push(
            row![
                text(&detail.medicine.name).size(20),
                button("Close")
                    .style(theme::button::ghost)
                    .on_press(Message::CloseDetail)
            ]
            .spacing(theme::spacing::LG),
        );
        content = content.push(text(format!(
            "Total physical stock: {}",
            detail.total_quantity
        )));
        let rows = detail
            .batches
            .iter()
            .map(|b| {
                vec![
                    text(b.batch_number.as_str()).into(),
                    text(format::date(b.expiry_date)).into(),
                    text(b.quantity_on_hand.to_string()).into(),
                ]
            })
            .collect();
        content = content.push(data_table(&["Batch", "Expiry", "Quantity"], rows, None));
    }
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
