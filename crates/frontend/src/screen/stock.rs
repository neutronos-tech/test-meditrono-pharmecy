use crate::{
    theme,
    validation::required,
    widget::{
        empty_state::empty_state,
        feedback::{error_banner, success_banner},
        page::page_header,
    },
};
use backend::{Backend, Error};
use contracts::{
    auth::PharmacyAccount,
    stock::{
        ReceivablePurchase, ReceivablePurchaseDetail, StockReceiptInput, StockReceiptLineInput,
    },
};
use iced::{
    Element, Length,
    widget::{button, checkbox, column, container, row, scrollable, text},
};
use iced_aw::{DatePicker, NumberInput, date_picker};
use lucide_icons::Icon;
use time::{Date, Month, OffsetDateTime, UtcOffset};
use uuid::Uuid;
#[derive(Default)]
pub struct State {
    pub purchases: Vec<ReceivablePurchase>,
    pub selected: Option<ReceivablePurchaseDetail>,
    pub lines: Vec<ReceiptLineState>,
    pub error: Option<String>,
    pub success: Option<String>,
}
pub struct ReceiptLineState {
    pub purchase_item_id: Uuid,
    pub medicine_name: String,
    pub remaining: u32,
    pub selected: bool,
    pub quantity: u32,
    pub batch_number: String,
    pub expiry_date: date_picker::Date,
    pub date_picker_open: bool,
}
#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    SelectPurchase(Uuid),
    SelectLine(usize, bool),
    Quantity(usize, u32),
    Batch(usize, String),
    OpenDate(usize),
    CloseDate(usize),
    SetDate(usize, date_picker::Date),
    Submit,
}
fn bd_today() -> Date {
    OffsetDateTime::now_utc()
        .to_offset(UtcOffset::from_hms(6, 0, 0).unwrap_or(UtcOffset::UTC))
        .date()
}
fn to_picker(date: Date) -> date_picker::Date {
    date_picker::Date::from_ymd(
        date.year(),
        u8::from(date.month()).into(),
        date.day().into(),
    )
}
fn from_picker(date: date_picker::Date) -> Result<Date, String> {
    let month = Month::try_from(u8::try_from(date.month).map_err(|_| "Invalid month.")?)
        .map_err(|_| "Invalid month.")?;
    Date::from_calendar_date(
        date.year,
        month,
        u8::try_from(date.day).map_err(|_| "Invalid day.")?,
    )
    .map_err(|_| "Invalid date.".into())
}
impl State {
    fn build_input(&self) -> Result<StockReceiptInput, String> {
        let purchase_id = self
            .selected
            .as_ref()
            .ok_or("Select a purchase.")?
            .purchase_id;
        let mut lines = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for line in self.lines.iter().filter(|line| line.selected) {
            if !seen.insert(line.purchase_item_id) {
                return Err("A purchase line appears twice.".into());
            }
            if line.quantity == 0 || line.quantity > line.remaining {
                return Err("Quantity must be within the remaining amount.".into());
            }
            let batch_number = required(&line.batch_number)?;
            let expiry_date = from_picker(line.expiry_date)?;
            if expiry_date < bd_today() {
                return Err("Expiry cannot be before today in Bangladesh.".into());
            }
            lines.push(StockReceiptLineInput {
                purchase_item_id: line.purchase_item_id,
                quantity: line.quantity,
                batch_number,
                expiry_date,
            });
        }
        if lines.is_empty() {
            return Err("Select at least one line to receive.".into());
        }
        Ok(StockReceiptInput { purchase_id, lines })
    }
}
pub fn refresh(state: &mut State, backend: &Backend) {
    match backend.receivable_purchases() {
        Ok(v) => {
            state.purchases = v;
            state.error = None;
        }
        Err(_) => state.error = Some("Could not load receivable purchases.".into()),
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
        Message::Refresh => refresh(state, backend),
        Message::SelectPurchase(id) => match backend.receivable_purchase(id) {
            Ok(detail) => {
                let default_date = to_picker(bd_today() + time::Duration::days(30));
                state.lines = detail
                    .items
                    .iter()
                    .map(|item| ReceiptLineState {
                        purchase_item_id: item.purchase_item_id,
                        medicine_name: item.medicine_name.clone(),
                        remaining: item.remaining,
                        selected: false,
                        quantity: 1,
                        batch_number: String::new(),
                        expiry_date: default_date,
                        date_picker_open: false,
                    })
                    .collect();
                state.selected = Some(detail);
            }
            Err(_) => state.error = Some("Could not load purchase.".into()),
        },
        Message::SelectLine(i, value) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.selected = value;
            }
        }
        Message::Quantity(i, value) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.quantity = value;
            }
        }
        Message::Batch(i, value) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.batch_number = value;
            }
        }
        Message::OpenDate(i) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.date_picker_open = true;
            }
        }
        Message::CloseDate(i) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.date_picker_open = false;
            }
        }
        Message::SetDate(i, value) => {
            if let Some(line) = state.lines.get_mut(i) {
                line.expiry_date = value;
                line.date_picker_open = false;
            }
        }
        Message::Submit => match state.build_input() {
            Ok(input) => match backend.receive_stock(pharmacy, input) {
                Ok(_) => {
                    state.success = Some("Stock received.".into());
                    state.selected = None;
                    state.lines.clear();
                    refresh(state, backend);
                }
                Err(e) => {
                    state.error = Some(match e {
                        Error::ReceiptExceedsPurchase => {
                            "Received quantity exceeds remaining purchase quantity.".into()
                        }
                        Error::BatchMetadataConflict => {
                            "This batch number already has a different expiry date.".into()
                        }
                        _ => "Could not receive stock. Review the lines and try again.".into(),
                    });
                }
            },
            Err(e) => state.error = Some(e),
        },
    }
    false
}
pub fn view(state: &State) -> Element<'_, Message> {
    let mut content = column![page_header(
        "Stock intake",
        "Receive purchased medicines into batches",
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
    if state.purchases.is_empty() {
        content = content.push(empty_state(
            Icon::PackagePlus,
            "No purchases to receive",
            "Create a purchase to start receiving stock.",
            None,
        ));
    } else {
        content = content.push(text("Receivable purchases").size(17));
        for purchase in &state.purchases {
            content = content.push(
                button(text(format!(
                    "{} · {} · {} lines",
                    purchase.invoice_number, purchase.supplier_name, purchase.remaining_items
                )))
                .style(theme::button::secondary)
                .padding(theme::size::BUTTON_PADDING)
                .width(Length::Fill)
                .on_press(Message::SelectPurchase(purchase.purchase_id)),
            );
        }
    }
    if let Some(detail) = &state.selected {
        content = content.push(text(format!("Receive: {}", detail.invoice_number)).size(18));
        for (index, line) in state.lines.iter().enumerate() {
            let picker = DatePicker::new(
                line.date_picker_open,
                line.expiry_date,
                button(text(line.expiry_date.to_string()))
                    .style(theme::button::secondary)
                    .padding(theme::size::BUTTON_PADDING)
                    .on_press(Message::OpenDate(index)),
                Message::CloseDate(index),
                move |date| Message::SetDate(index, date),
            );
            content = content.push(
                container(
                    column![
                        row![
                            checkbox(line.selected)
                                .label(&line.medicine_name)
                                .on_toggle(move |v| Message::SelectLine(index, v)),
                            text(format!("Remaining: {}", line.remaining))
                                .style(theme::text::muted)
                        ]
                        .spacing(theme::spacing::MD),
                        row![
                            text("Quantity"),
                            NumberInput::new(&line.quantity, 1..=line.remaining.max(1), move |n| {
                                Message::Quantity(index, n)
                            })
                            .width(110),
                            text("Batch"),
                            crate::widget::form::input("Batch number", &line.batch_number)
                                .on_input(move |v| Message::Batch(index, v)),
                            text("Expiry"),
                            picker
                        ]
                        .spacing(theme::spacing::SM)
                        .align_y(iced::Alignment::Center)
                    ]
                    .spacing(theme::spacing::MD),
                )
                .padding(theme::spacing::LG)
                .style(theme::container::card),
            );
        }
        content = content.push(
            button("Receive selected stock")
                .style(theme::button::primary)
                .padding(theme::size::BUTTON_PADDING)
                .on_press(Message::Submit),
        );
    }
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
