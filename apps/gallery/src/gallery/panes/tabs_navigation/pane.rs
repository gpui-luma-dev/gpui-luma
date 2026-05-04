use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, IntoElement, MouseDownEvent, MouseUpEvent, Render, SharedString,
    Subscription, Window, div, prelude::*, px,
};
use gpui_luma::controls::tabs_navigation::{
    ControlFocusState, TabsNavigation, TabsNavigationClickHandler, TabsNavigationEvent, TabsNavigationHoverHandler,
    TabsNavigationItem, TabsNavigationItemState, TabsNavigationMouseDownHandler, TabsNavigationMouseUpHandler,
    TabsNavigationRenderItem, TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers,
    ThemedTabsNavigationTemplate,
};
use gpui_luma::controls::tabs_navigation::{
    DefaultTabsNavigationTheme, TabsNavigationItemAppearance, TabsNavigationListAppearance, TabsNavigationTheme,
};
use gpui_luma::theme::InteractionState;

use crate::gallery::control::GalleryApp;
use crate::gallery::theme::GalleryThemePack;

use super::super::shared::{gallery_pane_with_usage, notify_entity};

#[derive(Clone)]
pub(in crate::gallery) struct TabsNavigationPane {
    tabs: Entity<TabsNavigation>,
    local_theme_tabs: Entity<TabsNavigation>,
    state_preview: Entity<TabsNavigationStatePreview>,
    active_label: String,
    local_theme_active_label: String,
}

impl TabsNavigationPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            tabs: TabsNavigation::new("project-tabs")
                .items(project_tabs())
                .active("activity")
                .template(theme.tabs_navigation_template())
                .spawn(cx),
            local_theme_tabs: TabsNavigation::new("project-tabs-local-theme")
                .items(project_tabs())
                .active("activity")
                .template(local_tabs_navigation_template(theme))
                .spawn(cx),
            state_preview: cx.new(|_| TabsNavigationStatePreview::new(theme)),
            active_label: "Activity".to_string(),
            local_theme_active_label: "Activity".to_string(),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(&self.tabs, |app, _, event: &TabsNavigationEvent, cx| {
            app.panes.tabs_navigation.handle_event(event, cx);
        }));
        subscriptions.push(cx.subscribe(&self.local_theme_tabs, |app, _, event: &TabsNavigationEvent, cx| {
            app.panes.tabs_navigation.handle_local_theme_event(event, cx);
        }));
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        gallery_pane_with_usage(
            "Tabs Navigation",
            "Tabs Navigation",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .child(self.tabs.clone())
                        .child(render_tab_content(&self.active_label, theme)),
                )
                .child(self.state_preview.clone())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_3()
                        .mt(px(10.0))
                        .child(render_example_label("Local theme customization", theme))
                        .child(self.local_theme_tabs.clone())
                        .child(render_tab_content(&self.local_theme_active_label, theme)),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.tabs, cx);
        notify_entity(&self.local_theme_tabs, cx);
        notify_entity(&self.state_preview, cx);
    }

    fn handle_event(&mut self, event: &TabsNavigationEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TabsNavigationEvent::Activate { label, .. } => {
                self.active_label = label.to_string();
                cx.notify();
            }
        }
    }

    fn handle_local_theme_event(&mut self, event: &TabsNavigationEvent, cx: &mut Context<GalleryApp>) {
        match event {
            TabsNavigationEvent::Activate { label, .. } => {
                self.local_theme_active_label = label.to_string();
                cx.notify();
            }
        }
    }
}

struct LocalTabsNavigationTheme {
    theme: GalleryThemePack,
}

impl TabsNavigationTheme for LocalTabsNavigationTheme {
    fn resolve_list(&self, enabled: bool) -> TabsNavigationListAppearance {
        DefaultTabsNavigationTheme::new(self.theme.tokens()).resolve_list(enabled)
    }

    fn resolve_item(&self, active: bool, state: InteractionState) -> TabsNavigationItemAppearance {
        let unfocused_state = InteractionState { focused: false, ..state };

        DefaultTabsNavigationTheme::new(self.theme.tokens()).resolve_item(active, unfocused_state)
    }
}

fn local_tabs_navigation_template(theme: &GalleryThemePack) -> Arc<dyn TabsNavigationTemplate> {
    Arc::new(ThemedTabsNavigationTemplate::new(Arc::new(LocalTabsNavigationTheme { theme: theme.clone() })))
}

#[derive(Clone)]
struct TabsNavigationStatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn TabsNavigationTemplate>,
}

struct TabsNavigationStateSample {
    id: &'static str,
    label: &'static str,
    active_index: usize,
    target_index: usize,
    target_state: TabsNavigationItemState,
    enabled: bool,
}

impl TabsNavigationStatePreview {
    fn new(theme: &GalleryThemePack) -> Self {
        Self { theme: theme.clone(), template: theme.tabs_navigation_template() }
    }
}

