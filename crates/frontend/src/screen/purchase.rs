use crate::{
    theme,
    validation::{decimal_from_ui, required},
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
    catalog::{Medicine, Supplier},
    purchase::{PurchaseInput, PurchaseItemInput},
};
use iced::{
    Element, Length,
    widget::{Space, button, column, container, row, scrollable, text},
};
use iced_aw::NumberInput;
use lucide_icons::Icon;
use uuid::Uuid;
#[derive(Default)]
pub struct State {
    pub supplier_query: String,
    pub supplier_results: Vec<Supplier>,
    pub selected_supplier: Option<Supplier>,
    pub invoice_number: String,
    pub medicine_query: String,
    pub medicine_results: Vec<Medicine>,
    pub draft_items: Vec<DraftItem>,
    pub error: Option<String>,
    pub success: Option<String>,
}
pub struct DraftItem {
    pub medicine: Medicine,
    pub quantity: u32,
    pub unit_cost: f64,
}
#[derive(Debug, Clone)]
pub enum Message {
    SupplierQuery(String),
    SelectSupplier(Uuid),
    Invoice(String),
    MedicineQuery(String),
    AddMedicine(Uuid),
    Quantity(usize, u32),
    Cost(usize, f64),
    Remove(usize),
    Submit,
    Refresh,
}
impl State {
    fn build_input(&self) -> Result<PurchaseInput, String> {
        let supplier_id = self
            .selected_supplier
            .as_ref()
            .ok_or("Select a supplier.")?
            .id;
        let invoice_number = required(&self.invoice_number)?;
        if self.draft_items.is_empty() {
            return Err("Add at least one medicine.".into());
        }
        let mut seen = std::collections::HashSet::new();
        let mut items = Vec::new();
        for item in &self.draft_items {
            if !seen.insert(item.medicine.id) {
                return Err("A medicine appears more than once.".into());
            }
            if item.quantity == 0 {
                return Err("Quantity must be at least 1.".into());
            }
            items.push(PurchaseItemInput {
                medicine_id: item.medicine.id,
                quantity: item.quantity,
                unit_cost: decimal_from_ui(item.unit_cost)?,
            });
        }
        Ok(PurchaseInput {
            supplier_id,
            invoice_number,
            items,
        })
    }
}
pub fn refresh(state: &mut State, backend: &Backend) {
    if let Ok(items) = backend.search_suppliers(&state.supplier_query, 20) {
        state.supplier_results = items;
    }
    if let Ok(items) = backend.search_medicines(&state.medicine_query, 20) {
        state.medicine_results = items;
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
        Message::SupplierQuery(query) => {
            state.supplier_query = query;
            match backend.search_suppliers(&state.supplier_query, 20) {
                Ok(v) => state.supplier_results = v,
                Err(_) => state.error = Some("Could not search suppliers.".into()),
            }
        }
        Message::SelectSupplier(id) => {
            state.selected_supplier = state.supplier_results.iter().find(|s| s.id == id).cloned();
            state.supplier_query = state
                .selected_supplier
                .as_ref()
                .map_or(String::new(), |s| s.name.clone());
            state.supplier_results.clear();
        }
        Message::Invoice(value) => state.invoice_number = value,
        Message::MedicineQuery(query) => {
            state.medicine_query = query;
            match backend.search_medicines(&state.medicine_query, 20) {
                Ok(v) => state.medicine_results = v,
                Err(_) => state.error = Some("Could not search medicines.".into()),
            }
        }
        Message::AddMedicine(id) => {
            if let Some(medicine) = state.medicine_results.iter().find(|m| m.id == id).cloned() {
                if state.draft_items.iter().any(|i| i.medicine.id == id) {
                    state.error = Some("Medicine already added.".into());
                } else {
                    let cost = medicine
                        .reference_price
                        .map(crate::format::decimal_as_f64)
                        .unwrap_or(0.0);
                    state.draft_items.push(DraftItem {
                        medicine,
                        quantity: 1,
                        unit_cost: cost,
                    });
                    state.medicine_query.clear();
                    state.medicine_results.clear();
                }
            }
        }
        Message::Quantity(i, value) => {
            if let Some(item) = state.draft_items.get_mut(i) {
                item.quantity = value;
            }
        }
        Message::Cost(i, value) => {
            if let Some(item) = state.draft_items.get_mut(i) {
                item.unit_cost = value;
            }
        }
        Message::Remove(i) => {
            if i < state.draft_items.len() {
                state.draft_items.remove(i);
            }
        }
        Message::Refresh => refresh(state, backend),
        Message::Submit => {
            match state.build_input() {
                Ok(input) => match backend.create_purchase(pharmacy, input) {
                    Ok(purchase) => {
                        state.success =
                            Some(format!("Purchase {} created.", purchase.invoice_number));
                        state.invoice_number.clear();
                        state.selected_supplier = None;
                        state.supplier_query.clear();
                        state.medicine_query.clear();
                        state.draft_items.clear();
                    }
                    Err(e) => {
                        state.error=Some(match e {Error::DuplicateInvoice=>"This invoice already exists for the supplier.".into(),Error::NotFound=>"Supplier or medicine is no longer available. Refresh and try again.".into(),_=>"Could not create purchase.".into()});
                    }
                },
                Err(e) => state.error = Some(e),
            }
        }
    }
    false
}
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![page_header(
        "New purchase",
        "Record an invoice before receiving stock",
        Some(
            button("Refresh")
                .style(theme::button::secondary)
                .padding(theme::size::BUTTON_PADDING)
                .on_press(Message::Refresh)
                .into()
        )
    )]
    .spacing(theme::spacing::LG);
    if let Some(e) = &state.error {
        content = content.push(error_banner(e));
    }
    if let Some(s) = &state.success {
        content = content.push(success_banner(s));
    }
    content = content.push(text("Supplier").size(16)).push(
        crate::widget::form::search("Search suppliers", &state.supplier_query)
            .on_input(Message::SupplierQuery),
    );
    for supplier in state.supplier_results.iter().take(6) {
        content = content.push(
            button(row![icon(Icon::Truck, 16.0), text(&supplier.name)].spacing(theme::spacing::SM))
                .style(theme::button::ghost)
                .on_press(Message::SelectSupplier(supplier.id))
                .width(Length::Fill),
        );
    }
    content = content.push(text("Invoice number")).push(
        crate::widget::form::input("Invoice number", &state.invoice_number)
            .on_input(Message::Invoice),
    );
    content = content.push(text("Add medicines").size(16)).push(
        crate::widget::form::search("Search catalog", &state.medicine_query)
            .on_input(Message::MedicineQuery),
    );
    for medicine in state.medicine_results.iter().take(8) {
        content = content.push(
            button(row![icon(Icon::Plus, 16.0), text(&medicine.name)].spacing(theme::spacing::SM))
                .style(theme::button::ghost)
                .on_press(Message::AddMedicine(medicine.id))
                .width(Length::Fill),
        );
    }
    if state.draft_items.is_empty() {
        content = content.push(empty_state(
            Icon::Pill,
            "No items added",
            "Search for medicines to add purchase lines.",
            None,
        ));
    }
    for (index, item) in state.draft_items.iter().enumerate() {
        content = content.push(
            container(
                row![
                    text(&item.medicine.name).width(Length::FillPortion(3)),
                    NumberInput::new(&item.quantity, 1..=u32::MAX, move |n| Message::Quantity(
                        index, n
                    ))
                    .width(120),
                    NumberInput::new(&item.unit_cost, 0.0..=1_000_000_000.0, move |n| {
                        Message::Cost(index, n)
                    })
                    .step(0.01)
                    .width(140),
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
    content = content
        .push(
            button("Create purchase")
                .style(theme::button::primary)
                .padding(theme::size::BUTTON_PADDING)
                .on_press(Message::Submit),
        )
        .push(Space::new().height(theme::spacing::XL));
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
