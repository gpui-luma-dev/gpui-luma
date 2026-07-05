mod model;
mod panels;
mod parsing;
mod subscriptions;
mod sync;

use gpui::{
    AnyElement, App, Context, Entity, Hsla, Render, SharedString, Subscription, Window, div, prelude::*, px,
    transparent_black,
};
use gpui_luma::controls::selector::{
    Selector, SelectorItemLike, SelectorItemRenderModel, SelectorTemplate, ThemedSelectorTemplate,
};
use gpui_luma::controls::selector_panel::default_selector_items_template;
use gpui_luma::controls::tabs_navigation::{
    TabsNavigation, TabsNavigationEvent, TabsNavigationItem, TabsNavigationWidthMode,
};

use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::{BuiltInTheme, ShadcnLook, ShadcnLookControlExt};

use self::model::{SidebarTab, TOKEN_CATEGORIES};
use self::panels::{ColorsPanel, OtherPanel, render_typography_panel};
use self::parsing::token_color_with_fallback;
use super::content_tabs::theme_studio_tabs_navigation_template;
use crate::studio::app::ThemeStudioApp;
use crate::studio::overrides::StudioOverrides;
use crate::theme::available_themes;

pub struct ThemeSidebar {
    look: std::sync::Arc<ShadcnLook>,
    global_overrides: std::collections::HashMap<String, gpui::Hsla>,
    theme_selector: Entity<Selector<ThemeSelectorItem>>,
    tabs: Entity<TabsNavigation>,
    active_tab: SidebarTab,
    colors_panel: Entity<ColorsPanel>,
    other_panel: Entity<OtherPanel>,
    _subscriptions: Vec<Subscription>,
}