impl Render for TabsNavigationStatePreview {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let samples = [
            TabsNavigationStateSample {
                id: "inactive",
                label: "Inactive",
                active_index: 1,
                target_index: 0,
                target_state: TabsNavigationItemState::default(),
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "active",
                label: "Active",
                active_index: 1,
                target_index: 1,
                target_state: TabsNavigationItemState { selected: true, ..TabsNavigationItemState::default() },
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "hover",
                label: "Hover",
                active_index: 1,
                target_index: 0,
                target_state: TabsNavigationItemState { hovered: true, ..TabsNavigationItemState::default() },
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "focus",
                label: "Focus",
                active_index: 1,
                target_index: 1,
                target_state: TabsNavigationItemState {
                    selected: true,
                    active: true,
                    focus_visible: true,
                    ..TabsNavigationItemState::default()
                },
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "pressed",
                label: "Pressed",
                active_index: 1,
                target_index: 1,
                target_state: TabsNavigationItemState {
                    selected: true,
                    active: true,
                    hovered: true,
                    pressed: true,
                    focus_visible: true,
                    ..TabsNavigationItemState::default()
                },
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "disabled-item",
                label: "Disabled item",
                active_index: 1,
                target_index: 2,
                target_state: TabsNavigationItemState { disabled: true, ..TabsNavigationItemState::default() },
                enabled: true,
            },
            TabsNavigationStateSample {
                id: "disabled-list",
                label: "Disabled list",
                active_index: 1,
                target_index: 1,
                target_state: TabsNavigationItemState {
                    selected: true,
                    disabled: true,
                    ..TabsNavigationItemState::default()
                },
                enabled: false,
            },
        ];

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .text_size(px(12.0))
                    .line_height(px(16.0))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(chrome.muted_text)
                    .child("Template state preview"),
            )
            .child(
                div().flex().flex_wrap().items_start().justify_center().gap(px(12.0)).children(
                    samples
                        .into_iter()
                        .map(|sample| render_tabs_state_sample(&self.template, sample, chrome.muted_text, window, cx)),
                ),
            )
    }
}

fn render_tabs_state_sample(
    template: &Arc<dyn TabsNavigationTemplate>,
    sample: TabsNavigationStateSample,
    label_color: gpui::Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = SharedString::from(format!("tabs-navigation-preview-{}", sample.id));
    let items = preview_tabs();
    let active_id = items.get(sample.active_index).map(TabsNavigationItem::id);
    let render_items = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let item_enabled = sample.enabled && item.is_enabled();
            let active = active_id.is_some_and(|active_id| active_id == item.id());
            let mut state = TabsNavigationItemState { selected: active, ..TabsNavigationItemState::default() };

            if index == sample.target_index {
                state = sample.target_state;
                state.selected = active;
            }

            if !item_enabled {
                state.disabled = true;
                state.hovered = false;
                state.pressed = false;
                state.active = false;
                state.focus_visible = false;
            }

            TabsNavigationRenderItem { id: item.id(), label: item.label_text(), active, enabled: item_enabled, state }
        })
        .collect::<Vec<_>>();
    let model = TabsNavigationRenderModel {
        id: &id,
        items: render_items,
        active_id,
        enabled: sample.enabled,
        focus: ControlFocusState {
            focused: sample.target_state.active,
            focus_visible: sample.target_state.focus_visible,
        },
    };

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(6.0))
        .child(template.render(&model, tabs_navigation_preview_handlers(items.len()), window, cx))
        .child(div().text_size(px(11.0)).line_height(px(15.0)).text_color(label_color).child(sample.label))
        .into_any_element()
}

fn project_tabs() -> [TabsNavigationItem; 4] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("metrics").label("Metrics"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}

fn preview_tabs() -> [TabsNavigationItem; 3] {
    [
        TabsNavigationItem::new("overview").label("Overview"),
        TabsNavigationItem::new("activity").label("Activity"),
        TabsNavigationItem::new("settings").label("Settings").enabled(false),
    ]
}

fn tabs_navigation_preview_handlers(count: usize) -> TabsNavigationTemplateHandlers {
    TabsNavigationTemplateHandlers {
        item_hovers: (0..count).map(|_| Box::new(noop_hover) as TabsNavigationHoverHandler).collect(),
        item_mouse_downs: (0..count).map(|_| Box::new(noop_mouse_down) as TabsNavigationMouseDownHandler).collect(),
        item_mouse_ups: (0..count).map(|_| Box::new(noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_mouse_up_outs: (0..count).map(|_| Box::new(noop_mouse_up) as TabsNavigationMouseUpHandler).collect(),
        item_clicks: (0..count).map(|_| Box::new(noop_click) as TabsNavigationClickHandler).collect(),
    }
}

fn noop_hover(_: &bool, _: &mut Window, _: &mut App) {}

fn noop_mouse_down(_: &MouseDownEvent, _: &mut Window, _: &mut App) {}

fn noop_mouse_up(_: &MouseUpEvent, _: &mut Window, _: &mut App) {}

fn noop_click(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn render_example_label(label: &'static str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .text_size(px(12.0))
        .line_height(px(16.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(chrome.muted_text)
        .child(label)
        .into_any_element()
}

fn render_tab_content(active_label: &str, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    div()
        .w(px(360.0))
        .min_h(px(112.0))
        .flex()
        .flex_col()
        .gap_2()
        .rounded(px(8.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .p(px(16.0))
        .text_color(chrome.body_text)
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(format!("{active_label} tab")),
        )
        .child(
            div()
                .text_size(px(13.0))
                .line_height(px(18.0))
                .child("The pane stays responsible for focus after a tab activation."),
        )
        .into_any_element()
}
