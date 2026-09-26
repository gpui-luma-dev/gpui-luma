//! Keyed positioning over ListState. Estimates only bridge unmeasured rows;
//! final alignment uses measured bounds and ListState's own boundary clamping.
use std::time::Instant;
use super::*;
use crate::motion::{DEFAULT_TRANSITION_DURATION, ease_out_cubic};

#[derive(Clone, Copy)]
enum Alignment {
    Start,
    End,
    Center,
}

pub(super) struct PositionRequest {
    id: SharedString,
    center: bool,
    smooth: bool,
    alignment: Option<Alignment>,
    started: Option<Instant>,
    progress: f32,
    last_correction: Option<gpui::ListOffset>,
}

impl<T: Clone + Send + Sync + 'static> TreeViewControl<T> {
    /// Minimally reveal a displayed node without selecting or focusing it.
    /// Oversized rows show their leading edge. Missing, filtered and collapsed
    /// nodes are ignored. Returns whether the request was accepted.
    pub fn scroll_to(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.request_position(id.into(), false, false, cx)
    }

    /// Center a displayed node, clamped to collection boundaries. Selection and
    /// focus are unchanged. The latest request wins, even if its ID is hidden.
    pub fn scroll_to_center(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.request_position(id.into(), true, false, cx)
    }

    /// Smooth counterpart of `scroll_to`. Manual input, data/projection changes,
    /// expansion changes and subsequent positioning requests interrupt motion.
    pub fn scroll_to_smooth(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.request_position(id.into(), false, true, cx)
    }

    /// Smoothly center a displayed node without changing selection or focus.
    pub fn scroll_to_center_smooth(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.request_position(id.into(), true, true, cx)
    }

    /// Open loaded ancestors immediately, then minimally reveal the node.
    /// Does not fetch data or override filtering. Emits ordinary expansion
    /// events for newly opened ancestors; selection and focus are unchanged.
    pub fn reveal_node(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.reveal_position(id.into(), false, cx)
    }

    /// Open loaded ancestors immediately, then smoothly reveal the node.
    pub fn reveal_node_smooth(&mut self, id: impl Into<SharedString>, cx: &mut Context<Self>) -> bool {
        self.reveal_position(id.into(), true, cx)
    }

    pub(super) fn cancel_position(&mut self) {
        self.position = None;
    }

    fn request_position(&mut self, id: SharedString, center: bool, smooth: bool, cx: &mut Context<Self>) -> bool {
        self.cancel_position();
        if !self.visible_ids().contains(&id) {
            return false;
        }
        self.position = Some(PositionRequest {
            id,
            center,
            smooth,
            alignment: None,
            started: None,
            progress: 0.0,
            last_correction: None,
        });
        cx.notify();
        true
    }

    fn reveal_position(&mut self, id: SharedString, smooth: bool, cx: &mut Context<Self>) -> bool {
        self.cancel_position();
        let mut included = false;
        self.projection.visit(&self.model.items, |node, _| {
            included |= node.id == id;
            true
        });
        if !included {
            return false;
        }
        let mut ancestors = Vec::new();
        let mut parent = self.model.index.nodes.get(&id).and_then(|node| node.parent.clone());
        while let Some(id) = parent {
            parent = self.model.index.nodes.get(&id).and_then(|node| node.parent.clone());
            ancestors.push(id);
        }
        for id in ancestors.into_iter().rev() {
            self.expand(id, cx);
        }
        // Resolve the new loaded path at its final geometry before positioning.
        self.expand_transitions.clear();
        self.sync_expand_transitions_for_tree();
        self.rebuild_flat_cache(cx);
        self.request_position(id, false, smooth, cx)
    }

    pub(super) fn schedule_position_frame(&mut self, window: &Window, cx: &mut Context<Self>) {
        if cx.has_active_drag()
            || (!window.is_window_active() && self.position.as_ref().is_some_and(|request| request.smooth))
        {
            self.cancel_position();
        }
        if self.position.is_none()
            || self.position_frame_scheduled
            || self.list_state.viewport_bounds().size.height <= px(0.0)
        {
            return;
        }
        self.position_frame_scheduled = true;
        let owner = cx.entity().downgrade();
        window.on_next_frame(move |_, app| {
            let _ = owner.update(app, |tree, cx| {
                tree.position_frame_scheduled = false;
                if tree.position.is_some() {
                    cx.notify();
                }
            });
        });
    }

    /// Called after layout, using geometry from the frame just painted.
    pub(super) fn advance_position(&mut self, cx: &mut Context<Self>) {
        let Some(mut request) = self.position.take() else {
            return;
        };
        if cx.has_active_drag() || self.list_state.is_scrollbar_dragging() {
            return;
        }
        let Some(index) = self.flat_index_for_id(&request.id) else {
            return;
        };
        let viewport = self.list_state.viewport_bounds();
        let height = viewport.size.height.as_f32();
        if height <= 0.0 {
            self.position = Some(request);
            return;
        }
        // Wait for existing disclosure motion before resolving the destination.
        if self.expand_transitions.values().any(DisclosureMotion::is_animating) {
            self.position = Some(request);
            return;
        }
        let current = self.list_state.logical_scroll_top();
        let bounds = self.list_state.bounds_for_item(index);
        let mut alignment = if let Some(alignment) = request.alignment {
            alignment
        } else {
            let alignment = if request.center {
                Alignment::Center
            } else if let Some(row) = bounds {
                if row.size.height > viewport.size.height || row.top() < viewport.top() {
                    Alignment::Start
                } else if row.bottom() > viewport.bottom() {
                    Alignment::End
                } else {
                    return;
                }
            } else if index <= current.item_ix {
                Alignment::Start
            } else {
                Alignment::End
            };
            request.alignment = Some(alignment);
            alignment
        };
        if !request.center && bounds.is_some_and(|row| row.size.height > viewport.size.height) {
            alignment = Alignment::Start;
            request.alignment = Some(alignment);
        }
        let now = cx.background_executor().now();
        let first_frame = request.started.is_none();
        let started = *request.started.get_or_insert(now);
        let t = if request.smooth {
            (now.duration_since(started).as_secs_f32() / DEFAULT_TRANSITION_DURATION.as_secs_f32()).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let progress = ease_out_cubic(t);
        let fraction = if progress >= 1.0 {
            1.0
        } else {
            ((progress - request.progress) / (1.0 - request.progress)).clamp(0.0, 1.0)
        };
        request.progress = progress;
        if let Some(row) = bounds {
            let delta = match alignment {
                Alignment::Start => row.top() - viewport.top(),
                Alignment::End => row.bottom() - viewport.bottom(),
                Alignment::Center => row.center().y - viewport.center().y,
            };
            if t >= 1.0 {
                // A repeated offset after correction means native layout clamped
                // at a boundary (e.g. centering the first or last row).
                if delta.abs() <= px(0.5)
                    || request.last_correction.is_some_and(|last| {
                        last.item_ix == current.item_ix
                            && (last.offset_in_item - current.offset_in_item).abs() <= px(0.1)
                    })
                {
                    return;
                }
                request.last_correction = Some(current);
            }
            self.list_state.scroll_by(delta * fraction);
        } else {
            // ListState exposes no prefix height for unmeasured rows. Interpolate
            // in logical row space until the target is measured, then correct in
            // pixels. This keeps virtualization and avoids a second scroll state.
            let estimate = self
                .list_state
                .bounds_for_item(current.item_ix)
                .map_or(24.0, |row| row.size.height.as_f32().max(1.0));
            let target = match alignment {
                Alignment::Start => index as f32,
                Alignment::End => index as f32 + 1.0 - height / estimate,
                Alignment::Center => index as f32 + 0.5 - height / (2.0 * estimate),
            }
            .max(0.0);
            let from = current.item_ix as f32 + current.offset_in_item.as_f32() / estimate;
            let next = (from + (target - from) * fraction).max(0.0);
            // At completion, anchor the unmeasured target itself. The next layout
            // supplies exact bounds and measures preceding rows for alignment.
            let offset = if t >= 1.0 {
                gpui::ListOffset { item_ix: index, offset_in_item: px(0.0) }
            } else {
                gpui::ListOffset { item_ix: next.floor() as usize, offset_in_item: px(next.fract() * estimate) }
            };
            self.list_state.scroll_to(offset);
        }
        self.position = Some(request);
        if first_frame || t >= 1.0 {
            cx.notify();
        }
    }
}
