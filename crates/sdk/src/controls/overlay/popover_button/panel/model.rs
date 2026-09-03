use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Bounds, Context, Entity, IntoElement, Pixels, SharedString, Size, Window};

use super::control::PopoverPanel;

pub type PopoverPanelContent = Arc<dyn Fn(&PopoverPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPanelPlacement {
    #[default]
    BelowStart,
    BelowCenter,
    AboveStart,
    SmartStart,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PopoverPanelDismissPolicy {
    #[default]
    KeepOpen,
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
}

#[derive(Clone)]
pub struct PopoverPanelModel {
    pub(crate) id: SharedString,
    pub(crate) content: PopoverPanelContent,
    pub(crate) open: bool,
    pub(crate) animated: bool,
    pub(crate) anchor_bounds: Option<Bounds<Pixels>>,
    pub(crate) placement: PopoverPanelPlacement,
    pub(crate) dismiss_policy: PopoverPanelDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) offset_y: Pixels,
    pub(crate) window_margin: Pixels,
    pub(crate) focus_on_open: bool,
    pub(crate) initial_content_size: Option<Size<Pixels>>,
}

pub struct PopoverPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub open: bool,
    pub anchor_bounds: Option<Bounds<Pixels>>,
    pub placement: PopoverPanelPlacement,
    pub dismiss_policy: PopoverPanelDismissPolicy,
    pub dismissible: bool,
    pub focused: bool,
    pub content_bounds: Option<Bounds<Pixels>>,
}

pub struct PopoverPanelBuilder {
    pub(crate) model: PopoverPanelModel,
}

pub fn new(id: impl Into<SharedString>) -> PopoverPanelBuilder {
    PopoverPanelBuilder::new(id)
}

impl PopoverPanelBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: PopoverPanelModel {
                id: id.into(),
                content: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                animated: true,
                anchor_bounds: None,
                placement: PopoverPanelPlacement::BelowStart,
                dismiss_policy: PopoverPanelDismissPolicy::KeepOpen,
                dismissible: true,
                offset_y: gpui::px(4.0),
                window_margin: gpui::px(8.0),
                focus_on_open: true,
                initial_content_size: None,
            },
        }
    }

    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&PopoverPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement + 'static,
    {
        self.model.content = Arc::new(content);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.model.animated = animated;
        self
    }

    pub fn anchor_bounds(mut self, bounds: Bounds<Pixels>) -> Self {
        self.model.anchor_bounds = Some(bounds);
        self
    }

    pub fn placement(mut self, placement: PopoverPanelPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn dismiss_policy(mut self, dismiss_policy: PopoverPanelDismissPolicy) -> Self {
        self.model.dismiss_policy = dismiss_policy;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.model.dismissible = dismissible;
        self
    }

    pub fn offset_y(mut self, offset_y: Pixels) -> Self {
        self.model.offset_y = offset_y;
        self
    }

    pub fn window_margin(mut self, margin: Pixels) -> Self {
        self.model.window_margin = margin;
        self
    }

    pub fn focus_on_open(mut self, focus_on_open: bool) -> Self {
        self.model.focus_on_open = focus_on_open;
        self
    }

    pub fn initial_content_size(mut self, size: Size<Pixels>) -> Self {
        self.model.initial_content_size = Some(size);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<PopoverPanel> {
        cx.new(|cx| PopoverPanel::from_builder(self, cx))
    }
}
