// Release builds are a windowed app; don't open a console window next to it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod format;
mod prefs;
mod route;
mod screen;
mod theme;
mod validation;
mod widget;
fn main() -> iced::Result {
    let mut fonts: Vec<std::borrow::Cow<'static, [u8]>> = theme::font::FILES
        .iter()
        .map(|bytes| std::borrow::Cow::Borrowed(*bytes))
        .collect();
    fonts.push(include_bytes!("../assets/lucide.ttf").as_slice().into());
    iced::application(app::App::boot, app::App::update, app::App::view)
        .title("Meditrino Pharmacy")
        .theme(|app: &app::App| theme::iced_theme(app.theme_mode))
        .settings(iced::Settings {
            fonts,
            default_font: theme::font::REGULAR,
            default_text_size: theme::typography::BODY.into(),
            antialiasing: true,
            ..Default::default()
        })
        .window(iced::window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            min_size: Some(iced::Size::new(960.0, 640.0)),
            ..Default::default()
        })
        .run()
}
