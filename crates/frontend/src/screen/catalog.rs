use crate::{
    format, theme,
    validation::{decimal_from_ui, required},
    widget::{
        feedback::{error_banner, success_banner},
        page::page_header,
        status_badge,
    },
};
use backend::{Backend, Error};
use contracts::{
    auth::PharmacyAccount,
    catalog::{CreateMedicineInput, CreateSupplierInput, Medicine, SetSellingPriceInput, Supplier},
};
use iced::{
    Element, Length,
    widget::{button, column, container, row, scrollable, text, text_input},
};
use iced_aw::NumberInput;
use uuid::Uuid;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Medicines,
    Suppliers,
}
#[derive(Default)]
pub struct State {
    pub tab: Tab,
    pub medicine_query: String,
    pub medicines: Vec<Medicine>,
    pub supplier_query: String,
    pub suppliers: Vec<Supplier>,
    pub new_medicine: String,
    pub new_supplier: String,
    pub selected_medicine: Option<Uuid>,
    pub selling_price: f64,
    pub error: Option<String>,
    pub success: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Message {
    Tab(Tab),
    MedicineQuery(String),
    SupplierQuery(String),
    MedicineName(String),
    SupplierName(String),
    CreateMedicine,
    CreateSupplier,
    SelectMedicine(Uuid),
    Price(f64),
    SavePrice,
    Refresh,
}
pub fn refresh(state: &mut State, backend: &Backend) {
    let meds = backend.search_medicines(&state.medicine_query, 100);
    let suppliers = backend.search_suppliers(&state.supplier_query, 100);
    match (meds, suppliers) {
        (Ok(m), Ok(s)) => {
            state.medicines = m;
            state.suppliers = s;
            state.error = None;
        }
        _ => state.error = Some("Could not load catalog.".into()),
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
        Message::Tab(tab) => state.tab = tab,
        Message::MedicineQuery(q) => {
            state.medicine_query = q;
            match backend.search_medicines(&state.medicine_query, 100) {
                Ok(v) => state.medicines = v,
                Err(_) => state.error = Some("Could not search medicines.".into()),
            }
        }
        Message::SupplierQuery(q) => {
            state.supplier_query = q;
            match backend.search_suppliers(&state.supplier_query, 100) {
                Ok(v) => state.suppliers = v,
                Err(_) => state.error = Some("Could not search suppliers.".into()),
            }
        }
        Message::MedicineName(v) => state.new_medicine = v,
        Message::SupplierName(v) => state.new_supplier = v,
        Message::CreateMedicine => match required(&state.new_medicine) {
            Ok(name) => match backend.create_medicine(pharmacy, CreateMedicineInput { name }) {
                Ok(_) => {
                    state.new_medicine.clear();
                    state.success = Some("Medicine created.".into());
                    refresh(state, backend);
                }
                Err(_) => state.error = Some("Could not create medicine.".into()),
            },
            Err(e) => state.error = Some(e),
        },
        Message::CreateSupplier => match required(&state.new_supplier) {
            Ok(name) => match backend.create_supplier(pharmacy, CreateSupplierInput { name }) {
                Ok(_) => {
                    state.new_supplier.clear();
                    state.success = Some("Supplier created.".into());
                    refresh(state, backend);
                }
                Err(_) => {
                    state.error =
                        Some("Could not create supplier. Check for an existing name.".into())
                }
            },
            Err(e) => state.error = Some(e),
        },
        Message::SelectMedicine(id) => {
            state.selected_medicine = Some(id);
            state.selling_price = state
                .medicines
                .iter()
                .find(|m| m.id == id)
                .and_then(|m| m.selling_price)
                .map(format::decimal_as_f64)
                .unwrap_or(0.0);
        }
        Message::Price(v) => state.selling_price = v,
        Message::SavePrice => {
            let Some(medicine_id) = state.selected_medicine else {
                return false;
            };
            match decimal_from_ui(state.selling_price) {
                Ok(price) => match backend.set_selling_price(
                    pharmacy,
                    SetSellingPriceInput {
                        medicine_id,
                        selling_price: Some(price),
                    },
                ) {
                    Ok(()) => {
                        state.success = Some("Selling price updated.".into());
                        refresh(state, backend);
                    }
                    Err(e) => {
                        state.error = Some(match e {
                            Error::NotFound => "Medicine no longer exists.".into(),
                            _ => "Could not update selling price.".into(),
                        })
                    }
                },
                Err(e) => state.error = Some(e),
            }
        }
        Message::Refresh => refresh(state, backend),
    }
    false
}
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![
        page_header(
            "Catalog",
            "Manage medicines, suppliers, and selling prices",
            Some(
                button("Refresh")
                    .style(theme::button::secondary)
                    .on_press(Message::Refresh)
                    .into()
            )
        ),
        row![
            button("Medicines")
                .style(if state.tab == Tab::Medicines {
                    theme::button::primary
                } else {
                    theme::button::secondary
                })
                .on_press(Message::Tab(Tab::Medicines)),
            button("Suppliers")
                .style(if state.tab == Tab::Suppliers {
                    theme::button::primary
                } else {
                    theme::button::secondary
                })
                .on_press(Message::Tab(Tab::Suppliers))
        ]
        .spacing(theme::spacing::SM)
    ]
    .spacing(theme::spacing::LG);
    if let Some(e) = &state.error {
        content = content.push(error_banner(e));
    }
    if let Some(s) = &state.success {
        content = content.push(success_banner(s));
    }
    match state.tab {
        Tab::Medicines => {
            content = content
                .push(
                    row![
                        text_input("New medicine name", &state.new_medicine)
                            .style(theme::text::input)
                            .on_input(Message::MedicineName),
                        button("Create")
                            .style(theme::button::primary)
                            .on_press(Message::CreateMedicine)
                    ]
                    .spacing(theme::spacing::SM),
                )
                .push(
                    text_input("Search medicines", &state.medicine_query)
                        .style(theme::text::input)
                        .on_input(Message::MedicineQuery),
                );
            content = content.push(
                container(
                    row![
                        text("Medicine").width(Length::FillPortion(3)),
                        text("Reference price").width(Length::FillPortion(1)),
                        text("Selling price").width(Length::FillPortion(1)),
                        text("Status")
                    ]
                    .spacing(theme::spacing::MD),
                )
                .padding(theme::spacing::SM)
                .style(theme::container::table_header),
            );
            for medicine in &state.medicines {
                content = content.push(
                    container(
                        row![
                            column![
                                text(&medicine.name),
                                text(format!(
                                    "{} · {} · {} · {}",
                                    medicine.generic_name.as_deref().unwrap_or(""),
                                    medicine.strength.as_deref().unwrap_or(""),
                                    medicine.dosage_form.as_deref().unwrap_or(""),
                                    medicine.manufacturer.as_deref().unwrap_or("")
                                ))
                                .size(12)
                                .style(theme::text::muted)
                            ]
                            .width(Length::FillPortion(3)),
                            text(
                                medicine
                                    .reference_price
                                    .map(format::money)
                                    .unwrap_or_else(|| "—".into())
                            )
                            .width(Length::FillPortion(1)),
                            button(text(
                                medicine
                                    .selling_price
                                    .map(format::money)
                                    .unwrap_or_else(|| "Set price".into())
                            ))
                            .style(theme::button::ghost)
                            .width(Length::FillPortion(1))
                            .on_press(Message::SelectMedicine(medicine.id)),
                            status_badge::active(medicine.active)
                        ]
                        .spacing(theme::spacing::MD)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(theme::spacing::SM)
                    .style(theme::container::table_row),
                );
            }
            if state.selected_medicine.is_some() {
                content = content.push(
                    container(
                        column![
                            text("Set selling price").size(18),
                            NumberInput::new(
                                &state.selling_price,
                                0.0..=1_000_000_000.0,
                                Message::Price
                            )
                            .step(0.01),
                            button("Save price")
                                .style(theme::button::primary)
                                .on_press(Message::SavePrice)
                        ]
                        .spacing(theme::spacing::SM),
                    )
                    .padding(theme::spacing::LG)
                    .style(theme::container::card),
                );
            }
        }
        Tab::Suppliers => {
            content = content
                .push(
                    row![
                        text_input("New supplier name", &state.new_supplier)
                            .style(theme::text::input)
                            .on_input(Message::SupplierName),
                        button("Create")
                            .style(theme::button::primary)
                            .on_press(Message::CreateSupplier)
                    ]
                    .spacing(theme::spacing::SM),
                )
                .push(
                    text_input("Search suppliers", &state.supplier_query)
                        .style(theme::text::input)
                        .on_input(Message::SupplierQuery),
                );
            for supplier in &state.suppliers {
                content = content.push(
                    container(text(&supplier.name))
                        .padding(theme::spacing::MD)
                        .width(Length::Fill)
                        .style(theme::container::table_row),
                );
            }
        }
    }
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
