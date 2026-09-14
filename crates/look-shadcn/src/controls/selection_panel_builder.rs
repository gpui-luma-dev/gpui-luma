//! Look-owned selection-panel builder. Spawn synthesizes the SDK [`luma::controls::selection_panel::SelectionPanelControl`].

use gpui::{App, Context, Entity, IntoElement, SharedString};
use luma::controls::selection_panel::{
    SelectionPanelBuilder, SelectionPanelControl, SelectionPanelItem, SelectionPanelItemLike,
    SelectionPanelItemRenderModel, SelectionPanelLookProvider,
};
use luma::infra::icon::SelectionStatusIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of a selection panel: Shadcn look plus SDK options, until `.spawn(cx)`.
pub struct SelectionPanel<T = SelectionPanelItem>
where
    T: SelectionPanelItemLike + 'static,
{
    look: Option<ShadcnLook>,
    builder: SelectionPanelBuilder<T>,
    custom_look_provider: bool,
    size: ShadcnSize,
}

impl SelectionPanel<SelectionPanelItem> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self::new_typed(id)
    }
}

impl<T> SelectionPanel<T>
where
    T: SelectionPanelItemLike + 'static,
{
    pub fn new_typed(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: SelectionPanelBuilder::new(id),
            custom_look_provider: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn panel_id(mut self, panel_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.panel_id(panel_id);
        self
    }

    pub fn item(mut self, item: T) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn visible_indices(mut self, visible_indices: impl IntoIterator<Item = usize>) -> Self {
        self.builder = self.builder.visible_indices(visible_indices);
        self
    }

    pub fn selected_source_index(mut self, selected_source_index: Option<usize>) -> Self {
        self.builder = self.builder.selected_source_index(selected_source_index);
        self
    }

    pub fn active_visible_index(mut self, active_visible_index: Option<usize>) -> Self {
        self.builder = self.builder.active_visible_index(active_visible_index);
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.builder = self.builder.open(open);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn show_selection_marker(mut self, show_selection_marker: bool) -> Self {
        self.builder = self.builder.show_selection_marker(show_selection_marker);
        self
    }

    pub fn icons(mut self, icons: SelectionStatusIcons) -> Self {
        self.builder = self.builder.icons(icons);
        self
    }

    pub fn scrolling(mut self, scrolling: bool) -> Self {
        self.builder = self.builder.scrolling(scrolling);
        self
    }

    pub fn min_visible_rows(mut self, min_visible_rows: usize) -> Self {
        self.builder = self.builder.min_visible_rows(min_visible_rows);
        self
    }

    pub fn max_visible_rows(mut self, max_visible_rows: usize) -> Self {
        self.builder = self.builder.max_visible_rows(max_visible_rows);
        self
    }

    pub fn visible_row_limits(mut self, min_visible_rows: usize, max_visible_rows: usize) -> Self {
        self.builder = self.builder.visible_row_limits(min_visible_rows, max_visible_rows);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn with_item_template<F, E>(mut self, template: F) -> Self
    where
        F: for<'a> Fn(&SelectionPanelItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.builder = self.builder.with_item_template(template);
        self
    }

    pub fn look_provider(mut self, provider: SelectionPanelLookProvider) -> Self {
        self.custom_look_provider = true;
        self.builder = self.builder.look_provider(provider);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<SelectionPanelControl<T>> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> SelectionPanelBuilder<T> {
        let theme = look.clone();
        let mut builder = self.builder.size(self.size.control_size()).scrollbar_template(theme.scrollbar_template());
        if !self.custom_look_provider {
            builder = builder.look_provider(theme.selection_panel_look_provider());
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
        let _builder = SelectionPanel::new("ok")
            .look(&look)
            .items([SelectionPanelItem::new("a").label("A")])
            .into_sdk_builder(look);
    }

    #[test]
    fn typed_into_sdk_builder_does_not_panic() {
        struct ThemeItem {
            id: SharedString,
            label: SharedString,
        }
        impl SelectionPanelItemLike for ThemeItem {
            fn id(&self) -> &SharedString {
                &self.id
            }
            fn label(&self) -> &SharedString {
                &self.label
            }
        }

        let look = ShadcnLook::built_in();
        let _builder = SelectionPanel::new_typed("ok")
            .look(&look)
            .items([ThemeItem { id: "a".into(), label: "A".into() }])
            .into_sdk_builder(look);
    }
}
