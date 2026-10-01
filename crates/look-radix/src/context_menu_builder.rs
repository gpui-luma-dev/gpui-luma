//! Look-owned context-menu builder. Spawn synthesizes the SDK [`luma::controls::context_menu::ContextMenu`].

use gpui::{App, Context, Div, Entity, IntoElement, SharedString, Stateful};
use luma::controls::context_menu::{ContextMenuBuilder, ContextMenuRenderModel};
use luma::infra::menu_item::MenuItem;

use crate::look::{Look, resolve_look};
use crate::{ContextMenuVariant, Tone, context_menu_template};

type ContextMenuModifier = Box<dyn Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync>;

/// Radix context menu backed by SDK input, focus, and submenu behavior.
///
/// ```no_run
/// use gpui::Context;
/// use luma::controls::context_menu::MenuItem;
/// use luma_look_radix::{ContextMenu, Look};
/// fn menu<M: 'static>(look: &Look, cx: &mut Context<M>) -> gpui::Entity<luma::controls::context_menu::ContextMenu> {
///     ContextMenu::new("actions").look(look).soft().label("Right-click here")
///         .items([MenuItem::new("copy").label("Copy")]).spawn(cx)
/// }
/// ```
pub struct ContextMenu {
    id: SharedString,
    look: Option<Look>,
    variant: ContextMenuVariant,
    tone: Tone,
    label: Option<SharedString>,
    items: Vec<MenuItem>,
    enabled: bool,
    target: Option<luma::controls::context_menu::ContextMenuTargetContent>,
    modifiers: Vec<ContextMenuModifier>,
}

impl ContextMenu {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            look: None,
            variant: ContextMenuVariant::default(),
            tone: Tone::Accent,
            label: None,
            items: Vec::new(),
            enabled: true,
            target: None,
            modifiers: Vec::new(),
        }
    }

    /// Bind a look. Draft / fork paths must call this; ambient Global is not enough.
    pub fn look(mut self, look: &Look) -> Self {
        self.look = Some(look.clone());
        self
    }

    pub fn variant(mut self, variant: ContextMenuVariant) -> Self {
        self.variant = variant;
        self
    }
    pub fn solid(self) -> Self {
        self.variant(ContextMenuVariant::Solid)
    }
    pub fn soft(self) -> Self {
        self.variant(ContextMenuVariant::Soft)
    }
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    pub fn accent(self) -> Self {
        self.tone(Tone::Accent)
    }
    pub fn gray(self) -> Self {
        self.tone(Tone::Gray)
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn target_content<F, E>(mut self, content: F) -> Self
    where
        F: Fn(&mut gpui::App) -> E + Send + Sync + 'static,
        E: IntoElement + 'static,
    {
        self.target = Some(std::sync::Arc::new(move |cx| content(cx).into_any_element()));
        self
    }

    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn with_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ContextMenuRenderModel<'_>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<luma::controls::context_menu::ContextMenu> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> Look {
        resolve_look(self.look.as_ref(), cx.try_global::<Look>())
    }

    fn into_sdk_builder(self, look: Look) -> ContextMenuBuilder {
        let mut builder = luma::controls::context_menu::ContextMenu::new(self.id)
            .template(context_menu_template(&look, self.variant, self.tone))
            .items(self.items)
            .enabled(self.enabled);
        if let Some(label) = self.label {
            builder = builder.label(label);
        }
        if let Some(target) = self.target {
            builder = builder.target_content(move |cx| target(cx));
        }
        for modifier in self.modifiers {
            builder = builder.with_template_modifier(move |root, model| (modifier)(root, model));
        }
        builder
    }
}
