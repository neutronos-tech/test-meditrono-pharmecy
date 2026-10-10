mod app;
mod format;
mod route;
mod screen;
mod theme;
mod validation;
mod widget;
fn main() -> iced::Result {
    iced::application(app::App::boot, app::App::update, app::App::view)
        .theme(|app: &app::App| theme::iced_theme(app.theme_mode))
        .window(iced::window::Settings {
            size: iced::Size::new(1280.0, 800.0),
            min_size: Some(iced::Size::new(960.0, 640.0)),
            ..Default::default()
        })
        .font(include_bytes!("../assets/lucide.ttf").as_slice())
        .run()
}
