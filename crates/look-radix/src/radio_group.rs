//! Standard Radix radio groups backed by SDK selection and keyboard navigation.

use std::sync::Arc;
use gpui::{Context, SharedString, div, prelude::*};
use gpui_luma::controls::button::ButtonRenderModel;
use gpui_luma::controls::radio_button::RadioButtonData;
use gpui_luma::controls::control_group::{ControlGroupItem, ControlGroupItemLike, ControlGroupItemHandlerExt};
use gpui_luma::controls::radio_group::{
    RadioGroupBuilder, RadioGroupLayout, RadioGroupTemplate, radio_group_button_item_element_template,
};
use crate::{Look, Paint, RadioSize, RadioVariant, radio_template_for};
use crate::look::resolve_look;

/// Standard (non-segmented) radio template.
pub fn radio_group_template<T: ControlGroupItemLike + 'static>(
    look: &Look,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
) -> RadioGroupTemplate<T> {
    let template = radio_template_for(look, variant, paint, size);
    Arc::new(move |model, handlers, window, cx| {
        let rows = model
            .items
            .iter()
            .zip(handlers.into_item_handlers())
            .map(|(item, handlers)| {
                let label = item.item.label().clone();
                let button = ButtonRenderModel {
                    id: format!("{}-{}", item.group_id, item.item.id()).into(),
                    data: RadioButtonData::new(item.selected),
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
            .when(layout == RadioGroupLayout::Vertical, |group| group.flex_col())
            .children(rows)
    })
}

/// Look-owned single-selection group. Events and state are owned by the SDK.
pub struct RadioGroup<T: ControlGroupItemLike + 'static = ControlGroupItem> {
    look: Option<Look>,
    variant: RadioVariant,
    paint: Paint,
    size: RadioSize,
    layout: RadioGroupLayout,
    builder: RadioGroupBuilder<T>,
}

impl<T: ControlGroupItemLike + 'static> RadioGroup<T> {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            variant: RadioVariant::default(),
            paint: Paint::accent(),
            size: RadioSize::default(),
            layout: RadioGroupLayout::Vertical,
            builder: gpui_luma::controls::radio_group::new(id).selection_follows_active(true),
        }
    }
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }
    pub fn variant(mut self, variant: RadioVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn paint(mut self, paint: Paint) -> Self {
        self.paint = paint;
        self
    }
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }
    pub fn horizontal(mut self) -> Self {
        self.layout = RadioGroupLayout::Horizontal;
        self.builder = self.builder.horizontal();
        self
    }
    pub fn items(mut self, items: impl IntoIterator<Item = T>) -> Self {
        self.builder = self.builder.items(items);
        self
    }
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(id);
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }
    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> gpui_luma::controls::radio_group::RadioGroup<T> {
        let look = resolve_look(self.look.as_ref(), cx.try_global::<Look>());
        let item =
            radio_group_button_item_element_template(radio_template_for(&look, self.variant, self.paint, self.size));
        self.builder
            .item_element_template(item)
            .template(radio_group_template(&look, self.variant, self.paint, self.size, self.layout))
            .spawn(cx)
    }
}
