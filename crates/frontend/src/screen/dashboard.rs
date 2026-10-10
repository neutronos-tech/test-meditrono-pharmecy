use crate::{
    format,
    route::Route,
    theme,
    widget::{
        data_table::data_table, empty_state::empty_state, feedback::error_banner,
        metric_card::metric_card, page::page_header,
    },
};
use backend::Backend;
use contracts::{
    auth::PharmacyAccount,
    dashboard::Dashboard,
    history::{HistoryEntry, HistoryFilter},
};
use iced::{
    Element, Length,
    widget::{button, column, row, scrollable, text},
};
use lucide_icons::Icon;
#[derive(Default)]
pub struct State {
    pub summary: Option<Dashboard>,
    pub recent_history: Vec<HistoryEntry>,
    pub error: Option<String>,
}
#[derive(Debug, Clone)]
pub enum Message {
    Refresh,
    QuickAction(Route),
}
pub fn refresh(state: &mut State, backend: &Backend) {
    match (backend.dashboard(), backend.history(HistoryFilter::All, 8)) {
        (Ok(summary), Ok(history)) => {
            state.summary = Some(summary);
            state.recent_history = history;
            state.error = None;
        }
        _ => state.error = Some("Could not load the dashboard.".into()),
    }
}
pub fn view<'a>(state: &'a State, _pharmacy: &'a PharmacyAccount) -> Element<'a, Message> {
    let mut content = column![page_header(
        "Dashboard",
        "A snapshot of your pharmacy today",
        Some(
            button("Refresh")
                .style(theme::button::secondary)
                .on_press(Message::Refresh)
                .into()
        )
    )]
    .spacing(theme::spacing::XL);
    if let Some(error) = &state.error {
        content = content.push(error_banner(error));
    }
    if let Some(s) = &state.summary {
        content = content.push(
            row![
                metric_card(
                    Icon::Receipt,
                    "Today's sales",
                    format::money(s.today_sales_total),
                    "Bangladesh business day"
                ),
                metric_card(
                    Icon::ShoppingCart,
                    "Sales count",
                    s.today_sales_count.to_string(),
                    "Completed sales"
                ),
                metric_card(
                    Icon::Pill,
                    "Medicines in stock",
                    s.medicines_in_stock.to_string(),
                    "Physical stock"
                )
            ]
            .spacing(theme::spacing::MD),
        );
        content = content.push(
            row![
                metric_card(
                    Icon::CircleAlert,
                    "Low stock",
                    s.low_stock_medicines.to_string(),
                    "10 units or fewer"
                ),
                metric_card(
                    Icon::CalendarDays,
                    "Near expiry",
                    s.near_expiry_batches.to_string(),
                    "Next 30 days"
                )
            ]
            .spacing(theme::spacing::MD),
        );
    }
    content = content.push(text("Quick actions").size(19));
    let mut actions = row![].spacing(theme::spacing::SM);
    for (route, label) in [
        (Route::Purchase, "New purchase"),
        (Route::Stock, "Receive stock"),
        (Route::Sale, "New sale"),
    ] {
        actions = actions.push(
            button(label)
                .style(theme::button::secondary)
                .on_press(Message::QuickAction(route)),
        );
    }
    content = content.push(actions).push(text("Recent activity").size(19));
    if state.recent_history.is_empty() {
        content = content.push(empty_state(
            Icon::History,
            "No activity yet",
            "Transactions will appear here.",
            None,
        ));
    } else {
        let rows = state
            .recent_history
            .iter()
            .map(|e| {
                vec![
                    text(format::timestamp(e.occurred_at)).into(),
                    text(format!("{:?}", e.kind)).into(),
                    text(e.description.as_str()).into(),
                    text(e.actor_name.as_str()).into(),
                ]
            })
            .collect();
        content = content.push(data_table(
            &["Time", "Type", "Description", "User"],
            rows,
            None,
        ));
    }
    scrollable(content.padding(theme::spacing::XL).width(Length::Fill)).into()
}
