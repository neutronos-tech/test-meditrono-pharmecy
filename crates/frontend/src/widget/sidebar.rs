use crate::{
    app::Message,
    route::Route,
    theme::{self, ThemeMode},
    widget::icon::icon,
};
use contracts::auth::PharmacyAccount;
use iced::{
    Element, Length,
    widget::{Space, button, column, container, row, rule, text},
};
use lucide_icons::Icon;
fn route_icon(route: Route) -> Icon {
    match route {
        Route::Dashboard => Icon::LayoutDashboard,
        Route::Purchase => Icon::Receipt,
        Route::Stock => Icon::PackagePlus,
        Route::Inventory => Icon::Boxes,
        Route::Sale => Icon::ShoppingCart,
        Route::History => Icon::History,
        Route::Catalog => Icon::Pill,
        Route::Settings => Icon::Settings,
    }
}
pub fn view<'a>(
    pharmacy: &'a PharmacyAccount,
    active: Route,
    compact: bool,
    mode: ThemeMode,
) -> Element<'a, Message> {
    let width = if compact {
        72.0
    } else {
        theme::size::SIDEBAR_WIDTH
    };
    let mut nav = column![
        row![
            icon(Icon::Pill, 22.0),
            if compact {
                text("")
            } else {
                text("Pharmacy").size(20)
            }
        ]
        .spacing(theme::spacing::MD)
        .align_y(iced::Alignment::Center)
    ]
    .spacing(theme::spacing::SM);
    for route in Route::ALL {
        if route == Route::Catalog {
            nav = nav.push(Space::new().height(theme::spacing::XL));
        }
        let label = if compact { "" } else { route.label() };
        let style = if route == active {
            theme::button::sidebar_active
        } else {
            theme::button::sidebar
        };
        nav = nav.push(
            button(
                row![icon(route_icon(route), 18.0), text(label)]
                    .spacing(theme::spacing::MD)
                    .align_y(iced::Alignment::Center),
            )
            .style(style)
            .width(Length::Fill)
            .on_press(Message::Navigate(route)),
        );
    }
    let light = button(
        row![
            icon(Icon::Sun, 16.0),
            text(if compact { "" } else { "Light" })
        ]
        .spacing(theme::spacing::XS),
    )
    .style(if mode == ThemeMode::Light {
        theme::button::sidebar_active
    } else {
        theme::button::ghost
    })
    .on_press(Message::SetTheme(ThemeMode::Light));
    let dark = button(
        row![
            icon(Icon::Moon, 16.0),
            text(if compact { "" } else { "Dark" })
        ]
        .spacing(theme::spacing::XS),
    )
    .style(if mode == ThemeMode::Dark {
        theme::button::sidebar_active
    } else {
        theme::button::ghost
    })
    .on_press(Message::SetTheme(ThemeMode::Dark));
    let theme_picker: Element<'_, Message> = if compact {
        column![light, dark].spacing(theme::spacing::XS).into()
    } else {
        row![light, dark].spacing(theme::spacing::XS).into()
    };
    let bottom = column![
        rule::horizontal(1),
        if compact {
            text("")
        } else {
            text(&pharmacy.username).size(13)
        },
        theme_picker,
        button(
            row![
                icon(Icon::LogOut, 18.0),
                text(if compact { "" } else { "Logout" })
            ]
            .spacing(theme::spacing::MD)
        )
        .style(theme::button::ghost)
        .on_press(Message::Logout)
    ]
    .spacing(theme::spacing::SM);
    container(column![nav, Space::new().height(Length::Fill), bottom].spacing(theme::spacing::LG))
        .padding(if compact {
            theme::spacing::SM
        } else {
            theme::spacing::LG
        })
        .width(width)
        .height(Length::Fill)
        .style(theme::container::sidebar)
        .into()
}
