use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, SharedString, Window, point, px};

use super::{DialogControl, DialogTemplate, default_dialog_template};
use crate::controls::command::button::{ButtonTemplate, default_button_template};
use crate::theme::ControlSize;

pub type DialogElementRenderer = Arc<dyn Fn(&DialogRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogMode {
    #[default]
    Modal,
    Modeless,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogDismissPolicy {
    #[default]
    KeepOpen,
    CloseOnClickAway,
    CloseOnFocusLoss,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DialogPosition {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Absolute(gpui::Point<gpui::Pixels>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DialogEvent {
    Opened,
    Dismissed,
    Confirmed,
}

#[derive(Clone)]
pub struct DialogModel {
    pub(crate) id: SharedString,
    pub(crate) title: SharedString,
    pub(crate) description: Option<SharedString>,
    pub(crate) body: DialogElementRenderer,
    pub(crate) open: bool,
    pub(crate) mode: DialogMode,
    pub(crate) position: DialogPosition,
    pub(crate) dismiss_policy: DialogDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) draggable: bool,
    pub(crate) show_footer: bool,
    pub(crate) size: ControlSize,
    pub(crate) width: Option<f32>,
    pub(crate) confirm_label: SharedString,
    pub(crate) cancel_label: SharedString,
    pub(crate) template: Arc<dyn DialogTemplate>,
    pub(crate) confirm_button_template: Arc<dyn ButtonTemplate<()>>,
    pub(crate) cancel_button_template: Arc<dyn ButtonTemplate<()>>,
    pub(crate) close_button_template: Arc<dyn ButtonTemplate<()>>,
}

pub struct DialogRenderModel<'a> {
    pub id: &'a SharedString,
    pub title: &'a SharedString,
    pub description: Option<&'a SharedString>,
    pub mode: DialogMode,
    pub position: DialogPosition,
    pub dismissible: bool,
    pub draggable: bool,
    pub show_footer: bool,
    pub size: ControlSize,
    pub width: Option<f32>,
    pub focused: bool,
    pub body: &'a DialogElementRenderer,
}

pub struct DialogBuilder {
    pub(crate) model: DialogModel,
}

pub fn new(id: impl Into<SharedString>) -> DialogBuilder {
    DialogBuilder::new(id)
}

impl DialogBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            model: DialogModel {
                id: id.clone(),
                title: id.clone(),
                description: None,
                body: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                mode: DialogMode::Modal,
                position: DialogPosition::Center,
                dismiss_policy: DialogDismissPolicy::KeepOpen,
                dismissible: true,
                draggable: false,
                show_footer: true,
                size: ControlSize::Md,
                width: None,
                confirm_label: SharedString::from("OK"),
                cancel_label: SharedString::from("Cancel"),
                template: default_dialog_template(),
                confirm_button_template: default_button_template(),
                cancel_button_template: default_button_template(),
                close_button_template: default_button_template(),
            },
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = title.into();
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.model.description = Some(description.into());
        self
    }

    pub fn clear_description(mut self) -> Self {
        self.model.description = None;
        self
    }

    pub fn body<F>(mut self, body: F) -> Self
    where
        F: Fn(&DialogRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.model.body = Arc::new(body);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.model.open = open;
        self
    }

    pub fn mode(mut self, mode: DialogMode) -> Self {
        self.model.mode = mode;
        self
    }

    pub fn position(mut self, position: DialogPosition) -> Self {
        self.model.position = position;
        self
    }

    pub fn absolute_position(mut self, x: f32, y: f32) -> Self {
        self.model.position = DialogPosition::Absolute(point(px(x), px(y)));
        self
    }

    pub fn dismiss_policy(mut self, dismiss_policy: DialogDismissPolicy) -> Self {
        self.model.dismiss_policy = dismiss_policy;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.model.dismissible = dismissible;
        self
    }

    pub fn draggable(mut self, draggable: bool) -> Self {
        self.model.draggable = draggable;
        self
    }

    pub fn show_footer(mut self, show_footer: bool) -> Self {
        self.model.show_footer = show_footer;
        self
    }

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.model.width = Some(width);
        self
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.cancel_label = label.into();
        self
    }

    pub fn template(mut self, template: Arc<dyn DialogTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn confirm_button_template(mut self, template: Arc<dyn ButtonTemplate<()>>) -> Self {
        self.model.confirm_button_template = template;
        self
    }

    pub fn cancel_button_template(mut self, template: Arc<dyn ButtonTemplate<()>>) -> Self {
        self.model.cancel_button_template = template;
        self
    }

    pub fn close_button_template(mut self, template: Arc<dyn ButtonTemplate<()>>) -> Self {
        self.model.close_button_template = template;
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<DialogControl> {
        cx.new(|cx| DialogControl::from_builder(self, cx))
    }
}
