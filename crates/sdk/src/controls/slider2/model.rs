use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{ops::RangeInclusive, sync::OnceLock};

use gpui::{AbsoluteLength, AppContext, Entity, Hsla, SharedString};

use super::constraints::{clamp_and_snap_value, normalize_intervals};
use super::control::Slider2Control;
use super::domain::{DomainTrackRenderer, HueDomainTrack};
use super::input::{Slider2InputStrategy, wrap_and_snap};
use super::segments::build_track_segments;
use super::template::{Slider2Template, default_slider2_template};
use crate::controls::slider::SliderState;
use crate::controls::value::{ControlRange, normalized_step, value_from_input};
use crate::theme::ControlSize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ThumbId(u64);

impl ThumbId {
    pub fn next() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SliderThumbRole {
    #[default]
    Value,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SliderThumbValue {
    pub id: ThumbId,
    pub position: f32,
    pub preview: Option<Hsla>,
    pub role: SliderThumbRole,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrackSegmentKind {
    Domain,
    Active,
    Inactive,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrackSegment {
    pub start: f32,
    pub end: f32,
    pub kind: TrackSegmentKind,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TrackPresentation {
    #[default]
    Fill,
    Domain,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Slider2Orientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Slider2ThumbSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl From<ControlSize> for Slider2ThumbSize {
    fn from(value: ControlSize) -> Self {
        match value {
            ControlSize::Sm => Self::Sm,
            ControlSize::Md => Self::Md,
            ControlSize::Lg => Self::Lg,
        }
    }
}

#[derive(Clone)]
pub struct Slider2Model {
    pub(crate) id: SharedString,
    pub(crate) strategy: Slider2InputStrategy,
    pub(crate) presentation: TrackPresentation,
    pub(crate) size: ControlSize,
    pub(crate) thumb_size: Option<Slider2ThumbSize>,
    pub(crate) range: ControlRange,
    pub(crate) step: f32,
    pub(crate) thumbs: Vec<SliderThumbValue>,
    pub(crate) allowed_intervals: Vec<RangeInclusive<f32>>,
    pub(crate) reversed: bool,
    pub(crate) wrapping: bool,
    pub(crate) enabled: bool,
    pub(crate) corner_radius: Option<AbsoluteLength>,
    pub(crate) domain_track: Option<Arc<dyn DomainTrackRenderer>>,
    pub(crate) template: Arc<dyn Slider2Template>,
}

pub struct Slider2RenderModel<'a> {
    pub id: &'a SharedString,
    pub strategy: Slider2InputStrategy,
    pub orientation: Slider2Orientation,
    pub presentation: TrackPresentation,
    pub size: ControlSize,
    pub thumb_size: Option<Slider2ThumbSize>,
    pub range: ControlRange,
    pub step: f32,
    pub thumbs: &'a [SliderThumbValue],
    pub track_segments: Vec<TrackSegment>,
    pub reversed: bool,
    pub wrapping: bool,
    pub enabled: bool,
    pub corner_radius: Option<AbsoluteLength>,
    pub domain_track: Option<Arc<dyn DomainTrackRenderer>>,
    pub state: SliderState,
}

pub struct Slider2Builder {
    pub(crate) model: Slider2Model,
}

impl Slider2Builder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let thumb_id = ThumbId::next();
        Self {
            model: Slider2Model {
                id: id.into(),
                strategy: Slider2InputStrategy::Horizontal,
                presentation: TrackPresentation::Fill,
                size: ControlSize::Md,
                thumb_size: None,
                range: ControlRange::default(),
                step: 1.0,
                thumbs: vec![SliderThumbValue {
                    id: thumb_id,
                    position: 0.0,
                    preview: None,
                    role: SliderThumbRole::Value,
                }],
                allowed_intervals: Vec::new(),
                reversed: false,
                wrapping: false,
                enabled: true,
                corner_radius: None,
                domain_track: None,
                template: default_slider2_template(),
            },
        }
    }

    pub fn strategy(mut self, strategy: Slider2InputStrategy) -> Self {
        self.model.strategy = strategy;
        self
    }

    pub fn orientation(mut self, orientation: Slider2Orientation) -> Self {
        self.model.strategy = orientation.into();
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.model.strategy = Slider2InputStrategy::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.model.strategy = Slider2InputStrategy::Vertical;
        self
    }

    pub fn angular(mut self, min_angle: f32, max_angle: f32) -> Self {
        self.model.strategy = Slider2InputStrategy::Angular { min_angle, max_angle };
        self
    }

    pub fn wrapping(mut self, wrapping: bool) -> Self {
        self.model.wrapping = wrapping;
        self.sync_primary_thumb_position();
        self
    }

    pub fn fill(mut self) -> Self {
        self.model.presentation = TrackPresentation::Fill;
        self.model.domain_track = None;
        self
    }

    pub fn domain_track(mut self, renderer: Arc<dyn DomainTrackRenderer>) -> Self {
        self.model.presentation = TrackPresentation::Domain;
        self.model.domain_track = Some(renderer);
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn thumb_size(mut self, size: Slider2ThumbSize) -> Self {
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
        self.sync_primary_thumb_position();
        self
    }

    pub fn allowed_intervals(mut self, intervals: Vec<RangeInclusive<f32>>) -> Self {
        self.model.allowed_intervals = normalize_intervals(&intervals, self.model.range);
        self.sync_primary_thumb_position();
        self
    }

    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.model.step = normalized_step(value_from_input(step));
        self.sync_primary_thumb_position();
        self
    }

    pub fn value(mut self, value: impl Into<f64>) -> Self {
        let value = constrain_primary_value(value_from_input(value), &self.model);
        self.set_primary_position(self.model.range.percentage(value));
        self
    }

    pub fn reversed(mut self, reversed: bool) -> Self {
        self.model.reversed = reversed;
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

    pub fn template(mut self, template: Arc<dyn Slider2Template>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<Slider2Control> {
        cx.new(|cx| Slider2Control::from_builder(self, cx))
    }

    fn sync_primary_thumb_position(&mut self) {
        let value = primary_value(&self.model);
        self.set_primary_position(self.model.range.percentage(value));
    }

    fn set_primary_position(&mut self, position: f32) {
        if let Some(thumb) = self.model.thumbs.first_mut() {
            thumb.position = position.clamp(0.0, 1.0);
        }
    }
}

pub(crate) fn primary_value(model: &Slider2Model) -> f32 {
    let position = model.thumbs.first().map(|thumb| thumb.position).unwrap_or(0.0);
    let value = model.range.value_at(position);
    constrain_primary_value(value, model)
}

pub(crate) fn constrain_primary_value(value: f32, model: &Slider2Model) -> f32 {
    if model.wrapping && model.allowed_intervals.is_empty() {
        wrap_and_snap(value, model.range, model.step)
    } else {
        clamp_and_snap_value(value, &model.allowed_intervals, model.range, model.step)
    }
}

pub(crate) fn build_render_segments(model: &Slider2Model) -> Vec<TrackSegment> {
    let thumb_position = model.thumbs.first().map(|thumb| thumb.position).unwrap_or(0.0);
    build_track_segments(model.presentation, thumb_position, &model.allowed_intervals, model.range)
}

static DEFAULT_HUE_TRACK: OnceLock<Arc<HueDomainTrack>> = OnceLock::new();

pub fn default_hue_domain_track() -> Arc<dyn DomainTrackRenderer> {
    DEFAULT_HUE_TRACK.get_or_init(|| Arc::new(HueDomainTrack)).clone()
}
