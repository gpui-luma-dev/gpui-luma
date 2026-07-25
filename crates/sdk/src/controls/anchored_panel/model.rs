use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Bounds, Context, Entity, IntoElement, Pixels, SharedString, Size, Window};

use super::AnchoredPanel;

pub type AnchoredPanelContent = Arc<dyn Fn(&AnchoredPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnchoredPanelPlacement {
    #[default]
    BelowStart,
    BelowCenter,
    AboveStart,
    SmartStart,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnchoredPanelDismissPolicy {
    #[default]
    KeepOpen,
    CloseOnClickAway,
    CloseOnFocusLoss,
    CloseOnClickAwayOrFocusLoss,
}

#[derive(Clone)]
pub struct AnchoredPanelModel {
    pub(crate) id: SharedString,
    pub(crate) content: AnchoredPanelContent,
    pub(crate) open: bool,
    pub(crate) anchor_bounds: Option<Bounds<Pixels>>,
    pub(crate) placement: AnchoredPanelPlacement,
    pub(crate) dismiss_policy: AnchoredPanelDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) offset_y: Pixels,
    pub(crate) window_margin: Pixels,
    pub(crate) focus_on_open: bool,
    pub(crate) initial_content_size: Option<Size<Pixels>>,
}

pub struct AnchoredPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub open: bool,
    pub anchor_bounds: Option<Bounds<Pixels>>,
    pub placement: AnchoredPanelPlacement,
    pub dismiss_policy: AnchoredPanelDismissPolicy,
    pub dismissible: bool,
    pub focused: bool,
    pub content_bounds: Option<Bounds<Pixels>>,
}

pub struct AnchoredPanelBuilder {
    pub(crate) model: AnchoredPanelModel,
}

pub fn new(id: impl Into<SharedString>) -> AnchoredPanelBuilder {
    AnchoredPanelBuilder::new(id)
}

impl AnchoredPanelBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: AnchoredPanelModel {
                id: id.into(),
                content: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                anchor_bounds: None,
                placement: AnchoredPanelPlacement::BelowStart,
                dismiss_policy: AnchoredPanelDismissPolicy::KeepOpen,
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
        F: Fn(&AnchoredPanelRenderModel<'_>, &mut Window, &mut App) -> AnyElement + 'static,
    {
        self.model.content = Arc::new(content);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn anchor_bounds(mut self, bounds: Bounds<Pixels>) -> Self {
        self.model.anchor_bounds = Some(bounds);
        self
    }

    pub fn placement(mut self, placement: AnchoredPanelPlacement) -> Self {
        self.model.placement = placement;
        self
    }

    pub fn dismiss_policy(mut self, dismiss_policy: AnchoredPanelDismissPolicy) -> Self {
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<AnchoredPanel> {
        cx.new(|cx| AnchoredPanel::from_builder(self, cx))
    }
}
