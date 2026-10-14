//! Small canvas charts for the dashboard, drawn with the theme's tokens.

use crate::theme::{font, palette::tokens, typography};
use iced::{
    Element, Length, Point, Radians, Rectangle, Renderer, Size, Theme, alignment, mouse,
    widget::canvas::{self, Frame, Geometry, LineCap, Path, Stroke, Text, path::Arc},
};

/// One bar: its label under the axis and its value.
pub struct Bar {
    pub label: String,
    pub value: f64,
    /// Shown above the bar.
    pub caption: String,
}

/// Rounded vertical bars; the last bar (today) is highlighted.
pub struct BarChart {
    bars: Vec<Bar>,
}

pub fn bar_chart<'a, Message: 'a>(bars: Vec<Bar>, height: f32) -> Element<'a, Message> {
    canvas::Canvas::new(BarChart { bars })
        .width(Length::Fill)
        .height(height)
        .into()
}

impl<Message> canvas::Program<Message> for BarChart {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let t = tokens(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let label_band = 22.0;
        let caption_band = 20.0;
        let plot_height = (bounds.height - label_band - caption_band).max(1.0);
        let max = self
            .bars
            .iter()
            .map(|bar| bar.value)
            .fold(0.0_f64, f64::max);
        let count = self.bars.len().max(1) as f32;
        let slot = bounds.width / count;
        let bar_width = (slot * 0.42).clamp(10.0, 36.0);
        // Faint guide lines behind the bars.
        for step in 0..=3 {
            let y = caption_band + plot_height * step as f32 / 3.0;
            frame.stroke(
                &Path::line(Point::new(0.0, y), Point::new(bounds.width, y)),
                Stroke::default().with_color(t.border).with_width(1.0),
            );
        }
        let last = self.bars.len().saturating_sub(1);
        for (index, bar) in self.bars.iter().enumerate() {
            let share = if max > 0.0 { bar.value / max } else { 0.0 } as f32;
            // Empty days still show a short stub so the week reads as a row.
            let height = (plot_height * share).max(6.0);
            let x = slot * index as f32 + (slot - bar_width) / 2.0;
            let y = caption_band + plot_height - height;
            let today = index == last;
            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(x, y),
                    Size::new(bar_width, height),
                    (bar_width / 2.0).min(10.0).into(),
                ),
                if today { t.primary } else { t.accent_soft },
            );
            let center = x + bar_width / 2.0;
            frame.fill_text(Text {
                content: bar.caption.clone(),
                position: Point::new(center, y - 6.0),
                color: if today { t.primary } else { t.accent },
                size: typography::CAPTION.into(),
                font: font::MEDIUM,
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Bottom,
                ..Text::default()
            });
            frame.fill_text(Text {
                content: bar.label.clone(),
                position: Point::new(center, bounds.height - label_band / 2.0),
                color: if today { t.text } else { t.text_muted },
                size: typography::CAPTION.into(),
                font: if today { font::SEMIBOLD } else { font::REGULAR },
                align_x: alignment::Horizontal::Center.into(),
                align_y: alignment::Vertical::Center,
                ..Text::default()
            });
        }
        vec![frame.into_geometry()]
    }
}

/// A progress ring with a percentage and label in the middle.
pub struct Ring {
    fraction: f32,
    label: String,
}

pub fn ring<'a, Message: 'a>(fraction: f32, label: String, size: f32) -> Element<'a, Message> {
    canvas::Canvas::new(Ring {
        fraction: fraction.clamp(0.0, 1.0),
        label,
    })
    .width(size)
    .height(size)
    .into()
}

impl<Message> canvas::Program<Message> for Ring {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let t = tokens(theme);
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let thickness = 9.0;
        let radius = bounds.width.min(bounds.height) / 2.0 - thickness;
        frame.fill(&Path::circle(center, radius - thickness), t.surface_alt);
        frame.stroke(
            &Path::circle(center, radius),
            Stroke::default()
                .with_color(t.accent_soft)
                .with_width(thickness),
        );
        if self.fraction > 0.0 {
            let start = -std::f32::consts::FRAC_PI_2;
            let arc = Path::new(|p| {
                p.arc(Arc {
                    center,
                    radius,
                    start_angle: Radians(start),
                    end_angle: Radians(start + std::f32::consts::TAU * self.fraction),
                })
            });
            frame.stroke(
                &arc,
                Stroke::default()
                    .with_color(t.accent)
                    .with_width(thickness)
                    .with_line_cap(LineCap::Round),
            );
        }
        frame.fill_text(Text {
            content: format!("{:.0}%", self.fraction * 100.0),
            position: Point::new(center.x, center.y - 6.0),
            color: t.text,
            size: (radius * 0.48).into(),
            font: font::REGULAR,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..Text::default()
        });
        frame.fill_text(Text {
            content: self.label.to_uppercase(),
            position: Point::new(center.x, center.y + radius * 0.32),
            color: t.accent,
            size: 11.0.into(),
            font: font::MEDIUM,
            align_x: alignment::Horizontal::Center.into(),
            align_y: alignment::Vertical::Center,
            ..Text::default()
        });
        vec![frame.into_geometry()]
    }
}
