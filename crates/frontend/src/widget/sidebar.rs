use crate::{
    app::Message,
    route::Route,
    theme::{self, ThemeMode, font, typography},
    widget::icon::icon,
};
use contracts::auth::PharmacyAccount;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{Column, Space, button, column, container, row, text},
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

/// Navigation groups, top to bottom. Settings sits with the account controls.
const SECTIONS: [(&str, &[Route]); 3] = [
    ("Overview", &[Route::Dashboard]),
    (
        "Operations",
        &[Route::Sale, Route::Purchase, Route::Stock, Route::Inventory],
    ),
    ("Records", &[Route::History, Route::Catalog]),
];

fn nav_item<'a>(route: Route, active: Route, compact: bool) -> Element<'a, Message> {
    let glyph = icon(route_icon(route), 18.0);
    let content: Element<'a, Message> = if compact {
        container(glyph).center_x(Length::Fill).into()
    } else {
        row![glyph, text(route.label()).font(font::MEDIUM)]
            .spacing(theme::spacing::MD)
            .align_y(Alignment::Center)
            .into()
    };
    button(content)
        .style(if route == active {
            theme::button::sidebar_active
        } else {
            theme::button::sidebar
        })
        .padding(Padding {
            top: theme::spacing::MD - 1.0,
            bottom: theme::spacing::MD - 1.0,
            left: theme::spacing::LG + 2.0,
            right: theme::spacing::LG,
        })
        .width(Length::Fill)
        .on_press(Message::Navigate(route))
        .into()
}

fn brand<'a>(compact: bool) -> Element<'a, Message> {
    let mark = container(icon(Icon::Pill, 18.0))
        .center(36.0)
        .style(theme::container::avatar);
    if compact {
        return container(mark).center_x(Length::Fill).into();
    }
    row![
        mark,
        column![
            text("Meditrino")
                .size(typography::H3 + 1.0)
                .font(font::BOLD)
                .style(theme::text::sidebar),
            text("Pharmacy")
                .size(typography::CAPTION)
                .style(theme::text::sidebar_muted)
        ]
    ]
    .spacing(theme::spacing::MD)
    .align_y(Alignment::Center)
    .into()
}

fn theme_switch<'a>(mode: ThemeMode, compact: bool) -> Element<'a, Message> {
    let option = |target: ThemeMode, glyph: Icon, label: &'static str| {
        let content: Element<'a, Message> = if compact {
            container(icon(glyph, 15.0)).center_x(Length::Fill).into()
        } else {
            row![icon(glyph, 15.0), text(label).size(typography::SMALL)]
                .spacing(theme::spacing::XS + 2.0)
                .align_y(Alignment::Center)
                .into()
        };
        container(
            button(content)
                .style(if mode == target {
                    theme::button::segment_active
                } else {
                    theme::button::segment
                })
                .padding([theme::spacing::XS + 2.0, theme::spacing::SM])
                .width(Length::Fill)
                .on_press(Message::SetTheme(target)),
        )
        .width(Length::Fill)
    };
    let light = option(ThemeMode::Light, Icon::Sun, "Light");
    let dark = option(ThemeMode::Dark, Icon::Moon, "Dark");
    let options: Element<'a, Message> = if compact {
        column![light, dark].spacing(2.0).into()
    } else {
        row![light, dark].spacing(2.0).into()
    };
    container(options)
        .padding(3.0)
        .width(Length::Fill)
        .style(theme::container::segmented)
        .into()
}

fn account<'a>(pharmacy: &'a PharmacyAccount, compact: bool) -> Element<'a, Message> {
    let initial = pharmacy
        .username
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let avatar = container(text(initial).font(font::SEMIBOLD))
        .center(34.0)
        .style(theme::container::avatar);
    if compact {
        return container(avatar).center_x(Length::Fill).into();
    }
    let card = row![
        avatar,
        column![
            text(&pharmacy.username)
                .font(font::SEMIBOLD)
                .style(theme::text::sidebar),
            text("Pharmacy account")
                .size(typography::CAPTION)
                .style(theme::text::sidebar_muted)
        ]
    ]
    .spacing(theme::spacing::MD)
    .align_y(Alignment::Center);
    container(card)
        .padding(theme::spacing::MD)
        .width(Length::Fill)
        .style(theme::container::segmented)
        .into()
}

pub fn view<'a>(
    pharmacy: &'a PharmacyAccount,
    active: Route,
    compact: bool,
    mode: ThemeMode,
) -> Element<'a, Message> {
    let width = if compact {
        theme::size::SIDEBAR_COMPACT_WIDTH
    } else {
        theme::size::SIDEBAR_WIDTH
    };
    let mut nav = Column::new().spacing(2.0);
    for (index, (title, routes)) in SECTIONS.into_iter().enumerate() {
        if index > 0 {
            nav = nav.push(Space::new().height(theme::spacing::MD));
        }
        if !compact {
            nav = nav.push(
                container(
                    text(title.to_uppercase())
                        .size(11.0)
                        .font(font::SEMIBOLD)
                        .style(theme::text::sidebar_muted),
                )
                .padding([theme::spacing::XS, theme::spacing::MD]),
            );
        }
        for route in routes {
            nav = nav.push(nav_item(*route, active, compact));
        }
    }
    let logout = button(if compact {
        Element::from(container(icon(Icon::LogOut, 18.0)).center_x(Length::Fill))
    } else {
        row![
            icon(Icon::LogOut, 18.0),
            text("Sign out").font(font::MEDIUM)
        ]
        .spacing(theme::spacing::MD)
        .align_y(Alignment::Center)
        .into()
    })
    .style(theme::button::sidebar)
    .padding([theme::spacing::SM + 1.0, theme::spacing::MD])
    .width(Length::Fill)
    .on_press(Message::Logout);
    // Everything but the nav keeps a right margin; nav items run to the panel edge.
    let right = if compact {
        theme::spacing::SM
    } else {
        theme::spacing::LG
    };
    let inset = |content: Element<'a, Message>| {
        container(content).padding(Padding {
            right,
            ..Padding::ZERO
        })
    };
    let footer = column![
        theme_switch(mode, compact),
        Space::new().height(theme::spacing::SM),
        account(pharmacy, compact),
    ];
    let bottom = column![
        nav_item(Route::Settings, active, compact),
        inset(logout.into()),
        Space::new().height(theme::spacing::SM),
        inset(footer.into()),
    ]
    .spacing(2.0);
    container(
        column![
            inset(brand(compact)),
            Space::new().height(theme::spacing::XL),
            nav,
            Space::new().height(Length::Fill),
            bottom
        ]
        .height(Length::Fill),
    )
    .padding(Padding {
        top: theme::spacing::XL,
        bottom: theme::spacing::LG,
        left: if compact {
            theme::spacing::SM
        } else {
            theme::spacing::LG
        },
        right: 0.0,
    })
    .width(width)
    .height(Length::Fill)
    .style(theme::container::sidebar)
    .into()
}
