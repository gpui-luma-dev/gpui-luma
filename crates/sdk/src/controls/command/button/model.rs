use std::cell::Cell;
use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, SharedString, div, prelude::*};

use super::control::Button;
use super::template::ButtonTemplate;
pub use crate::controls::presenter::{ControlPresenter, HasPresenter};
use crate::controls::button_family::{ButtonInteractionState as ButtonState, ButtonKind, ButtonSize};
use crate::controls::button_family::ButtonFamilyRole;
use lucide_icons::Icon as LucideIcon;

#[derive(Clone)]
pub enum ControlIcon {
    Lucide(LucideIcon),
    SvgPath(SharedString),
}

impl From<LucideIcon> for ControlIcon {
    fn from(icon: LucideIcon) -> Self {
        Self::Lucide(icon)
    }
}

#[derive(Clone)]
pub struct ButtonModel<D = ()> {
    pub(crate) id: SharedString,
    pub(crate) data: D,
    pub(crate) content: ControlPresenter<ButtonRenderModel<D>>,
    pub(crate) kind: ButtonKind,
    pub(crate) role: ButtonFamilyRole,
    pub(crate) size: ButtonSize,
    pub(crate) enabled: bool,
    pub(crate) tab_stop: bool,
    pub(crate) round: bool,
    pub(crate) template: Arc<dyn ButtonTemplate<D>>,
}

pub struct ButtonRenderModel<D> {
    pub id: SharedString,
    pub data: D,
    pub content: ControlPresenter<ButtonRenderModel<D>>,
    pub kind: ButtonKind,
    pub role: ButtonFamilyRole,
    pub size: ButtonSize,
    pub state: ButtonState,
    pub round: bool,
    pub radius_override: Cell<Option<f32>>,
}

pub struct ButtonBuilder<D = ()> {
    pub(crate) model: ButtonModel<D>,
}

impl ButtonBuilder<()> {
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<()> {
        let id = id.into();

        ButtonBuilder {
            model: ButtonModel {
                id: id.clone(),
                data: (),
                content: Arc::new(move |_, _| div().child(id.clone()).into_any_element()),
                kind: ButtonKind::Standard,
                role: ButtonFamilyRole::Text,
                size: ButtonSize::Md,
                enabled: true,
                tab_stop: true,
                round: false,
                template: super::template::default_button_template(),
            },
        }
    }
}

impl<D: Clone + 'static> ButtonBuilder<D> {
    pub fn data<NewD: Clone + 'static>(self, data: NewD) -> ButtonBuilder<NewD> {
        let old = self.model;
        let id = old.id.clone();
        ButtonBuilder {
            model: ButtonModel {
                id: old.id,
                data: data.clone(),
                content: Arc::new(move |_, _| div().child(id.clone()).into_any_element()),
                kind: old.kind,
                role: old.role,
                size: old.size,
                enabled: old.enabled,
                tab_stop: old.tab_stop,
                round: old.round,
                template: super::template::default_button_template(),
            },
        }
    }

    pub fn kind(mut self, kind: ButtonKind) -> Self {
        self.model.kind = kind;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn role(mut self, role: ButtonFamilyRole) -> Self {
        self.model.role = role;
        self
    }

    pub fn round(mut self, round: bool) -> Self {
        self.model.round = round;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    /// Sets whether this button participates in Tab-key focus traversal.
    ///
    /// Setting this to `false` removes the button from the tab order, but does not
    /// prevent mouse interaction or programmatic focus. This is useful when composing
    /// buttons inside a container control that owns keyboard focus/navigation.
    pub fn tab_stop(mut self, tab_stop: bool) -> Self {
        self.model.tab_stop = tab_stop;
        self
    }

    pub fn template(mut self, template: Arc<dyn ButtonTemplate<D>>) -> Self {
        self.model.template = template;
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<Button<D>> {
        cx.new(|cx| Button::from_builder(self, cx))
    }
}

impl<D: 'static> HasPresenter<ButtonRenderModel<D>> for ButtonBuilder<D> {
    fn set_presenter(&mut self, content: ControlPresenter<ButtonRenderModel<D>>) {
        self.model.content = content;
    }
}
