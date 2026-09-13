//! Look-owned tabs builder. Spawn synthesizes the SDK [`luma::controls::tabs::Tabs`].

use std::sync::Arc;

use gpui::{App, Context, Div, Entity, SharedString, Stateful};
use luma::controls::tabs::{TabsBuilder, TabsItem, TabsRenderModel, TabsTemplate, TabsWidthMode};
use luma::infra::icon::DisclosureIcons;
use crate::look::{ShadcnLook, resolve_look_from};
use crate::size::ShadcnSize;

type TabsModifier = Box<dyn Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Builder in the guise of tabs: Shadcn axes plus SDK options, until `.spawn(cx)`.
pub struct Tabs {
    id: SharedString,
    look: Option<ShadcnLook>,
    size: ShadcnSize,
    width_mode: TabsWidthMode,
    items: Vec<TabsItem>,
    active_id: Option<SharedString>,
    enabled: bool,
    animated: bool,
    disclosure_icons: Option<DisclosureIcons>,
    template: Option<Arc<dyn TabsTemplate>>,
    modifiers: Vec<TabsModifier>,
}

impl Tabs {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            size: ShadcnSize::Md,
            width_mode: TabsWidthMode::default(),
            items: Vec::new(),
            active_id: None,
            enabled: true,
            animated: true,
            disclosure_icons: None,
            template: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &ShadcnLook) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn size(mut self, size: ShadcnSize) -> Self {
        self.size = size;
        self
    }

    pub fn width_mode(mut self, width_mode: TabsWidthMode) -> Self {
        self.width_mode = width_mode;
        self
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.active_id = Some(active_id.into());
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.disclosure_icons = Some(icons);
        self
    }

    pub fn template(mut self, template: Arc<dyn TabsTemplate>) -> Self {
        self.template = Some(template);
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &TabsRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<luma::controls::tabs::Tabs> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TabsBuilder {
        let template = self.template.unwrap_or_else(|| look.tabs_template());
        let mut builder = luma::controls::tabs::Tabs::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .width_mode(self.width_mode)
            .items(self.items)
            .enabled(self.enabled)
            .animated(self.animated);
        if let Some(active_id) = self.active_id {
            builder = builder.active(active_id);
        }
        if let Some(icons) = self.disclosure_icons {
            builder = builder.disclosure_icons(icons);
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
    fn items_keep_shadcn_axes() {
        let tabs = Tabs::new("preview")
            .size(ShadcnSize::Sm)
            .active("colors")
            .items([TabsItem::new("colors").label("Colors")]);
        assert_eq!(tabs.size, ShadcnSize::Sm);
        assert_eq!(tabs.active_id.as_deref(), Some("colors"));
        assert_eq!(tabs.items.len(), 1);
    }

    #[test]
    fn into_sdk_builder_does_not_panic() {
        let look = ShadcnLook::built_in();
        let _builder = Tabs::new("ok")
            .look(&look)
            .items([TabsItem::new("one").label("One")])
            .active("one")
            .with_template_modifier(|root, _| root)
            .into_sdk_builder(look);
    }
}
