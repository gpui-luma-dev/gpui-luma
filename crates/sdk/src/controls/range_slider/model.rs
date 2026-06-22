use std::{ops::RangeInclusive, sync::Arc};

use gpui::{AbsoluteLength, AppContext, Entity, SharedString};

use super::constraints::{clamp_and_snap_value, normalize_intervals};
use super::control::RangeSliderControl;
use super::template::{RangeSliderTemplate, default_range_slider_template};
use crate::controls::slider::{SliderOrientation, SliderState, SliderThumbSize};
use crate::controls::value::{ControlRange, normalized_step, value_from_input};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeSliderSegmentKind {
    Allowed,
    Gap,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeSliderTrackSegment {
    pub start_percentage: f32,
    pub end_percentage: f32,
    pub kind: RangeSliderSegmentKind,
}

#[derive(Clone)]
pub struct RangeSliderModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: SliderOrientation,
    pub(crate) size: ControlSize,
    pub(crate) thumb_size: Option<SliderThumbSize>,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) value: f32,
    pub(crate) allowed_intervals: Vec<RangeInclusive<f32>>,
    pub(crate) enabled: bool,
    pub(crate) corner_radius: Option<AbsoluteLength>,
    pub(crate) template: Arc<dyn RangeSliderTemplate>,
}

pub struct RangeSliderRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: SliderOrientation,
    pub size: ControlSize,
    pub thumb_size: Option<SliderThumbSize>,
    pub range: ControlRange,
    pub step: f32,
    pub value: f32,
    pub percentage: f32,
    pub allowed_intervals: &'a [RangeInclusive<f32>],
    pub track_segments: Vec<RangeSliderTrackSegment>,
    pub enabled: bool,
    pub corner_radius: Option<AbsoluteLength>,
    pub state: SliderState,
}

pub struct RangeSliderBuilder {
    pub(crate) model: RangeSliderModel,
}

impl RangeSliderBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: RangeSliderModel {
                id: id.into(),
                orientation: SliderOrientation::Horizontal,
                size: ControlSize::Md,
                thumb_size: None,
                range: ControlRange::default(),
                step: 1.0,
                value: 0.0,
                allowed_intervals: Vec::new(),
                enabled: true,
                corner_radius: None,
                template: default_range_slider_template(),
            },
        }
    }

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.model.orientation = orientation;
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.model.orientation = SliderOrientation::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.model.orientation = SliderOrientation::Vertical;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn thumb_size(mut self, size: SliderThumbSize) -> Self {
        self.model.thumb_size = Some(size);
        self
    }

    pub fn clear_thumb_size(mut self) -> Self {
        self.model.thumb_size = None;
        self
    }

    pub fn range(mut self, range: impl Into<ControlRange>) -> Self {
        self.model.range = range.into();
        self.model.allowed_intervals = normalize_intervals(&self.model.allowed_intervals, self.model.range);
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        self
    }

    pub fn allowed_intervals(mut self, intervals: Vec<RangeInclusive<f32>>) -> Self {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        self.model.value =
            constrain_value(value_from_input(value), self.model.range, self.model.step, &self.model.allowed_intervals);
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.model.step = normalized_step(value_from_input(step));
        self.model.value =
            constrain_value(self.model.value, self.model.range, self.model.step, &self.model.allowed_intervals);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn corner_radius(mut self, radius: AbsoluteLength) -> Self {
        self.model.corner_radius = Some(radius);
        self
    }

    pub fn clear_corner_radius(mut self) -> Self {
        self.model.corner_radius = None;
        self
    }

    pub fn template(mut self, template: Arc<dyn RangeSliderTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<RangeSliderControl> {
        cx.new(|cx| RangeSliderControl::from_builder(self, cx))
    }
}

pub(crate) fn constrain_value(
    value: f32,
    range: ControlRange,
    step: f32,
    allowed_intervals: &[RangeInclusive<f32>],
) -> f32 {
    clamp_and_snap_value(value, allowed_intervals, range, step)
}

pub(crate) fn build_track_segments(
    allowed_intervals: &[RangeInclusive<f32>],
    range: ControlRange,
) -> Vec<RangeSliderTrackSegment> {
    let normalized = normalize_intervals(allowed_intervals, range);
    if normalized.is_empty() {
        return vec![RangeSliderTrackSegment {
            start_percentage: 0.0,
            end_percentage: 1.0,
            kind: RangeSliderSegmentKind::Allowed,
        }];
    }

    let mut segments = Vec::with_capacity(normalized.len() * 2 + 1);
    let mut cursor = range.start;

    for interval in normalized {
        let start = *interval.start();
        let end = *interval.end();

        if start > cursor {
            segments.push(RangeSliderTrackSegment {
                start_percentage: range.percentage(cursor),
                end_percentage: range.percentage(start),
                kind: RangeSliderSegmentKind::Gap,
            });
        }

        segments.push(RangeSliderTrackSegment {
            start_percentage: range.percentage(start),
            end_percentage: range.percentage(end),
            kind: RangeSliderSegmentKind::Allowed,
        });
        cursor = end;
    }

    if cursor < range.end {
        segments.push(RangeSliderTrackSegment {
            start_percentage: range.percentage(cursor),
            end_percentage: range.percentage(range.end),
            kind: RangeSliderSegmentKind::Gap,
        });
    }

    if segments.is_empty() {
        segments.push(RangeSliderTrackSegment {
            start_percentage: 0.0,
            end_percentage: 1.0,
            kind: RangeSliderSegmentKind::Allowed,
        });
    }

    segments
}
