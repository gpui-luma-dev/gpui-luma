use std::sync::Arc;

use gpui::{App, AppContext, Entity, SharedString, Window};

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

    pub fn on_resize(mut self, callback: impl Fn(&f32, &mut Window, &mut App) + 'static) -> Self {
        self.model.on_resize = Some(Box::new(callback));
        self
    }

    pub fn template(mut self, template: Arc<dyn DockSplitterTemplate>) -> Self {
        self.model.template = template;
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
