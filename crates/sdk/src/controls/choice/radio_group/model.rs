use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Div, Entity, IntoElement, SharedString, Stateful, Window};

use crate::controls::control_group::{
    self as control_group, ControlGroupBuilder as InnerControlGroupBuilder, ControlGroupChromeModel,
    ControlGroupFocusStrategy, ControlGroupFocusTarget, ControlGroupFocusTargetProvider, ControlGroupItem,
    ControlGroupItemElementTemplate, ControlGroupItemElements, ControlGroupItemLike, ControlGroupItemRenderModel,
    ControlGroupItemTemplate, ControlGroupItemVisualContext, ControlGroupLayout, ControlGroupRenderModel,
    ControlGroupTemplate, ControlGroupTemplateHandlers, ControlGroupTheme, ControlSelectionMode,
};
use crate::theme::ControlSize;

use super::control::RadioGroupControl;
use super::template::radio_group_container_template;

pub type RadioGroup<T> = Entity<RadioGroupControl<T>>;
pub type RadioGroupItem = ControlGroupItem;
pub type RadioGroupRenderModel<'a, T> = ControlGroupRenderModel<'a, T>;
pub type RadioGroupTemplate<T> = ControlGroupTemplate<T>;
pub type RadioGroupTemplateHandlers = ControlGroupTemplateHandlers;
pub type SelectionMode = ControlSelectionMode;
pub use crate::controls::control_group::ControlGroupItemLike as RadioGroupItemLike;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum RadioGroupEvent {
    Change { value: Option<SharedString> },
    Activate { value: SharedString },
    FocusChanged { focused: bool },
    ItemFocused { item_id: SharedString },
}

pub struct RadioGroupBuilder<T>(pub(super) InnerControlGroupBuilder<T>)
where
    T: ControlGroupItemLike + 'static;

