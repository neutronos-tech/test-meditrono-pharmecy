use crate::{theme, widget::page::page_header};
use contracts::auth::PharmacyAccount;
use iced::{
    Element, Length,
    widget::{column, scrollable, text},
};

#[derive(Default)]
pub struct State;

#[derive(Debug, Clone)]
pub enum Message {}

pub fn refresh(_state: &mut State) {}

pub fn view<'a>(pharmacy: &'a PharmacyAccount) -> Element<'a, Message> {
    scrollable(
        column![
            page_header("Settings", "Your connected pharmacy account", None),
            text("Signed in as").style(theme::text::muted),
            text(&pharmacy.username).size(20),
        ]
        .spacing(theme::spacing::MD)
        .padding(theme::spacing::XL)
        .width(Length::Fill),
    )
    .into()
}
