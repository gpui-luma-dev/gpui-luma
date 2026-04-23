use std::sync::Arc;

use gpui::{AppContext, Entity, SharedString};

use super::{Scrollbar, ScrollbarState, ScrollbarTemplate, default_scrollbar_template};
use crate::controls::value::{ControlRange, normalized_step, value_from_input};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ScrollbarOrientation {
    Horizontal,
    #[default]
    Vertical,
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
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ScrollbarTemplate>,
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

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.template = template;
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
    use super::normalized_thumb_fraction;

    #[test]
    fn thumb_fraction_stays_visible_and_finite() {
        assert_eq!(normalized_thumb_fraction(0.0), 0.05);
        assert_eq!(normalized_thumb_fraction(0.5), 0.5);
        assert_eq!(normalized_thumb_fraction(2.0), 1.0);
        assert_eq!(normalized_thumb_fraction(f32::NAN), 0.25);
    }
}
