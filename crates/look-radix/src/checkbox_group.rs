//! Multiple-selection Radix checkbox groups using SDK group state and input.

use std::sync::Arc;
use gpui::{Context, SharedString, div, prelude::*};
use gpui_luma::controls::button::ButtonRenderModel;
use gpui_luma::controls::checkbox::CheckboxData;
use gpui_luma::controls::control_group::{
    self, ControlGroupBuilder, ControlGroupItem, ControlGroupItemLike, ControlGroupItemHandlerExt, ControlGroupLayout,
    ControlGroupTemplate,
};
use crate::{Look, Paint, CheckboxSize, CheckboxVariant, Radius, checkbox_template_for};
use crate::look::resolve_look;

/// Checkbox rows with SDK-owned focus, disabled state and multiple selection.
pub fn checkbox_group_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
) -> ControlGroupTemplate<T> {
    let template = checkbox_template_for(look, variant, paint, size, radius);
    Arc::new(move |model, handlers, window, cx| {
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let label = item.item.label().clone();
                let button = ButtonRenderModel {
                    id: format!("{}-{}", item.group_id, item.item.id()).into(),
                    data: CheckboxData::new(item.selected),
                    content: Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
                    state: item.state.interaction_state(),
                    size: size.control_size(),
                    ..Default::default()
                };
                let selector = button.id.to_string();
                template
                    .render(&button, window, cx)
                    .debug_selector(move || selector.clone())
                    .control_group_item_handlers(handlers)
                    .into_any_element()
            })
            .collect::<Vec<_>>();
        div()
            .id(model.id.clone())
            .flex()
            .gap_2()
            .items_start()
            .when(model.layout == ControlGroupLayout::Vertical, |group| group.flex_col())
            .children(rows)
    })
}

/// Look-owned checkbox group. Subscribe to SDK `ControlGroupEvent` for changes.
pub struct CheckboxGroup<T: ControlGroupItemLike + 'static = ControlGroupItem> {
    look: Option<Look>,
    variant: CheckboxVariant,
    paint: Paint,
    size: CheckboxSize,
    radius: Radius,
    builder: ControlGroupBuilder<T>,
}
impl<T: ControlGroupItemLike + 'static> CheckboxGroup<T> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            variant: CheckboxVariant::default(),
            paint: Paint::accent(),
            size: CheckboxSize::default(),
            radius: Radius::default(),
            builder: control_group::new(id).multiple().independent_toggles(true),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: CheckboxVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
    pub fn size(mut self, size: CheckboxSize) -> Self {
        self.size = size;
        self
    }
    pub fn radius(mut self, radius: Radius) -> Self {
        self.radius = radius;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.builder = self.builder.horizontal();
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }
    pub fn selected_ids<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SharedString>,
    {
        self.builder = self.builder.selected_ids(ids);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> control_group::ControlGroup<T> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        self.builder
            .template(checkbox_group_template(&look, self.variant, self.paint, self.size, self.radius))
            .spawn(cx)
    }
}
