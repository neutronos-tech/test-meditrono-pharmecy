use super::{palette, radius};
use iced::{Border, Theme, widget::container};
pub fn app(theme: &Theme) -> container::Style {
    container::Style::default().background(theme.palette().background)
}
pub fn sidebar(theme: &Theme) -> container::Style {
    container::Style::default()
        .background(palette::surface(theme))
        .border(Border {
            color: palette::border(theme),
            width: 1.0,
            radius: 0.0.into(),
        })
}
pub fn surface(theme: &Theme) -> container::Style {
    container::Style::default().background(palette::surface(theme))
}
pub fn card(theme: &Theme) -> container::Style {
    surface(theme).border(Border {
        color: palette::border(theme),
        width: 1.0,
        radius: radius::MD.into(),
    })
}
pub fn table_header(theme: &Theme) -> container::Style {
    container::Style::default().background(theme.extended_palette().background.strong.color)
}
pub fn table_row(theme: &Theme) -> container::Style {
    container::Style::default().border(Border {
        color: palette::border(theme),
        width: 0.5,
        radius: 0.0.into(),
    })
}
pub fn modal(theme: &Theme) -> container::Style {
    card(theme).border(Border {
        color: palette::border(theme),
        width: 1.0,
        radius: radius::LG.into(),
    })
}