impl<T> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    fn from_inner(inner: InnerControlGroupBuilder<T>) -> Self {
        Self(inner)
    }

    pub fn layout(self, layout: ControlGroupLayout) -> Self {
        Self(self.0.layout(layout))
    }

    pub fn horizontal(self) -> Self {
        Self(self.0.horizontal())
    }

    pub fn vertical(self) -> Self {
        Self(self.0.vertical())
    }

    pub fn item(self, item: T) -> Self {
        Self(self.0.item(item))
    }

    pub fn items(self, items: impl IntoIterator<Item = T>) -> Self {
        Self(self.0.items(items))
    }

    pub fn mode(self, selection_mode: ControlSelectionMode) -> Self {
        Self(self.0.mode(selection_mode))
    }

    pub fn selection_mode(self, selection_mode: ControlSelectionMode) -> Self {
        Self(self.0.selection_mode(selection_mode))
    }

    pub fn selection_follows_active(self, selection_follows_active: bool) -> Self {
        Self(self.0.selection_follows_active(selection_follows_active))
    }

    pub fn focus_strategy(self, focus_strategy: ControlGroupFocusStrategy) -> Self {
        Self(self.0.focus_strategy(focus_strategy))
    }

    pub fn active_descendant(self) -> Self {
        Self(self.0.active_descendant())
    }

    pub fn roving_item_focus(self) -> Self {
        Self(self.0.roving_item_focus())
    }

    pub fn focus_target_provider(self, provider: ControlGroupFocusTargetProvider<T>) -> Self {
        Self(self.0.focus_target_provider(provider))
    }

    pub fn with_focus_target_provider<F>(self, provider: F) -> Self
    where
        F: for<'a> Fn(&'a T, &mut Window, &mut App) -> Option<ControlGroupFocusTarget> + Send + Sync + 'static,
    {
        Self(self.0.with_focus_target_provider(provider))
    }

    pub fn clear_focus_target_provider(self) -> Self {
        Self(self.0.clear_focus_target_provider())
    }

    pub fn single_required(self) -> Self {
        Self(self.0.single_required())
    }

    pub fn single_allow_none(self) -> Self {
        Self(self.0.single_allow_none())
    }

    pub fn selected(self, selected_id: impl Into<SharedString>) -> Self {
        Self(self.0.selected(selected_id))
    }

    pub fn selected_ids<I, S>(self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        Self(self.0.selected_ids(selected_ids))
    }

    pub fn managed(self) -> Self {
        Self(self.0.managed())
    }

    pub fn unmanaged(self) -> Self {
        Self(self.0.unmanaged())
    }

    pub fn managed_selected(self, selected_id: impl Into<SharedString>) -> Self {
        Self(self.0.managed_selected(selected_id))
    }

    pub fn managed_selected_ids<I, S>(self, selected_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        Self(self.0.managed_selected_ids(selected_ids))
    }

    pub fn active(self, active_id: impl Into<SharedString>) -> Self {
        Self(self.0.active(active_id))
    }

    pub fn enabled(self, enabled: bool) -> Self {
        Self(self.0.enabled(enabled))
    }

    pub fn tab_stop(self, tab_stop: bool) -> Self {
        Self(self.0.tab_stop(tab_stop))
    }

    pub fn item_template(self, template: ControlGroupItemTemplate<T>) -> Self {
        Self(self.0.item_template(template))
    }

    pub fn item_element_template(self, template: ControlGroupItemElementTemplate<T>) -> Self {
        Self(self.0.item_element_template(template))
    }

    pub fn with_item_template<F, E>(self, template: F) -> Self
    where
        F: for<'a> Fn(&ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        Self(self.0.with_item_template(template))
    }

    pub fn with_item_template_modifier<F>(self, modifier: F) -> Self
    where
        F: for<'a> Fn(AnyElement, &ControlGroupItemRenderModel<'a, T>, &mut Window, &mut App) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        Self(self.0.with_item_template_modifier(modifier))
    }

    pub fn clear_item_template(self) -> Self {
        Self(self.0.clear_item_template())
    }

    pub fn clear_item_element_template(self) -> Self {
        Self(self.0.clear_item_element_template())
    }

    pub fn template(self, template: ControlGroupTemplate<T>) -> Self {
        Self(self.0.template(template))
    }

    pub fn with_template_modifier<F>(self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ControlGroupChromeModel) -> Stateful<Div> + Send + Sync + 'static,
    {
        Self(self.0.with_template_modifier(modifier))
    }

    pub fn with_template<F>(self, template: F) -> Self
    where
        F: for<'a> Fn(
                &ControlGroupRenderModel<'a, T>,
                ControlGroupTemplateHandlers,
                &mut Window,
                &mut App,
            ) -> Stateful<Div>
            + Send
            + Sync
            + 'static,
    {
        Self(self.0.with_template(template))
    }

    pub fn with_item_layout<F, E>(self, layout: F) -> Self
    where
        F: for<'a> Fn(ControlGroupItemElements, &ControlGroupRenderModel<'a, T>, &mut Window, &mut App) -> E
            + Send
            + Sync
            + 'static,
        E: IntoElement + 'static,
    {
        Self(self.0.with_item_layout(layout))
    }

    pub fn with_menu_row_item_content<F>(self, theme: Arc<dyn ControlGroupTheme>, content: F) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        self.with_menu_row_item_content_sized(theme, ControlSize::Sm, 27.0, 4.0, content)
    }

    pub fn with_menu_row_item_content_sized<F>(
        self,
        theme: Arc<dyn ControlGroupTheme>,
        size: ControlSize,
        row_height: f32,
        row_radius: f32,
        content: F,
    ) -> Self
    where
        F: for<'a> Fn(
                &'a ControlGroupItemRenderModel<'a, T>,
                &ControlGroupItemVisualContext,
                &mut Window,
                &mut App,
            ) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        Self(self.0.with_menu_row_item_content_sized(theme, size, row_height, row_radius, content))
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> RadioGroup<T> {
        cx.new(|cx| RadioGroupControl::from_builder(self, cx))
    }
}

pub fn new<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    RadioGroupBuilder::from_inner(control_group::new(id).single_required().template(radio_group_container_template()))
}

pub fn horizontal<T>(id: impl Into<SharedString>) -> RadioGroupBuilder<T>
where
    T: ControlGroupItemLike + 'static,
{
    new(id).horizontal()
}
