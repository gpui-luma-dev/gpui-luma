//! Look-owned autocomplete builder. Spawn synthesizes the SDK [`gpui_luma::controls::autocomplete::Autocomplete`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use gpui_luma::controls::autocomplete::{AutocompleteBuilder, AutocompleteControl, SelectionItem};
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of an autocomplete: Shadcn templates plus SDK options, until `.spawn(cx)`.
pub struct Autocomplete {
    look: Option<ShadcnLook>,
    builder: AutocompleteBuilder,
    size: ShadcnSize,
}

impl Autocomplete {
    pub fn new(id: impl Into<SharedString>, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        Self { look: None, builder: gpui_luma::controls::autocomplete::new(id, items), size: ShadcnSize::Md }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectionItem>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.builder = self.builder.placeholder(placeholder);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn wheel_scroll_policy(mut self, policy: gpui_luma::interaction::WheelScrollPolicy) -> Self {
        self.builder = self.builder.wheel_scroll_policy(policy);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn scroll_boundary_policy(mut self, policy: gpui_luma::interaction::ScrollBoundaryPolicy) -> Self {
        self.builder = self.builder.scroll_boundary_policy(policy);
        self
    }

    /// Configure popup wheel routing independently.
    pub fn wheel_focus_scope(mut self, policy: gpui_luma::interaction::WheelFocusScope) -> Self {
        self.builder = self.builder.wheel_focus_scope(policy);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn invalid(mut self, invalid: bool) -> Self {
        self.builder = self.builder.invalid(invalid);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.builder = self.builder.full_width(full_width);
        self
    }

    pub fn clean_on_escape(mut self, clean_on_escape: bool) -> Self {
        self.builder = self.builder.clean_on_escape(clean_on_escape);
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.builder = self.builder.scrolling(scrolling);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<AutocompleteControl> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> AutocompleteBuilder {
        let theme = look.clone();
        self.builder
            .size(self.size.control_size())
            .textfield_template(theme.primary_textfield_template())
            .autocomplete_theme(theme.autocomplete_theme())
            .scrollbar_template(theme.scrollbar_template())
            .popup_look_provider(Arc::new(move |size| {
                theme.selector_items_panel_look(ShadcnSize::from_control_size(size))
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Autocomplete::new("ok", [SelectionItem::new("a", "A")])
            .look(&look)
            .placeholder("Type…")
            .into_sdk_builder(look);
    }
}
