use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Entity, IntoElement, Pixels, Point, SharedString};

use super::{ContextMenu, ContextMenuTemplate, ControlFocusState, MenuPath, default_context_menu_template};
use crate::theme::InteractionState;
use super::template::modified_context_menu_template;
use crate::infra::menu_item::MenuItem;
use crate::motion::overlay_presence::OverlayPresence;

#[derive(Clone)]
pub struct ContextMenuModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) target_content: Option<ContextMenuTargetContent>,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) enabled: bool,
    pub(crate) template: Arc<dyn ContextMenuTemplate>,
}

pub type ContextMenuTargetContent = Arc<dyn Fn(&mut App) -> AnyElement + Send + Sync + 'static>;

pub struct ContextMenuRenderModel<'a> {
    pub id: &'a SharedString,
    pub label: &'a SharedString,
    pub target_content: Option<&'a ContextMenuTargetContent>,
    pub items: &'a [MenuItem],
    pub menu_position: Option<Point<Pixels>>,
    pub presence: OverlayPresence,
    pub open_submenu: Option<usize>,
    pub active_path: Option<MenuPath>,
    pub submenu_presence: OverlayPresence,
    pub submenu_transition: Option<(usize, f32)>,
    pub enabled: bool,
    pub focus: ControlFocusState,
    pub state: InteractionState,
}

pub struct ContextMenuBuilder {
    pub(crate) model: ContextMenuModel,
}

impl ContextMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();

        Self {
            model: ContextMenuModel {
                label: id.clone(),
                id,
                target_content: None,
                items: Vec::new(),
                enabled: true,
                template: default_context_menu_template(),
            },
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = label.into();
        self
    }

    pub fn target_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.model.target_content = Some(Arc::new(move |cx| content(cx).into_any_element()));
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.model.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.model.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn template(mut self, template: Arc<dyn ContextMenuTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>, &ContextMenuRenderModel<'_>) -> gpui::Stateful<gpui::Div>
            + Send
            + Sync
            + 'static,
    {
        self.model.template = modified_context_menu_template(Arc::clone(&self.model.template), modifier);
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<ContextMenu> {
        cx.new(|cx| ContextMenu::from_builder(self, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_template_modifier_wraps_template() {
        let template = default_context_menu_template();
        let builder = ContextMenuBuilder::new("context-test")
            .template(template.clone())
            .with_template_modifier(|element, _| element);

        assert!(!Arc::ptr_eq(&builder.model.template, &template));
    }
}
