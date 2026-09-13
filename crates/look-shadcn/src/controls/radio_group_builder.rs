//! Look-owned radio-group builder. Spawn synthesizes the SDK [`luma::controls::radio_group::RadioGroup`].

use gpui::{Context, SharedString};
use luma::controls::control_group::{ControlGroupItemLike, ControlGroupTemplate};
use luma::controls::radio_group::{
    RadioGroupBuilder, RadioGroupLayout, radio_group_button_item_element_template, radio_group_buttons_template,
};

use super::button::ShadcnButtonStyle;
use crate::look::{ShadcnLook, resolve_look_from};

/// Builder in the guise of a radio group: Shadcn style plus SDK options, until `.spawn(cx)`.
pub struct RadioGroup<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    look: Option<ShadcnLook>,
    style: ShadcnButtonStyle,
    horizontal: bool,
    builder: RadioGroupBuilder<T>,
    custom_template: bool,
}

impl<T> RadioGroup<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            style: ShadcnButtonStyle::Primary,
            horizontal: false,
            builder: luma::controls::radio_group::new(id),
            custom_template: false,
        }
    }

    pub fn horizontal(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            style: ShadcnButtonStyle::Primary,
            horizontal: true,
            builder: luma::controls::radio_group::horizontal(id),
            custom_template: false,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn primary(mut self) -> Self {
        self.style = ShadcnButtonStyle::Primary;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.style = ShadcnButtonStyle::Secondary;
        self
    }

    pub fn outline(mut self) -> Self {
        self.style = ShadcnButtonStyle::Outline;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.style = ShadcnButtonStyle::Ghost;
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

    pub fn selected(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.builder = self.builder.selected(selected_id);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn template(mut self, template: ControlGroupTemplate<T>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> luma::controls::radio_group::RadioGroup<T> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> RadioGroupBuilder<T> {
        let radio_template = look.radio_button_template(self.style);
        let mut builder =
            self.builder.item_element_template(radio_group_button_item_element_template(radio_template.clone()));
        if !self.custom_template {
            let layout = if self.horizontal {
                RadioGroupLayout::Horizontal
            } else {
                RadioGroupLayout::Vertical
            };
            builder = builder.template(radio_group_buttons_template(radio_template, layout));
        }
        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luma::controls::radio_group::RadioGroupItem;

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = RadioGroup::new("ok")
            .look(&look)
            .secondary()
            .items([RadioGroupItem::new("a").label("A")])
            .selected("a")
            .into_sdk_builder(look);
    }
}
