use std::time::Duration;

use super::super::model::{SliderModel, ThumbId};
use super::SliderControl;
use crate::motion::{DEFAULT_TRANSITION_DURATION, VisualTransition};

impl SliderControl {
    pub(super) fn apply_thumb_motion(&mut self, thumb_id: ThumbId, percentage: f32, animate: bool) {
        self.sync_transitions_with_thumbs();
        let should_animate = animate && self.animated;
        let Some(transition) = self.transition_for_thumb_mut(thumb_id) else {
            return;
        };
        if should_animate {
            transition.set_target(percentage);
        } else {
            transition.snap_to(percentage);
        }
    }

    pub(super) fn transition_for_thumb_mut(&mut self, thumb_id: ThumbId) -> Option<&mut VisualTransition> {
        let index = self.model.thumbs.iter().position(|thumb| thumb.id == thumb_id)?;
        self.thumb_transitions.get_mut(index)
    }

    pub(super) fn sync_transitions_with_thumbs(&mut self) {
        let duration = transition_duration(self.animated);
        while self.thumb_transitions.len() < self.model.thumbs.len() {
            let index = self.thumb_transitions.len();
            let position = self.model.thumbs[index].position;
            self.thumb_transitions.push(VisualTransition::new(position, duration));
        }
        if self.thumb_transitions.len() > self.model.thumbs.len() {
            self.thumb_transitions.truncate(self.model.thumbs.len());
        }
    }

    pub(super) fn rebuild_thumb_transitions(&mut self) {
        let duration = transition_duration(self.animated);
        self.thumb_transitions =
            self.model.thumbs.iter().map(|thumb| VisualTransition::new(thumb.position, duration)).collect();
    }

    pub(super) fn refresh_display_thumbs(&mut self) {
        self.display_thumbs = self
            .model
            .thumbs
            .iter()
            .enumerate()
            .map(|(index, thumb)| {
                let mut display = thumb.clone();
                display.position =
                    self.thumb_transitions.get(index).map(|transition| transition.progress()).unwrap_or(thumb.position);
                display
            })
            .collect();
    }
}

pub(super) fn transition_duration(animated: bool) -> Duration {
    if animated {
        DEFAULT_TRANSITION_DURATION
    } else {
        Duration::ZERO
    }
}

pub(super) fn sync_thumb_previews(model: &mut SliderModel) {
    let thumb_ids: Vec<_> = model.thumbs.iter().map(|thumb| thumb.id).collect();
    for thumb_id in thumb_ids {
        sync_thumb_preview(model, thumb_id);
    }
}

pub(super) fn sync_thumb_preview(model: &mut SliderModel, thumb_id: ThumbId) {
    let Some(renderer) = model.domain_track.as_ref() else {
        return;
    };

    let position = model.thumbs.iter().find(|thumb| thumb.id == thumb_id).map(|thumb| thumb.position);
    let Some(position) = position else {
        return;
    };

    if let Some(color) = renderer.get_color_at_position(position)
        && let Some(thumb) = model.thumbs.iter_mut().find(|thumb| thumb.id == thumb_id)
    {
        thumb.preview = Some(color);
    }
}
