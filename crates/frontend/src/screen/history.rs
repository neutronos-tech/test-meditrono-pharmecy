use crate::{
    format, theme,
    widget::{
        data_table::data_table,
        empty_state::empty_state,
        feedback::{error_banner, success_banner},
        page::page_header,
    },
};
use backend::{Backend, Error};
use contracts::{
    auth::PharmacyAccount,
    history::{HistoryEntry, HistoryFilter, OperationKind},
};
use iced::{
    Element, Length,
    widget::{button, column, container, row, scrollable, text},
};
use lucide_icons::Icon;
use uuid::Uuid;
pub struct State {
    pub filter: HistoryFilter,
    pub entries: Vec<HistoryEntry>,
    pub confirm_reversal: Option<Uuid>,
    pub error: Option<String>,
    pub success: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            filter: HistoryFilter::All,
            entries: Vec::new(),
            confirm_reversal: None,
            error: None,
            success: None,
        }
    }
}
#[derive(Debug, Clone)]
pub enum Message {
    Filter(HistoryFilter),
    Refresh,
    Confirm(Uuid),
    Cancel,
    Reverse,
}
pub fn refresh(state: &mut State, backend: &Backend) {
    match backend.history(state.filter, 100) {
        Ok(entries) => {
            state.entries = entries;
            state.error = None;
        }
        Err(_) => state.error = Some("Could not load history.".into()),
    }
}
pub fn update(
    state: &mut State,
    message: Message,
    backend: &mut Backend,
    pharmacy: &PharmacyAccount,
) -> bool {
    match message {
        Message::Filter(filter) => {
            state.filter = filter;
            refresh(state, backend)
        }
        Message::Refresh => refresh(state, backend),
        Message::Confirm(id) => state.confirm_reversal = Some(id),
        Message::Cancel => state.confirm_reversal = None,
        Message::Reverse => {
            let Some(id) = state.confirm_reversal.take() else {
                return false;
            };
            let Some(entry) = state.entries.iter().find(|e| e.operation_id == id) else {
                return false;
            };
            let result = match entry.kind {
                OperationKind::StockReceipt => backend.reverse_stock_receipt(pharmacy, id),
                OperationKind::Sale => backend.reverse_sale(pharmacy, id),
                _ => return false,
            };
            match result {
                Ok(_) => {
                    state.success =
                        Some("Reversal recorded. Original history remains available.".into());
                    refresh(state, backend);
                }
                Err(e) => {
                    state.error=Some(match e {Error::StockWouldGoNegative=>"This receipt cannot be reversed because some of this stock has already been consumed.".into(),Error::AlreadyReversed=>"This operation was already reversed.".into(),_=>"Could not reverse operation.".into()});
                }
            }
        }
    }
    false
}
pub fn view<'a>(state: &'a State, _pharmacy: &'a PharmacyAccount) -> Element<'a, Message> {
    let mut content = column![page_header(
        "History",
        "Business operations and reversals",
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
    let filters = [
        (HistoryFilter::All, "All"),
        (HistoryFilter::Purchases, "Purchases"),
        (HistoryFilter::Stock, "Stock"),
        (HistoryFilter::Sales, "Sales"),
        (HistoryFilter::Reversals, "Reversals"),
    ];
    let mut tabs = row![].spacing(theme::spacing::SM);
    for (filter, label) in filters {
        tabs = tabs.push(
            button(label)
                .style(if state.filter == filter {
                    theme::button::primary
                } else {
                    theme::button::secondary
                })
                .on_press(Message::Filter(filter)),
        );
    }
    content = content.push(tabs);
    if state.entries.is_empty() {
        content = content.push(empty_state(
            Icon::History,
            "No history",
            "Operations will appear here.",
            None,
        ));
    }
    let rows = state
        .entries
        .iter()
        .map(|entry| {
            let action: Element<'_, Message> = if entry.reversible
                && matches!(
                    entry.kind,
                    OperationKind::StockReceipt | OperationKind::Sale
                ) {
                button("Reverse")
                    .style(theme::button::danger)
                    .padding(theme::size::BUTTON_PADDING)
                    .on_press(Message::Confirm(entry.operation_id))
                    .into()
            } else {
                text("—").style(theme::text::muted).into()
            };
            vec![
                text(format::timestamp(entry.occurred_at)).into(),
                text(format!("{:?}", entry.kind)).into(),
                text(entry.description.as_str()).into(),
                text(entry.actor_name.as_str()).into(),
                action,
            ]
        })
        .collect();
    if !state.entries.is_empty() {
        content = content.push(data_table(
            &["Time", "Type", "Description", "User", "Action"],
            rows,
            None,
        ));
    }
    if state.confirm_reversal.is_some() {
        content = content.push(
            container(
                column![
                    text("Reverse this operation?").size(19),
                    text("This creates a compensating operation. The original history remains."),
                    row![
                        button("Cancel")
                            .style(theme::button::secondary)
                            .padding(theme::size::BUTTON_PADDING)
                            .on_press(Message::Cancel),
                        button("Confirm reversal")
                            .style(theme::button::danger)
                            .padding(theme::size::BUTTON_PADDING)
                            .on_press(Message::Reverse)
                    ]
                    .spacing(theme::spacing::MD)
                ]
                .spacing(theme::spacing::MD),
            )
            .padding(theme::spacing::LG)
            .style(theme::container::modal),
        );
    }
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
