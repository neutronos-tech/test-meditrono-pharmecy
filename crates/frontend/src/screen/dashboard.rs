use crate::{
    format,
    route::Route,
    theme::{self, Tone, typography},
    widget::{
        chart::{Bar, bar_chart, ring},
        data_table::data_table,
        empty_state::empty_state,
        feedback::error_banner,
        icon::icon,
        metric_card::{metric_card, toned_metric_card},
        page::{page_header, section_title},
        status_badge,
    },
};
use backend::Backend;
use contracts::{
    auth::PharmacyAccount,
    dashboard::Dashboard,
    history::{HistoryEntry, HistoryFilter},
};
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, scrollable, text},
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
/// The sales chart card: one bar per day, today last.
fn sales_week<'a>(summary: &Dashboard) -> Element<'a, Message> {
    let bars = summary
        .daily_sales
        .iter()
        .map(|day| Bar {
            label: format!("{:.3}", day.date.weekday()),
            value: format::decimal_as_f64(day.total),
            caption: format::compact_money(day.total),
        })
        .collect();
    let week_total: rust_decimal::Decimal = summary.daily_sales.iter().map(|day| day.total).sum();
    container(
        column![
            row![
                section_title("Sales this week"),
                Space::new().width(Length::Fill),
                status_badge::badge(format!("Total {}", format::money(week_total)), Tone::Accent),
            ]
            .align_y(Alignment::Center),
            bar_chart(bars, 220.0),
        ]
        .spacing(theme::spacing::LG),
    )
    .padding(theme::spacing::XL)
    .width(Length::FillPortion(2))
    .style(theme::container::card)
    .into()
}

/// Share of stocked medicines above the low-stock line, plus the two risks.
fn stock_health<'a>(summary: &Dashboard) -> Element<'a, Message> {
    let stocked = summary.medicines_in_stock;
    let healthy = if stocked == 0 {
        0.0
    } else {
        stocked.saturating_sub(summary.low_stock_medicines) as f32 / stocked as f32
    };
    let risk = |value: u32, label: &'static str, tone: Tone| {
        row![
            text(value.to_string()).size(typography::H1),
            column![
                text(label).size(typography::SMALL),
                status_badge::badge(
                    if value == 0 {
                        "All clear"
                    } else {
                        "Needs attention"
                    },
                    if value == 0 { Tone::Success } else { tone }
                )
            ]
            .spacing(2.0)
        ]
        .spacing(theme::spacing::MD)
        .align_y(Alignment::Center)
    };
    container(
        column![
            section_title("Stock health"),
            container(ring(
                healthy,
                if stocked == 0 {
                    "No stock yet"
                } else {
                    "Well stocked"
                }
                .into(),
                170.0
            ))
            .center_x(Length::Fill),
            risk(
                summary.low_stock_medicines,
                "Low stock medicines",
                Tone::Warning
            ),
            risk(
                summary.near_expiry_batches,
                "Batches near expiry",
                Tone::Danger
            ),
        ]
        .spacing(theme::spacing::LG),
    )
    .padding(theme::spacing::XL)
    .width(Length::FillPortion(1))
    .style(theme::container::card)
    .into()
}

pub fn view<'a>(state: &'a State, _pharmacy: &'a PharmacyAccount) -> Element<'a, Message> {
    let mut content = column![page_header(
        "Dashboard",
        "A snapshot of your pharmacy today",
        Some(
            button(
                row![icon(Icon::RefreshCw, 15.0), text("Refresh")]
                    .spacing(theme::spacing::SM)
                    .align_y(Alignment::Center)
            )
            .style(theme::button::secondary)
            .padding(theme::size::BUTTON_PADDING)
            .on_press(Message::Refresh)
            .into()
        )
    )]
    .spacing(theme::spacing::XL);
    if let Some(error) = &state.error {
        content = content.push(error_banner(error));
    }
    if let Some(s) = &state.summary {
        content = content
            .push(
                row![
                    metric_card(
                        Icon::Banknote,
                        "Today's sales",
                        format::money(s.today_sales_total),
                        "Bangladesh business day"
                    ),
                    metric_card(
                        Icon::ShoppingCart,
                        "Sales count",
                        s.today_sales_count.to_string(),
                        "Completed today"
                    ),
                    metric_card(
                        Icon::Pill,
                        "Medicines in stock",
                        s.medicines_in_stock.to_string(),
                        "With physical stock"
                    ),
                    toned_metric_card(
                        Icon::CalendarClock,
                        Tone::Accent,
                        "Near expiry",
                        s.near_expiry_batches.to_string(),
                        "Batches in the next 30 days"
                    ),
                ]
                .spacing(theme::spacing::LG),
            )
            .push(row![sales_week(s), stock_health(s)].spacing(theme::spacing::LG));
    }
    content = content.push(section_title("Quick actions"));
    let mut actions = row![].spacing(theme::spacing::SM);
    for (route, label) in [
        (Route::Purchase, "New purchase"),
        (Route::Stock, "Receive stock"),
        (Route::Sale, "New sale"),
    ] {
        actions = actions.push(
            button(label)
                .style(theme::button::secondary)
                .padding(theme::size::BUTTON_PADDING)
                .on_press(Message::QuickAction(route)),
        );
    }
    content = content.push(actions).push(section_title("Recent activity"));
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
    scrollable(content.padding(theme::spacing::XXL).width(Length::Fill)).into()
}
