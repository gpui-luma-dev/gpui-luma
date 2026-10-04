//! Look-owned tabs builder. Spawn synthesizes the SDK [`gpui_luma::controls::tabs::Tabs`].

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;

use gpui::{App, Context, Div, Entity, IntoElement, Render, SharedString, Stateful, Window};
use gpui_luma::controls::tabs::{TabsBuilder, TabsContent, TabsItem, TabsRenderModel, TabsTemplate, TabsWidthMode};
use gpui_luma::infra::icon::DisclosureIcons;
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
    contents: HashMap<SharedString, TabsContent>,
    fade_duration: Option<Duration>,
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
            contents: HashMap::new(),
            fade_duration: None,
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
        self.contents.retain(|id, _| self.items.iter().any(|item| item.id() == id));
        self
    }

    /// Add a tab backed by an existing view, retained across selection changes.
    pub fn tab<V: Render + 'static>(
        self,
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        view: Entity<V>,
    ) -> Self {
        self.tab_with(id, label, move |_, _| view.clone())
    }

    /// Add a presenter invoked only for the selected panel.
    pub fn tab_with<F, E>(self, id: impl Into<SharedString>, label: impl Into<SharedString>, presenter: F) -> Self
    where
        F: Fn(&mut Window, &mut App) -> E + 'static,
        E: IntoElement + 'static,
    {
        self.tab_content(TabsItem::new(id).label(label), TabsContent::new(presenter))
    }

    /// Add an item and its panel, preserving item options.
    pub fn tab_content(mut self, item: TabsItem, content: TabsContent) -> Self {
        self.contents.insert(item.id().clone(), content);
        self.items.push(item);
        self
    }

    /// Override inherited body motion. Zero disables the fade.
    pub fn fade_in(mut self, duration: Duration) -> Self {
        self.fade_duration = Some(duration);
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

    pub fn spawn<M: 'static>(self, cx: &mut Context<M>) -> Entity<gpui_luma::controls::tabs::Tabs> {
        let look = self.resolve_look(cx);
        self.into_sdk_builder(look).spawn(cx)
    }

    fn resolve_look(&self, cx: &App) -> ShadcnLook {
        resolve_look_from(self.look.as_ref(), cx)
    }

    fn into_sdk_builder(self, look: ShadcnLook) -> TabsBuilder {
        let template = self.template.unwrap_or_else(|| look.tabs_template());
        let mut builder = gpui_luma::controls::tabs::Tabs::new(self.id)
            .template(template)
            .size(self.size.control_size())
            .width_mode(self.width_mode)
            .enabled(self.enabled)
            .animated(self.animated)
            .stylesheet(&look.common_stylesheet(), "shadcn");
        if let Some(duration) = self.fade_duration {
            builder = builder.fade_in(duration);
        }
        for item in self.items {
            if let Some(content) = self.contents.get(item.id()) {
                builder = builder.tab_content(item, content.clone());
            } else {
                builder = builder.item(item);
            }
        }
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
    fn custom_stylesheet_motion_and_content_reach_sdk() {
        use gpui_luma::theme::stylesheet::MotionSource;
        for (source, expected) in [("", None), ("[common.tabs.content]\nfade_in_ms = 175", Some(175))] {
            let stylesheet = crate::stylesheet::StylesheetConfig::parse(source).unwrap();
            let look = ShadcnLook::from_css_str_with_stylesheet(crate::FALLBACK_CSS, stylesheet).unwrap();
            let copy = look.with_color_overrides(&Default::default());
            assert_eq!(copy.common_stylesheet(), look.common_stylesheet());
            let tabs = Tabs::new("custom").tab_with("one", "One", |_, _| gpui::div());
            assert_eq!(tabs.contents.len(), 1);
            let builder = tabs.into_sdk_builder(copy);
            assert_eq!(builder.body_motion().duration, Duration::from_millis(expected.unwrap_or(0)));
            assert_eq!(matches!(builder.body_motion().source, MotionSource::SdkFallback), expected.is_none());
        }
    }

    #[test]
    fn body_motion_inherits_look_and_respects_overrides() {
        use gpui_luma::theme::stylesheet::MotionSource;
        let look = ShadcnLook::built_in();
        let inherited = Tabs::new("motion").into_sdk_builder(look.clone()).body_motion();
        assert_eq!(inherited.duration, Duration::from_millis(300));
        assert_eq!(
            inherited.source,
            MotionSource::Look { name: "shadcn".into(), field: "common.tabs.content.fade_in_ms" }
        );
        let explicit = Tabs::new("motion").fade_in(Duration::ZERO).into_sdk_builder(look.clone()).body_motion();
        assert_eq!(explicit.duration, Duration::ZERO);
        assert_eq!(explicit.source, MotionSource::InstanceOverride);
        let disabled = Tabs::new("motion")
            .fade_in(Duration::from_millis(900))
            .animated(false)
            .into_sdk_builder(look)
            .body_motion();
        assert_eq!(disabled.duration, Duration::ZERO);
        assert_eq!(disabled.source, MotionSource::AnimationDisabled);
    }

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
