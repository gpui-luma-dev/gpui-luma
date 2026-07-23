use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, SharedString, Window, point, px};

use super::{DialogControl, DialogTemplate, default_dialog_template};
use super::template::modified_dialog_template;
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
#[non_exhaustive]
pub enum DialogEvent {
    Opened,
    Dismissed,
}

#[derive(Clone)]
pub struct DialogModel {
    pub(crate) id: SharedString,
    pub(crate) content: DialogElementRenderer,
    pub(crate) open: bool,
    pub(crate) mode: DialogMode,
    pub(crate) position: DialogPosition,
    pub(crate) dismiss_policy: DialogDismissPolicy,
    pub(crate) dismissible: bool,
    pub(crate) draggable: bool,
    pub(crate) size: ControlSize,
    pub(crate) width: Option<f32>,
    pub(crate) template: Arc<dyn DialogTemplate>,
}

pub struct DialogRenderModel<'a> {
    pub id: &'a SharedString,
    pub mode: DialogMode,
    pub position: DialogPosition,
    pub dismissible: bool,
    pub draggable: bool,
    pub size: ControlSize,
    pub width: Option<f32>,
    pub focused: bool,
    pub content: &'a DialogElementRenderer,
}

pub(crate) struct ThemeInvalidator(Box<dyn Fn(&mut App) + Send>);

impl ThemeInvalidator {
    pub fn from_entity<T: 'static>(entity: Entity<T>) -> Self {
        Self(Box::new(move |cx| {
            entity.update(cx, |_, cx| cx.notify());
        }))
    }

    pub fn invalidate(&self, cx: &mut App) {
        (self.0)(cx);
    }
}

pub struct DialogBuilder {
    pub(crate) model: DialogModel,
    pub(crate) theme_children: Vec<ThemeInvalidator>,
}

pub fn new(id: impl Into<SharedString>) -> DialogBuilder {
    DialogBuilder::new(id)
}

impl DialogBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: DialogModel {
                id: id.into(),
                content: Arc::new(|_, _, _| gpui::div().into_any_element()),
                open: false,
                mode: DialogMode::Modal,
                position: DialogPosition::Center,
                dismiss_policy: DialogDismissPolicy::KeepOpen,
                dismissible: true,
                draggable: false,
                size: ControlSize::Md,
                width: None,
                template: default_dialog_template(),
            },
            theme_children: Vec::new(),
        }
    }

    /// Register a persistent entity hosted in this overlay for theme invalidation fan-out.
    ///
    /// Spawn child controls before the overlay, then register them here so they re-render when
    /// [`crate::theme::LumaThemeRevision`] changes without app-level notify plumbing.
    pub fn theme_child<T: 'static>(mut self, entity: Entity<T>) -> Self {
        self.theme_children.push(ThemeInvalidator::from_entity(entity));
        self
    }

    pub fn theme_children<I, T>(mut self, entities: I) -> Self
    where
        I: IntoIterator<Item = Entity<T>>,
        T: 'static,
    {
        for entity in entities {
            self.theme_children.push(ThemeInvalidator::from_entity(entity));
        }
        self
    }

    pub fn content<F>(mut self, content: F) -> Self
    where
        F: Fn(&DialogRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.model.content = Arc::new(content);
        self
    }

    pub fn body<F>(self, body: F) -> Self
    where
        F: Fn(&DialogRenderModel<'_>, &mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.content(body)
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

    pub fn size(mut self, size: ControlSize) -> Self {
        self.model.size = size;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.model.width = Some(width);
        self
    }

    pub fn template(mut self, template: Arc<dyn DialogTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &DialogRenderModel<'_>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.template = modified_dialog_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<DialogControl> {
        cx.new(|cx| DialogControl::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_dialog_template();
        let builder = DialogBuilder::new("dialog-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }

    #[test]
    fn theme_children_starts_empty() {
        let builder = DialogBuilder::new("dialog-test");
        assert!(builder.theme_children.is_empty());
    }
}
