use crate::{
    format, theme,
    validation::decimal_from_ui,
    widget::{
        empty_state::empty_state,
        feedback::{error_banner, success_banner},
        icon::icon,
        page::page_header,
    },
};
use backend::{Backend, Error};
use contracts::{
    auth::PharmacyAccount,
    inventory::InventoryItem,
    sale::{SaleInput, SaleItemInput},
};
use iced::{
    Element, Length,
    widget::{button, column, container, row, scrollable, text, text_input},
};
use iced_aw::NumberInput;
use lucide_icons::Icon;
use uuid::Uuid;
#[derive(Default)]
pub struct State {
    pub query: String,
    pub results: Vec<InventoryItem>,
    pub items: Vec<DraftSaleItem>,
    pub error: Option<String>,
    pub success: Option<String>,
}
pub struct DraftSaleItem {
    pub medicine_id: Uuid,
    pub medicine_name: String,
    pub available: u64,
    pub quantity: u32,
    pub unit_price: f64,
}
#[derive(Debug, Clone)]
pub enum Message {
    Query(String),
    Add(Uuid),
    Quantity(usize, u32),
    Price(usize, f64),
    Remove(usize),
    Submit,
    Refresh,
}
impl State {
    fn build_input(&self) -> Result<SaleInput, String> {
        if self.items.is_empty() {
            return Err("Add at least one medicine.".into());
        }
        let mut seen = std::collections::HashSet::new();
        let mut items = Vec::new();
        for item in &self.items {
            if !seen.insert(item.medicine_id) {
                return Err("A medicine appears twice.".into());
            }
            if item.quantity == 0 || u64::from(item.quantity) > item.available {
                return Err(format!("Review quantity for {}.", item.medicine_name));
            }
            items.push(SaleItemInput {
                medicine_id: item.medicine_id,
                quantity: item.quantity,
                unit_price: decimal_from_ui(item.unit_price)?,
            });
        }
        Ok(SaleInput { items })
    }
}
pub fn refresh(state: &mut State, backend: &Backend) {
    match backend.sellable_inventory(&state.query, 30) {
        Ok(items) => {
            state.results = items;
            state.error = None;
        }
        Err(_) => state.error = Some("Could not load sellable stock.".into()),
    }
}
pub fn update(
    state: &mut State,
    message: Message,
    backend: &mut Backend,
    pharmacy: &PharmacyAccount,
) -> bool {
    state.error = None;
    state.success = None;
    match message {
        Message::Query(query) => {
            state.query = query;
            match backend.sellable_inventory(&state.query, 30) {
                Ok(items) => state.results = items,
                Err(_) => state.error = Some("Could not search sellable stock.".into()),
            }
        }
        Message::Add(id) => {
            if let Some(item) = state.results.iter().find(|i| i.medicine_id == id) {
                if state.items.iter().any(|i| i.medicine_id == id) {
                    state.error = Some("Medicine already added.".into());
                } else {
                    state.items.push(DraftSaleItem {
                        medicine_id: id,
                        medicine_name: item.medicine_name.clone(),
                        available: item.total_quantity,
                        quantity: 1,
                        unit_price: item
                            .selling_price
                            .map(format::decimal_as_f64)
                            .unwrap_or(0.0),
                    });
                    state.query.clear();
                    state.results.clear();
                }
            }
        }
        Message::Quantity(i, value) => {
            if let Some(item) = state.items.get_mut(i) {
                item.quantity = value;
            }
        }
        Message::Price(i, value) => {
            if let Some(item) = state.items.get_mut(i) {
                item.unit_price = value;
            }
        }
        Message::Remove(i) => {
            if i < state.items.len() {
                state.items.remove(i);
            }
        }
        Message::Refresh => refresh(state, backend),
        Message::Submit => match state.build_input() {
            Ok(input) => match backend.create_sale(pharmacy, input) {
                Ok(_) => {
                    state.items.clear();
                    state.success = Some("Sale completed.".into());
                    refresh(state, backend);
                }
                Err(e) => {
                    refresh(state, backend);
                    state.error=Some(match e {Error::InsufficientStock=>"Stock changed or is no longer sufficient. Review the sale and try again.".into(),_=>"Could not complete the sale.".into()});
                }
            },
            Err(e) => state.error = Some(e),
        },
    }
    false
}
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![
        page_header(
            "New sale",
            "Sell available, unexpired stock",
            Some(
                button("Refresh stock")
                    .style(theme::button::secondary)
                    .on_press(Message::Refresh)
                    .into()
            )
        ),
        text_input("Search sellable medicines", &state.query)
            .style(theme::text::input)
            .on_input(Message::Query)
    ]
    .spacing(theme::spacing::LG);
    if let Some(e) = &state.error {
        content = content.push(error_banner(e));
    }
    if let Some(s) = &state.success {
        content = content.push(success_banner(s));
    }
    for item in state.results.iter().take(10) {
        content = content.push(
            button(
                row![
                    icon(Icon::Plus, 16.0),
                    text(format!(
                        "{} · {} available",
                        item.medicine_name, item.total_quantity
                    ))
                ]
                .spacing(theme::spacing::SM),
            )
            .style(theme::button::ghost)
            .width(Length::Fill)
            .on_press(Message::Add(item.medicine_id)),
        );
    }
    if state.items.is_empty() {
        content = content.push(empty_state(
            Icon::ShoppingCart,
            "Cart is empty",
            "Search available medicines to start a sale.",
            None,
        ));
    }
    for (index, item) in state.items.iter().enumerate() {
        content = content.push(
            container(
                row![
                    text(&item.medicine_name).width(Length::FillPortion(3)),
                    text(format!("{} available", item.available)).style(theme::text::muted),
                    NumberInput::new(
                        &item.quantity,
                        1..=u32::try_from(item.available).unwrap_or(u32::MAX),
                        move |n| Message::Quantity(index, n)
                    )
                    .width(110),
                    NumberInput::new(&item.unit_price, 0.0..=1_000_000_000.0, move |n| {
                        Message::Price(index, n)
                    })
                    .step(0.01)
                    .width(130),
                    button(icon(Icon::Trash2, 16.0))
                        .style(theme::button::ghost)
                        .on_press(Message::Remove(index))
                ]
                .spacing(theme::spacing::MD)
                .align_y(iced::Alignment::Center),
            )
            .padding(theme::spacing::MD)
            .style(theme::container::card),
        );
    }
    content = content.push(
        button("Complete sale")
            .style(theme::button::primary)
            .on_press(Message::Submit),
    );
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