impl ThemeSidebar {
    pub fn new(
        _app: Entity<ThemeStudioApp>,
        look: std::sync::Arc<ShadcnLook>,
        active_theme_id: impl Into<gpui::SharedString>,
        overrides: &StudioOverrides,
        cx: &mut Context<Self>,
    ) -> Self {
        let active_theme_id = active_theme_id.into();
        let global_overrides = overrides.global_color_overrides.clone();
        let colors_panel = cx.new(|cx| ColorsPanel::new(look.clone(), overrides, cx));
        let other_panel = cx.new(|cx| OtherPanel::new(look.clone(), overrides, cx));
        let theme_items = theme_selector_items(look.as_ref());
        let theme_selector = Selector::new_typed("theme-studio-theme-selector")
            .label("Theme")
            .items(theme_items)
            .selected_id(active_theme_id.clone())
            .template(theme_selector_template(&look))
            .with_item_template(render_theme_selector_item)
            .spawn(cx);

        let tabs = look
            .tabs_navigation("theme-studio-sidebar-tabs")
            .size(ControlSize::Lg)
            .width_mode(TabsNavigationWidthMode::Uniform)
            .template(theme_studio_tabs_navigation_template(look.clone(), ControlSize::Lg))
            .items([
                TabsNavigationItem::new("colors").label("Colors"),
                TabsNavigationItem::new("typography").label("Typography"),
                TabsNavigationItem::new("other").label("Other"),
            ])
            .active("colors")
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |sidebar, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            if let Some(tab) = SidebarTab::from_id(tab_id.as_ref()) {
                sidebar.active_tab = tab;
                cx.notify();
            }
        }));

        Self {
            look,
            global_overrides,
            theme_selector,
            tabs,
            active_tab: SidebarTab::Colors,
            colors_panel,
            other_panel,
            _subscriptions: subscriptions,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ThemeSwatches {
    primary_background: Hsla,
    accent_background: Hsla,
    secondary_background: Hsla,
    border: Hsla,
}

#[derive(Clone, Debug)]
struct ThemeSelectorItem {
    id: SharedString,
    label: SharedString,
    swatches: ThemeSwatches,
}

impl ThemeSelectorItem {
    fn new(id: impl Into<SharedString>, label: impl Into<SharedString>, swatches: ThemeSwatches) -> Self {
        Self { id: id.into(), label: label.into(), swatches }
    }
}

impl SelectorItemLike for ThemeSelectorItem {
    fn id(&self) -> &SharedString {
        &self.id
    }

    fn label(&self) -> &SharedString {
        &self.label
    }
}

fn theme_selector_items(current_look: &ShadcnLook) -> Vec<ThemeSelectorItem> {
    let mut items =
        vec![ThemeSelectorItem::new("default", "Default", swatches_for_look(&native_look_for_mode(current_look)))];
    for theme in available_themes() {
        if let Some(item) = built_in_theme_selector_item(theme, current_look) {
            items.push(item);
        }
    }
    items
}

fn built_in_theme_selector_item(theme: &BuiltInTheme, current_look: &ShadcnLook) -> Option<ThemeSelectorItem> {
    let look = ShadcnLook::from_built_in_theme(theme.id).ok()?;
    look.set_mode(current_look.mode());
    Some(ThemeSelectorItem::new(theme.id, theme.display_name(), swatches_for_look(&look)))
}

fn native_look_for_mode(current_look: &ShadcnLook) -> ShadcnLook {
    let look = ShadcnLook::native();
    look.set_mode(current_look.mode());
    look
}

fn swatches_for_look(look: &ShadcnLook) -> ThemeSwatches {
    let palette = look.mode_tokens().palette;
    ThemeSwatches {
        primary_background: palette.primary.background,
        accent_background: palette.accent_background,
        secondary_background: palette.secondary.background,
        border: palette.border_default,
    }
}

fn theme_selector_template(
    theme: &std::sync::Arc<ShadcnLook>,
) -> std::sync::Arc<dyn SelectorTemplate<ThemeSelectorItem>> {
    std::sync::Arc::new(
        ThemedSelectorTemplate::new(theme.selector_theme(), default_selector_items_template::<ThemeSelectorItem>())
            .with_modifier(|element, _| element.bg(transparent_black())),
    )
}

fn render_theme_selector_item(model: &SelectorItemRenderModel<'_, ThemeSelectorItem>, _cx: &mut App) -> AnyElement {
    let swatch_size = px(20.0);
    let swatch_radius = px(4.0);
    let swatch_gap = px(6.0);
    let label_gap = px(14.0);

    div()
        .flex()
        .items_center()
        .gap(label_gap)
        .child(
            div()
                .flex()
                .items_center()
                .gap(swatch_gap)
                .child(render_theme_swatch(model.item.swatches.primary_background, swatch_size, swatch_radius))
                .child(render_theme_swatch(model.item.swatches.accent_background, swatch_size, swatch_radius))
                .child(render_theme_swatch(model.item.swatches.secondary_background, swatch_size, swatch_radius))
                .child(render_theme_swatch(model.item.swatches.border, swatch_size, swatch_radius)),
        )
        .child(div().flex_1().min_w(px(0.0)).truncate().child(model.item.label.clone()))
        .into_any_element()
}

fn render_theme_swatch(color: Hsla, size: gpui::Pixels, radius: gpui::Pixels) -> impl IntoElement {
    div().size(size).flex_shrink_0().rounded(radius).bg(color)
}

pub(crate) fn palette_tokens() -> Vec<&'static str> {
    TOKEN_CATEGORIES.iter().flat_map(|(_, tokens)| tokens.iter().map(|(token, _)| *token)).collect()
}

impl Render for ThemeSidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let sidebar_bg =
            token_color_with_fallback(&self.look, &self.global_overrides, "sidebar", chrome.panel_background);

        let tab_body = match self.active_tab {
            SidebarTab::Colors => self.colors_panel.clone().into_any_element(),
            SidebarTab::Typography => render_typography_panel(),
            SidebarTab::Other => self.other_panel.clone().into_any_element(),
        };

        div()
            .id("theme-studio-sidebar")
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(sidebar_bg)
            .child(
                div()
                    .id("theme-studio-sidebar-theme-selector-shell")
                    .w_full()
                    .h(px(48.0))
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .px(px(24.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .child(self.theme_selector.clone()),
            )
            .child(div().flex_shrink_0().pt(px(8.0)).child(div().w_full().child(self.tabs.clone())))
            .child(
                div()
                    .id("theme-studio-sidebar-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .w_full()
                    .overflow_y_scroll()
                    .p(px(12.0))
                    .child(tab_body),
            )
    }
}
