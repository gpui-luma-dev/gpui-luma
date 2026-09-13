//! Look-owned context-menu builder. Spawn synthesizes the SDK [`luma::controls::context_menu::ContextMenu`].

use gpui::{App, Context, Div, Entity, IntoElement, SharedString, Stateful};
use luma::controls::context_menu::{ContextMenuBuilder, ContextMenuRenderModel};
use luma::infra::menu_item::MenuItem;

use crate::look::{ShadcnLook, resolve_look_from};

type ContextMenuModifier = Box<dyn Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a context menu: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct ContextMenu {
    id: SharedString,
    look: Option<ShadcnLook>,
    label: Option<SharedString>,
    items: Vec<MenuItem>,
    enabled: bool,
    target: Option<luma::controls::context_menu::ContextMenuTargetContent>,
    modifiers: Vec<ContextMenuModifier>,
}

impl ContextMenu {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            label: None,
            items: Vec::new(),
            enabled: true,
            target: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn target_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut gpui::App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.target = Some(std::sync::Arc::new(move |cx| content(cx).into_any_element()));
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<luma::controls::context_menu::ContextMenu> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ContextMenuBuilder {
        let mut builder = luma::controls::context_menu::ContextMenu::new(self.id)
            .template(look.context_menu_template())
            .items(self.items)
            .enabled(self.enabled);
        if let Some(label) = self.label {
            builder = builder.label(label);
        }
        if let Some(target) = self.target {
            builder = builder.target_content(move |cx| target(cx));
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_keep_id() {
        let menu = ContextMenu::new("ctx").items([MenuItem::new("one").label("One")]);
        assert_eq!(menu.items.len(), 1);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = ContextMenu::new("ok")
            .look(&look)
            .items([MenuItem::new("one").label("One")])
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
