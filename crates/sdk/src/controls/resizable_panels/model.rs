use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, IntoElement, Pixels, SharedString, px};

use super::{
    ResizablePanels, ResizablePanelsTemplate, ResizablePanelsTheme, default_resizable_panels_template,
    default_resizable_panels_theme,
};

pub type PanelRender = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResizablePanelsOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct ResizablePanelSpec {
    pub default_size: f32,
    pub min_size: f32,
    pub max_size: f32,
    pub render: PanelRender,
}

impl ResizablePanelSpec {
    pub fn new_render<E, F>(render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        Self {
            default_size: 50.0,
            min_size: 10.0,
            max_size: 90.0,
            render: Rc::new(move || render().into_any_element()),
        }
    }

    pub fn default_size(mut self, percent: f32) -> Self {
        self.default_size = percent;
        self
    }

    pub fn min_size(mut self, percent: f32) -> Self {
        self.min_size = percent;
        self
    }

    pub fn max_size(mut self, percent: f32) -> Self {
        self.max_size = percent;
        self
    }
}

#[derive(Clone)]
pub struct ResizablePanelsModel {
    pub(crate) id: SharedString,
    pub(crate) orientation: ResizablePanelsOrientation,
    pub(crate) frame_width: Option<Pixels>,
    pub(crate) frame_height: Option<Pixels>,
    pub(crate) show_border: bool,
    pub(crate) enabled: bool,
    pub(crate) show_handle: bool,
    pub(crate) handle_size: Pixels,
    pub(crate) handle_grip: bool,
    pub(crate) keyboard_step: f32,
    pub(crate) keyboard_shift_step: f32,
    pub(crate) panels: Vec<ResizablePanelSpec>,
    pub(crate) template: Arc<dyn ResizablePanelsTemplate>,
    pub(crate) theme: Arc<dyn ResizablePanelsTheme>,
}

pub struct ResizablePanelsRenderModel<'a> {
    pub id: &'a SharedString,
    pub orientation: ResizablePanelsOrientation,
    pub frame_width: Option<Pixels>,
    pub frame_height: Option<Pixels>,
    pub show_border: bool,
    pub enabled: bool,
    pub show_handle: bool,
    pub handle_size: Pixels,
    pub handle_grip: bool,
    pub sizes: &'a [f32],
    pub panels: &'a [ResizablePanelSpec],
    pub measured_size: Option<gpui::Size<Pixels>>,
}

pub struct ResizablePanelsBuilder {
    pub(crate) model: ResizablePanelsModel,
}

impl ResizablePanelsBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: ResizablePanelsModel {
                id: id.into(),
                orientation: ResizablePanelsOrientation::Horizontal,
                frame_width: None,
                frame_height: None,
                show_border: true,
                enabled: true,
                show_handle: false,
                handle_size: px(1.0),
                handle_grip: false,
                keyboard_step: 2.0,
                keyboard_shift_step: 10.0,
                panels: Vec::new(),
                template: default_resizable_panels_template(),
                theme: default_resizable_panels_theme(),
            },
        }
    }

    pub fn orientation(mut self, orientation: ResizablePanelsOrientation) -> Self {
        self.model.orientation = orientation;
        self
    }

    /// Fixed width and height for demos; omit to fill the parent (`size_full`).
    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.model.frame_width = Some(width);
        self.model.frame_height = Some(height);
        self
    }

    pub fn show_border(mut self, show_border: bool) -> Self {
        self.model.show_border = show_border;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn show_handle(mut self, show_handle: bool) -> Self {
        self.model.show_handle = show_handle;
        self
    }

    pub fn handle_size(mut self, handle_size: Pixels) -> Self {
        self.model.handle_size = handle_size;
        self
    }

    pub fn handle_grip(mut self, handle_grip: bool) -> Self {
        self.model.handle_grip = handle_grip;
        self
    }

    pub fn keyboard_step(mut self, keyboard_step: f32) -> Self {
        self.model.keyboard_step = keyboard_step.max(0.1);
        self
    }

    pub fn keyboard_shift_step(mut self, keyboard_shift_step: f32) -> Self {
        self.model.keyboard_shift_step = keyboard_shift_step.max(0.1);
        self
    }

    pub fn panel(mut self, panel: ResizablePanelSpec) -> Self {
        self.model.panels.push(panel);
        self
    }

    pub fn panels<I>(mut self, panels: I) -> Self
    where
        I: IntoIterator<Item = ResizablePanelSpec>,
    {
        self.model.panels.extend(panels);
        self
    }

    pub fn template(mut self, template: Arc<dyn ResizablePanelsTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn theme(mut self, theme: Arc<dyn ResizablePanelsTheme>) -> Self {
        self.model.theme = theme;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ResizablePanels> {
        cx.new(|cx| ResizablePanels::from_builder(self, cx))
    }
}

pub fn render_pane<E, F>(render: F) -> PanelRender
where
    E: IntoElement,
    F: Fn() -> E + 'static,
{
    Rc::new(move || render().into_any_element())
}
