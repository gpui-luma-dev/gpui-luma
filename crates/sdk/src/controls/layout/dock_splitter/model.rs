use std::sync::Arc;

use gpui::{App, AppContext, Entity, FocusHandle, Pixels, SharedString, Window};

use super::template::template_with_modifier;
use super::{DockSplitterTemplate, DockSplitterTheme, default_dock_splitter_template, default_dock_splitter_theme};
use super::control::DockSplitter;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SplitterOrientation {
    Horizontal,
    Vertical,
}

pub type DockSplitterResizeHandler = Box<dyn Fn(&f32, &mut Window, &mut App) + 'static>;

pub struct DockSplitterModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: SplitterOrientation,
    pub(crate) enabled: bool,
    pub(crate) keyboard_step: f32,
    pub(crate) keyboard_shift_step: f32,
    pub(crate) on_resize: Option<DockSplitterResizeHandler>,
    pub(crate) template: Arc<dyn DockSplitterTemplate>,
    pub(crate) theme: Arc<dyn DockSplitterTheme>,
}

pub struct DockSplitterRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: SplitterOrientation,
    pub enabled: bool,
    pub hovered: bool,
    pub dragging: bool,
    pub focused: bool,
    pub focus_handle: &'a FocusHandle,
}

pub struct DockSplitterBuilder {
    callbacks: crate::infra::callbacks::EventCallbacks<crate::controls::dock_splitter::DockSplitterEvent>,

    pub(crate) model: DockSplitterModel,
}

impl DockSplitterBuilder {
    /// Append an owner-bound handler for every semantic event.
    /// Registrations append in order; released owners are skipped.
    pub fn on_event<M: 'static>(
        mut self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &crate::controls::dock_splitter::DockSplitterEvent, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.callbacks = self.callbacks.on_event(cx, callback);
        self
    }

    /// Handle `DockSplitterEvent::ResizeStart` through the existing event stream.
    pub fn on_resize_start<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::ResizeStart = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `DockSplitterEvent::Resize` through the existing event stream.
    pub fn on_resize_delta<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, f32, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::Resize { total_delta } = event {
                callback(owner, *total_delta, cx);
            }
        })
    }

    /// Handle `DockSplitterEvent::ResizeEnd` through the existing event stream.
    pub fn on_resize_end<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::ResizeEnd = event {
                callback(owner, cx);
            }
        })
    }

    /// Handle `DockSplitterEvent::FocusChanged` through the existing event stream.
    pub fn on_focus_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::FocusChanged { focused } = event {
                callback(owner, *focused, cx);
            }
        })
    }

    /// Handle `DockSplitterEvent::HoverChanged` through the existing event stream.
    pub fn on_hover_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::HoverChanged { hovered } = event {
                callback(owner, *hovered, cx);
            }
        })
    }

    /// Handle `DockSplitterEvent::EnabledChanged` through the existing event stream.
    pub fn on_enabled_changed<M: 'static>(
        self,
        cx: &gpui::Context<M>,
        callback: impl Fn(&mut M, bool, &mut gpui::Context<M>) + 'static,
    ) -> Self {
        self.on_event(cx, move |owner, event, cx| {
            if let crate::controls::dock_splitter::DockSplitterEvent::EnabledChanged { enabled } = event {
                callback(owner, *enabled, cx);
            }
        })
    }

    pub fn new(id: impl Into<SharedString>, orientation: SplitterOrientation) -> Self {
        Self {
            callbacks: Default::default(),

            model: DockSplitterModel {
                id: id.into(),
                orientation,
                enabled: true,
                keyboard_step: 2.0,
                keyboard_shift_step: 10.0,
                on_resize: None,
                template: default_dock_splitter_template(),
                theme: default_dock_splitter_theme(),
            },
        }
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn keyboard_step(mut self, step: Pixels) -> Self {
        self.model.keyboard_step = step.as_f32().max(0.1);
        self
    }

    pub fn keyboard_shift_step(mut self, step: Pixels) -> Self {
        self.model.keyboard_shift_step = step.as_f32().max(0.1);
        self
    }

    pub fn on_resize(mut self, callback: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.model.on_resize = Some(Box::new(callback));
        self
    }

    pub fn template(mut self, template: Arc<dyn DockSplitterTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &DockSplitterRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = template_with_modifier(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn theme(mut self, theme: Arc<dyn DockSplitterTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn spawn(mut self, cx: &mut impl AppContext) -> Entity<DockSplitter> {
        let callbacks = std::mem::take(&mut self.callbacks);
        let control = { cx.new(|cx| DockSplitter::from_builder(self, cx)) };
        callbacks.bind(&control, cx);
        control
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_dock_splitter_template();
        let builder = DockSplitterBuilder::new("splitter-test", SplitterOrientation::Vertical)
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
