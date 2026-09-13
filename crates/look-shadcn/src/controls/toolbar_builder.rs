//! Look-owned toolbar builder. Spawn synthesizes the SDK [`luma::controls::toolbar::ToolbarControl`].

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use luma::controls::control_group::ControlGroupFocusStrategy;
use luma::controls::toolbar::{ToolbarBuilder, ToolbarItem, ToolbarRenderModel, ToolbarVariant};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type ToolbarModifier = Box<dyn Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of a toolbar: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Toolbar {
    id: SharedString,
    look: Option<ShadcnLook>,
    items: Vec<ToolbarItem>,
    enabled: bool,
    size: ShadcnSize,
    variant: ToolbarVariant,
    focus_strategy: ControlGroupFocusStrategy,
    modifiers: Vec<ToolbarModifier>,
}

impl Toolbar {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            items: Vec::new(),
            enabled: true,
            size: ShadcnSize::Md,
            variant: ToolbarVariant::Outline,
            focus_strategy: ControlGroupFocusStrategy::RovingItemFocus,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn separator(self, id: impl Into<SharedString>) -> Self {
        self.item(ToolbarItem::separator(id))
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn variant(mut self, variant: ToolbarVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn outline(self) -> Self {
        self.variant(ToolbarVariant::Outline)
    }

    pub fn ghost(self) -> Self {
        self.variant(ToolbarVariant::Ghost)
    }

    pub fn focus_strategy(mut self, strategy: ControlGroupFocusStrategy) -> Self {
        self.focus_strategy = strategy;
        self
    }

    pub fn roving_item_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::RovingItemFocus)
    }

    pub fn sequential_focus(self) -> Self {
        self.focus_strategy(ControlGroupFocusStrategy::ActiveDescendant)
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ToolbarRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<luma::controls::toolbar::ToolbarControl> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ToolbarBuilder {
        let mut builder = luma::controls::toolbar::new(self.id)
            .template(look.toolbar_template())
            .items(self.items)
            .enabled(self.enabled)
            .size(self.size.control_size())
            .variant(self.variant)
            .focus_strategy(self.focus_strategy);
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
    fn variant_helpers_set_ghost() {
        let toolbar = Toolbar::new("editor").ghost().size(ShadcnSize::Sm);
        assert_eq!(toolbar.variant, ToolbarVariant::Ghost);
        assert_eq!(toolbar.size, ShadcnSize::Sm);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Toolbar::new("ok").look(&look).outline().into_sdk_builder(look);
    }
}
