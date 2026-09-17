//! Look-owned listbox builder. Spawn synthesizes the SDK listbox control-group.

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use luma::controls::control_group::{
    ControlGroupBuilder, ControlGroupChromeModel, ControlGroupControl, ControlGroupItemElementTemplate,
    ControlGroupItemTemplate, ControlGroupTemplate,
};
use luma::controls::listbox::{ListBoxItem, multiple as sdk_multiple, new as sdk_new};

use crate::look::{ShadcnLook, resolve_look_from};

type ListBoxTemplateModifier =
    Box<dyn Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static>;

/// Builder in the guise of a listbox: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct ListBox {
    look: Option<ShadcnLook>,
    builder: ControlGroupBuilder<ListBoxItem>,
    custom_template: bool,
    template_modifiers: Vec<ListBoxTemplateModifier>,
}

impl ListBox {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { look: None, builder: sdk_new(id), custom_template: false, template_modifiers: Vec::new() }
    }

    pub fn multiple(id: impl Into<SharedString>) -> Self {
        Self { look: None, builder: sdk_multiple(id), custom_template: false, template_modifiers: Vec::new() }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }

    pub fn single_allow_none(mut self) -> Self {
        self.builder = self.builder.single_allow_none();
        self
    }

    pub fn vertical(mut self) -> Self {
        self.builder = self.builder.vertical();
        self
    }

    pub fn item(mut self, item: ListBoxItem) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ListBoxItem>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(selected_id);
        self
    }

    pub fn selected_ids<I, S>(mut self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.builder = self.builder.selected_ids(selected_ids);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn template(mut self, template: ControlGroupTemplate<ListBoxItem>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn item_template(mut self, template: ControlGroupItemTemplate<ListBoxItem>) -> Self {
        self.builder = self.builder.item_template(template);
        self
    }

    pub fn item_element_template(mut self, template: ControlGroupItemElementTemplate<ListBoxItem>) -> Self {
        self.builder = self.builder.item_element_template(template);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.template_modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<ControlGroupControl<ListBoxItem>> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> ControlGroupBuilder<ListBoxItem> {
        let mut builder = if self.custom_template {
            self.builder
        } else {
            self.builder.template(look.listbox_template())
        };

        for modifier in self.template_modifiers {
            builder = builder.with_template_modifier(modifier);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = ListBox::new("ok")
            .look(&look)
            .items([ListBoxItem::new("a", "a").label("A")])
            .selected_ids(["a"])
            .into_sdk_builder(look);
    }
}
