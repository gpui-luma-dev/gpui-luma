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
    pub(crate) model: DockSplitterModel,
}

impl DockSplitterBuilder {
    pub fn new(id: impl Into<SharedString>, orientation: SplitterOrientation) -> Self {
        Self {
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

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<DockSplitter> {
        cx.new(|cx| DockSplitter::from_builder(self, cx))
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
