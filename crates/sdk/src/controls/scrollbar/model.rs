use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::template::template_with_modifier;
use super::{Scrollbar, ScrollbarState, ScrollbarTemplate, default_scrollbar_template};
use crate::controls::value::{ControlRange, normalized_step, value_from_input};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarOrientation {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarStyle {
    #[default]
    Ghost,
    Soft,
}

#[derive(Clone)]
pub struct ScrollbarModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: ScrollbarOrientation,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) page_step: f32,
    pub(crate) value: f32,
    pub(crate) thumb_fraction: f32,
    pub(crate) length: Option<f32>,
    pub(crate) size: ControlSize,
    pub(crate) style: ScrollbarStyle,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ScrollbarTemplate>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarViewport {
    pub content: ControlRange,
    pub visible_start: f32,
    pub visible_end: f32,
}

impl ScrollbarViewport {
    pub fn new(content_start: f32, content_end: f32, visible_start: f32, visible_end: f32) -> Self {
        let content = ControlRange::new(content_start, content_end);
        let visible_start = if visible_start.is_finite() {
            visible_start
        } else {
            content.start
        };
        let visible_end = if visible_end.is_finite() {
            visible_end
        } else {
            visible_start
        };

        Self { content, visible_start, visible_end }
    }

    pub fn scroll_range(self) -> ControlRange {
        let visible_span = self.visible_span();
        ControlRange::new(self.content.start, (self.content.end - visible_span).max(self.content.start))
    }

    pub fn value(self, step: f32) -> f32 {
        self.scroll_range().snap(self.visible_start, step)
    }

    pub fn thumb_fraction(self) -> f32 {
        normalized_thumb_fraction(self.visible_span() / self.content.span())
    }

    pub fn scrollable(self) -> bool {
        self.content.span() - self.visible_span() > 0.5
    }

    fn visible_span(self) -> f32 {
        let start = self.visible_start.min(self.visible_end);
        let end = self.visible_start.max(self.visible_end);
        (end - start).clamp(0.0, self.content.span())
    }
}

pub struct ScrollbarRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: ScrollbarOrientation,
    pub range: ControlRange,
    pub step: f32,
    pub page_step: f32,
    pub value: f32,
    pub percentage: f32,
    pub thumb_fraction: f32,
    pub length: Option<f32>,
    pub size: ControlSize,
    pub style: ScrollbarStyle,
    pub enabled: bool,
    pub state: ScrollbarState,
}

pub struct ScrollbarBuilder {
    pub(crate) model: ScrollbarModel,
}

impl ScrollbarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ScrollbarModel {
                id: id.into(),
                orientation: ScrollbarOrientation::Vertical,
                range: ControlRange::default(),
                step: 1.0,
                page_step: 10.0,
                value: 0.0,
                thumb_fraction: 0.25,
                length: None,
                size: ControlSize::Md,
                style: ScrollbarStyle::Ghost,
                enabled: true,
                template: default_scrollbar_template(),
            },
        }
    }

    pub fn orientation(mut self, orientation: ScrollbarOrientation) -> Self {
        self.model.orientation = orientation;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.model.orientation = ScrollbarOrientation::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.model.orientation = ScrollbarOrientation::Vertical;
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.model.step = normalized_step(value_from_input(step));
        self.model.value = self.model.range.snap(self.model.value, self.model.step);
        self
    }

    pub fn page_step(mut self, page_step: impl Into<f64>) -> Self {
        self.model.page_step = normalized_step(value_from_input(page_step));
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value = self.model.range.snap(value_from_input(value), self.model.step);
        self
    }

    pub fn thumb_fraction(mut self, thumb_fraction: impl Into<f64>) -> Self {
        self.model.thumb_fraction = normalized_thumb_fraction(value_from_input(thumb_fraction));
        self
    }

    pub fn length(mut self, length: impl Into<f64>) -> Self {
        let length = value_from_input(length);
        self.model.length = (length.is_finite() && length > 0.0).then_some(length);
        self
    }

    pub fn viewport(
        mut self,
        content_start: impl Into<f64>,
        content_end: impl Into<f64>,
        visible_start: impl Into<f64>,
        visible_end: impl Into<f64>,
    ) -> Self {
        let viewport = ScrollbarViewport::new(
            value_from_input(content_start),
            value_from_input(content_end),
            value_from_input(visible_start),
            value_from_input(visible_end),
        );
        self.model.range = viewport.scroll_range();
        self.model.value = viewport.value(self.model.step);
        self.model.thumb_fraction = viewport.thumb_fraction();
        self.model.enabled = self.model.enabled && viewport.scrollable();
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn style(mut self, style: ScrollbarStyle) -> Self {
        self.model.style = style;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.model.style = ScrollbarStyle::Ghost;
        self
    }

    pub fn soft(mut self) -> Self {
        self.model.style = ScrollbarStyle::Soft;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ScrollbarRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Scrollbar> {
        cx.new(|cx| Scrollbar::from_builder(self, cx))
    }
}

pub(crate) fn normalized_thumb_fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.05, 1.0)
    } else {
        0.25
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ScrollbarBuilder, ScrollbarStyle, ScrollbarViewport, default_scrollbar_template, normalized_thumb_fraction,
    };
    use crate::controls::value::ControlRange;

    #[test]
    fn thumb_fraction_stays_visible_and_finite() {
        assert_eq!(normalized_thumb_fraction(0.0), 0.05);
        assert_eq!(normalized_thumb_fraction(0.5), 0.5);
        assert_eq!(normalized_thumb_fraction(2.0), 1.0);
        assert_eq!(normalized_thumb_fraction(f32::NAN), 0.25);
    }

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_scrollbar_template();
        let builder = ScrollbarBuilder::new("scrollbar-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!std::sync::Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn style_helpers_set_scrollbar_style() {
        assert_eq!(ScrollbarBuilder::new("soft").soft().model.style, ScrollbarStyle::Soft);
        assert_eq!(ScrollbarBuilder::new("ghost").soft().ghost().model.style, ScrollbarStyle::Ghost);
    }

    #[test]
    fn viewport_maps_content_and_visible_window_to_scrollbar_state() {
        let viewport = ScrollbarViewport::new(0.0, 1_000.0, 250.0, 450.0);

        assert_eq!(viewport.scroll_range(), ControlRange::new(0.0, 800.0));
        assert_eq!(viewport.value(1.0), 250.0);
        assert_eq!(viewport.thumb_fraction(), 0.2);
        assert!(viewport.scrollable());
    }

    #[test]
    fn viewport_clamps_thumb_fraction_when_content_fits() {
        let viewport = ScrollbarViewport::new(0.0, 100.0, 0.0, 100.0);

        assert_eq!(viewport.scroll_range(), ControlRange::new(0.0, 1.0));
        assert_eq!(viewport.value(1.0), 0.0);
        assert_eq!(viewport.thumb_fraction(), 1.0);
        assert!(!viewport.scrollable());
    }
}
