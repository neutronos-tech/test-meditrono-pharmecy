use iced::widget::Text;
use lucide_icons::Icon;
pub fn icon(icon: Icon, size: f32) -> Text<'static> {
    icon.widget::<iced::Theme>().size(size)
}
