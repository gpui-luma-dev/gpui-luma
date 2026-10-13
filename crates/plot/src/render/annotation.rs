use gpui::{Bounds, App, Window, TextAlign, point, size, fill, px};
use super::geometry::CartesianGeometry;
use crate::{Annotation, ChartAnnotation};
pub(super) fn paint(items: &[ChartAnnotation], geometry: &CartesianGeometry, window: &mut Window, cx: &mut App) {
    window.with_content_mask(Some(gpui::ContentMask { bounds: geometry.plot_bounds }), |window| {
        for item in items {
            let (from, to) = match item.geometry {
                Annotation::Point(p) | Annotation::Text(p) => (p, None),
                Annotation::Line { from, to } | Annotation::Band { from, to } => (from, Some(to)),
            };
            let Some(a) = geometry.data_position(from) else {
                continue;
            };
            let style = &item.presentation;
            match item.geometry {
                Annotation::Point(_) => {
                    let radius = super::axis::positive(style.marker_radius);
                    if radius > px(0.0)
                        && let Some(color) = style.fill
                    {
                        window.paint_quad(
                            fill(Bounds::new(a - point(radius, radius), size(radius * 2.0, radius * 2.0)), color)
                                .corner_radii(radius),
                        );
                    }
                }
                Annotation::Line { .. } | Annotation::Band { .. } => {
                    let Some(b) = to.and_then(|p| geometry.data_position(p)) else {
                        continue;
                    };
                    if matches!(item.geometry, Annotation::Band { .. }) {
                        if let Some(color) = style.fill {
                            window.paint_quad(fill(
                                Bounds::new(
                                    point(a.x.min(b.x), a.y.min(b.y)),
                                    size((a.x - b.x).abs(), (a.y - b.y).abs()),
                                ),
                                color,
                            ));
                        }
                    } else if let Some(stroke) = style.stroke {
                        super::line::paint_path([a, b], stroke, window);
                    }
                }
                Annotation::Text(_) => {}
            }
            if let Some(text) = &style.text {
                let line = super::axis::shape(text.clone(), &style.text_style, window);
                line.paint(a, super::axis::positive(style.text_style.line_height), TextAlign::Left, None, window, cx)
                    .ok();
            }
        }
    });
}
