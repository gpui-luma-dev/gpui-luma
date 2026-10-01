//! Look-owned accordion builder. Spawn synthesizes the SDK [`gpui_luma::controls::accordion::Accordion`].

use std::sync::Arc;

use gpui::{Context, Entity, SharedString};
use gpui_luma::controls::accordion::{
    AccordionBuilder, AccordionControl, AccordionItem, AccordionSelectionMode, AccordionTemplate,
};
use gpui_luma::infra::icon::DisclosureIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

/// Builder in the guise of an accordion: Shadcn template plus SDK options, until `.spawn(cx)`.
pub struct Accordion {
    look: Option<ShadcnLook>,
    builder: AccordionBuilder,
    custom_template: bool,
    size: ShadcnSize,
}

impl Accordion {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            look: None,
            builder: gpui_luma::controls::accordion::new(id),
            custom_template: false,
            size: ShadcnSize::Md,
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn mode(mut self, mode: AccordionSelectionMode) -> Self {
        self.builder = self.builder.mode(mode);
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.builder = self.builder.animated(animated);
        self
    }

    pub fn single(mut self) -> Self {
        self.builder = self.builder.single();
        self
    }

    pub fn multiple(mut self) -> Self {
        self.builder = self.builder.multiple();
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.builder = self.builder.collapsible(collapsible);
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.builder = self.builder.item(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.builder = self.builder.items(items);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.builder = self.builder.enabled(enabled);
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn item_dividers(mut self, item_dividers: bool) -> Self {
        self.builder = self.builder.item_dividers(item_dividers);
        self
    }

    pub fn content_padding_y(mut self, padding_y: f32) -> Self {
        self.builder = self.builder.content_padding_y(padding_y);
        self
    }

    pub fn content_padding_top(mut self, padding_top: f32) -> Self {
        self.builder = self.builder.content_padding_top(padding_top);
        self
    }

    pub fn content_padding_bottom(mut self, padding_bottom: f32) -> Self {
        self.builder = self.builder.content_padding_bottom(padding_bottom);
        self
    }

    pub fn trigger_min_height(mut self, height: f32) -> Self {
        self.builder = self.builder.trigger_min_height(height);
        self
    }

    pub fn trigger_padding_y(mut self, padding_y: f32) -> Self {
        self.builder = self.builder.trigger_padding_y(padding_y);
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.builder = self.builder.disclosure_icons(icons);
        self
    }

    pub fn template(mut self, template: Arc<dyn AccordionTemplate>) -> Self {
        self.custom_template = true;
        self.builder = self.builder.template(template);
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<AccordionControl> {
        let look = resolve_look_from(self.look.as_ref(), cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> AccordionBuilder {
        let builder = self.builder.size(self.size.control_size());
        if self.custom_template {
            builder
        } else {
            builder.template(look.accordion_template())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::controls::accordion::{AccordionContent, AccordionTrigger};

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Accordion::new("ok")
            .look(&look)
            .single()
            .item(AccordionItem::new("one", AccordionTrigger::new("One"), AccordionContent::new("Body")))
            .into_sdk_builder(look);
    }
}
