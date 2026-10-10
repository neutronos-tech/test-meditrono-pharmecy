use crate::{theme, widget::feedback::error_banner};
use iced::{
    Element, Length,
    widget::{Space, button, column, container, text, text_input},
};

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

pub fn view(state: &State) -> Element<'_, Message> {
    let mut form = column![
        text("Welcome back").size(30),
        text("Sign in to your pharmacy account.").style(theme::text::muted),
        Space::new().height(theme::spacing::MD),
        text("Username"),
        text_input("Username", &state.username)
            .style(theme::text::input)
            .on_input(Message::Username),
        text("Password"),
        text_input("Password", &state.password)
            .style(theme::text::input)
            .secure(true)
            .on_input(Message::Password)
            .on_submit(Message::Submit)
    ]
    .spacing(theme::spacing::SM);
    if let Some(error) = &state.error {
        form = form.push(error_banner(error));
    }
    form = form.push(
        button(if state.loading {
            "Signing in…"
        } else {
            "Sign in"
        })
        .style(theme::button::primary)
        .on_press_maybe((!state.loading).then_some(Message::Submit))
        .width(Length::Fill)
        .height(theme::size::CONTROL_HEIGHT),
    );
    container(
        container(form)
            .padding(theme::spacing::XXL)
            .width(420)
            .style(theme::container::card),
    )
    .center(Length::Fill)
    .into()
}
