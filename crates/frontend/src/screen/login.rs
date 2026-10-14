use crate::{
    theme::{self, Tone, font, typography},
    widget::{
        feedback::error_banner,
        form,
        icon::{chip, icon},
    },
};
use iced::{
    Alignment, Element, Length,
    widget::{Space, button, column, container, row, text},
};
use lucide_icons::Icon;

#[derive(Default)]
pub struct State {
    pub username: String,
    pub password: String,
    pub error: Option<String>,
    pub loading: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Username(String),
    Password(String),
    Submit,
}

fn feature<'a>(glyph: Icon, title: &'a str, detail: &'a str) -> Element<'a, Message> {
    row![
        container(icon(glyph, 18.0))
            .center(40.0)
            .style(theme::container::segmented),
        column![
            text(title).font(font::SEMIBOLD).style(theme::text::sidebar),
            text(detail)
                .size(typography::SMALL)
                .style(theme::text::sidebar_muted)
        ]
        .spacing(2.0)
    ]
    .spacing(theme::spacing::MD)
    .align_y(Alignment::Center)
    .into()
}

fn brand_panel<'a>() -> Element<'a, Message> {
    let mark = container(icon(Icon::Pill, 22.0))
        .center(44.0)
        .style(theme::container::avatar);
    container(
        column![
            row![
                mark,
                text("Meditrino Pharmacy")
                    .size(typography::H2)
                    .font(font::BOLD)
                    .style(theme::text::sidebar)
            ]
            .spacing(theme::spacing::MD)
            .align_y(Alignment::Center),
            Space::new().height(Length::Fill),
            text("Run your pharmacy,\nonline or off.")
                .size(typography::DISPLAY + 4.0)
                .font(font::BOLD)
                .style(theme::text::sidebar),
            text(
                "Sales, stock and the medicine catalogue stay on this computer and sync when you're connected."
            )
            .size(typography::H3)
            .style(theme::text::sidebar_muted),
            Space::new().height(theme::spacing::XL),
            feature(
                Icon::WifiOff,
                "Works offline",
                "The full medicine catalogue is stored locally."
            ),
            feature(
                Icon::CalendarClock,
                "Expiry and stock alerts",
                "See low stock and near-expiry batches at a glance."
            ),
            feature(
                Icon::ShieldCheck,
                "Private by default",
                "Your pharmacy's records stay on this device."
            ),
            Space::new().height(Length::Fill),
        ]
        .spacing(theme::spacing::LG),
    )
    .padding(theme::spacing::XXL + 16.0)
    .width(Length::FillPortion(5))
    .height(Length::Fill)
    .style(theme::container::sidebar)
    .into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    let mut form = column![
        chip(Icon::LogIn, Tone::Primary, 48.0),
        Space::new().height(theme::spacing::SM),
        text("Welcome back").size(typography::H1).font(font::BOLD),
        text("Sign in to your pharmacy account.").style(theme::text::muted),
        Space::new().height(theme::spacing::MD),
        form::field(
            "Username",
            form::input("e.g. green-pharmacy", &state.username).on_input(Message::Username),
            None
        ),
        form::field(
            "Password",
            form::input("Your password", &state.password)
                .secure(true)
                .on_input(Message::Password)
                .on_submit(Message::Submit),
            None
        ),
    ]
    .spacing(theme::spacing::MD);
    if let Some(error) = &state.error {
        form = form.push(error_banner(error));
    }
    let label = if state.loading {
        "Signing in…"
    } else {
        "Sign in"
    };
    form = form.push(Space::new().height(theme::spacing::XS)).push(
        button(
            container(text(label).font(font::SEMIBOLD))
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .style(theme::button::primary)
        .on_press_maybe((!state.loading).then_some(Message::Submit))
        .width(Length::Fill)
        .height(theme::size::CONTROL_HEIGHT + 4.0),
    );
    let form_side = container(
        container(form)
            .padding(theme::spacing::XXL + 8.0)
            .width(420)
            .style(theme::container::card),
    )
    .center(Length::Fill)
    .width(Length::FillPortion(6))
    .style(theme::container::app);
    row![brand_panel(), form_side].height(Length::Fill).into()
}
