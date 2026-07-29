#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window, div, px};
use gpui::prelude::*;
use gpui_luma::controls::accordion::{
    AccordionContent, AccordionControl, AccordionItem, AccordionSelectionMode, AccordionTrigger,
};
use gpui_luma::controls::tabs_navigation::{TabsNavigation, TabsNavigationEvent, TabsNavigationItem};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnLookControlExt};

use super::inspector::{
    ControlInspectorSpec, InspectorCategory, InspectorSelection, InspectorStateSpec, SharedInspectorResolver, layout,
    render_category_content,
};

pub struct ThemeInspector {
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
    variant_tabs: Option<Entity<TabsNavigation>>,
    variants: Vec<VariantInspectorControls>,
    active_variant_id: SharedString,
    synced_mode: ThemeMode,
    embedded: bool,
    _subscriptions: Vec<Subscription>,
}

struct VariantInspectorControls {
    variant_id: SharedString,
    state: Entity<AccordionControl>,
}

struct ThemePropertyTree {
    categories: Entity<AccordionControl>,
}

struct LayoutSizeInspector {
    look: Arc<ShadcnLook>,
    resolver: SharedInspectorResolver,
    variant_id: SharedString,
    state_id: SharedString,
    size_tabs: Entity<TabsNavigation>,
    active_size_id: SharedString,
    _subscriptions: Vec<Subscription>,
}

struct ColorValueInspector {
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
    variant_id: SharedString,
    state_id: SharedString,
    value_tabs: Entity<TabsNavigation>,
    active_value_id: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl ThemeInspector {
    pub fn new(
        look: Arc<ShadcnLook>,
        spec: &'static ControlInspectorSpec,
        resolver: SharedInspectorResolver,
        embedded: bool,
        cx: &mut Context<Self>,
    ) -> Self {
        let (variant_tabs, subscriptions, active_variant_id) = if spec.variants.is_empty() {
            (None, Vec::new(), SharedString::from(""))
        } else {
            let tabs = look
                .tabs_navigation(format!("{}-variant-tabs", spec.id_prefix))
                .items(variant_tab_items(spec))
                .active(spec.default_variant_id)
                .spawn(cx);

            let mut subs = Vec::new();
            subs.push(cx.subscribe(&tabs, |inspector, _, event: &TabsNavigationEvent, cx| {
                let TabsNavigationEvent::Activate { tab_id, .. } = event else {
                    return;
                };
                inspector.active_variant_id = tab_id.clone();
                cx.notify();
            }));

            (Some(tabs), subs, SharedString::from(spec.default_variant_id))
        };

        let variants = build_variant_controls(look.clone(), spec, resolver.clone(), cx);

        Self {
            synced_mode: look.mode(),
            look,
            spec,
            resolver,
            variant_tabs,
            variants,
            active_variant_id,
            embedded,
            _subscriptions: subscriptions,
        }
    }

    pub fn for_embedded_pane_with_spec(
        look: Arc<ShadcnLook>,
        spec: &'static ControlInspectorSpec,
        resolver: SharedInspectorResolver,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::new(look, spec, resolver, true, cx)
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.variants = build_variant_controls(self.look.clone(), self.spec, self.resolver.clone(), cx);
        self.synced_mode = self.look.mode();
        if let Some(tabs) = &self.variant_tabs {
            tabs.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    fn active_tree(&self) -> Entity<AccordionControl> {
        if self.spec.variants.is_empty() {
            return self.variants[0].state.clone();
        }

        self.variants
            .iter()
            .find(|controls| controls.variant_id == self.active_variant_id)
            .map(|controls| controls.state.clone())
            .unwrap_or_else(|| self.variants[0].state.clone())
    }

    fn sync_if_needed(&mut self, cx: &mut Context<Self>) {
        let mode = self.look.mode();
        if self.synced_mode == mode {
            return;
        }
        self.synced_mode = mode;
        cx.notify();
    }
}

impl Render for ThemeInspector {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_if_needed(cx);

        let chrome = self.look.chrome();
        let body = &self.look.mode_tokens().typography.text.body;
        let caption = &self.look.mode_tokens().typography.text.caption;

        let mut root = div()
            .id(SharedString::from(self.spec.id_prefix))
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(chrome.panel_background);

        if !self.embedded {
            root = root.border_1().border_color(chrome.border).rounded(px(8.0));
        }

        let mut header = div()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .border_b_1()
            .border_color(chrome.border)
            .p(px(layout::PANEL_PADDING))
            .child(
                div().flex().items_center().gap(px(12.0)).child(
                    div()
                        .flex()
                        .items_baseline()
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_size(px(body.size + 3.0))
                                .line_height(px(body.line_height + 3.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(chrome.title_text)
                                .child("Theme Inspector"),
                        )
                        .when(!self.embedded, |row| {
                            row.child(
                                div()
                                    .text_size(px(caption.size))
                                    .line_height(px(caption.line_height))
                                    .text_color(chrome.muted_text)
                                    .child(self.spec.control_label),
                            )
                        }),
                ),
            );

        if let Some(variant_tabs) = &self.variant_tabs {
            header = header.child(variant_tabs.clone());
        }

        root.child(header).child(
            div()
                .id(SharedString::from(format!("{}-scroll", self.spec.id_prefix)))
                .flex_1()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .p(px(layout::PANEL_PADDING))
                .child(self.active_tree()),
        )
    }
}

fn variant_tab_items(spec: &ControlInspectorSpec) -> Vec<TabsNavigationItem> {
    spec.variants
        .iter()
        .map(|variant| TabsNavigationItem::new(variant.id).label(variant.label))
        .collect()
}

fn build_variant_controls(
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
    cx: &mut Context<ThemeInspector>,
) -> Vec<VariantInspectorControls> {
    if spec.variants.is_empty() {
        let state_trees = spec
            .states
            .iter()
            .map(|state_spec| {
                let categories = applicable_categories(&look, &resolver, spec, "", state_spec);
                let tree = cx.new(|cx| {
                    ThemePropertyTree::new(
                        look.clone(),
                        spec,
                        resolver.clone(),
                        SharedString::from(""),
                        SharedString::from(state_spec.id),
                        categories,
                        cx,
                    )
                });
                (*state_spec, tree)
            })
            .collect();
        let state = state_accordion(look, spec, SharedString::from(""), state_trees, cx);
        return vec![VariantInspectorControls { variant_id: SharedString::from(""), state }];
    }

    spec.variants
        .iter()
        .map(|variant| {
            let state_trees = spec
                .states
                .iter()
                .map(|state_spec| {
                    let categories = applicable_categories(&look, &resolver, spec, variant.id, state_spec);
                    let tree = cx.new(|cx| {
                        ThemePropertyTree::new(
                            look.clone(),
                            spec,
                            resolver.clone(),
                            SharedString::from(variant.id),
                            SharedString::from(state_spec.id),
                            categories,
                            cx,
                        )
                    });
                    (*state_spec, tree)
                })
                .collect();
            let state = state_accordion(look.clone(), spec, SharedString::from(variant.id), state_trees, cx);
            VariantInspectorControls { variant_id: SharedString::from(variant.id), state }
        })
        .collect()
}

fn applicable_categories(
    look: &ShadcnLook,
    resolver: &SharedInspectorResolver,
    spec: &ControlInspectorSpec,
    variant_id: &str,
    state_spec: &InspectorStateSpec,
) -> Vec<InspectorCategory> {
    let selection = inspector_selection(variant_id, state_spec.id, spec.default_size_id, spec.default_value_id);
    state_spec
        .categories
        .iter()
        .copied()
        .filter(|category| resolver.category_applies(look, selection, category.id))
        .collect()
}

fn state_accordion(
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    variant_id: SharedString,
    state_trees: Vec<(InspectorStateSpec, Entity<ThemePropertyTree>)>,
    cx: &mut Context<ThemeInspector>,
) -> Entity<AccordionControl> {
    look.accordion(format!("{}-{}-states", spec.id_prefix, variant_id))
        .mode(AccordionSelectionMode::Multiple)
        .item_dividers(false)
        .trigger_min_height(30.0)
        .trigger_padding_y(3.0)
        .content_padding_top(2.0)
        .content_padding_bottom(6.0)
        .items(state_trees.into_iter().map(|(state_spec, tree)| {
            AccordionItem::new(
                format!("{}-{}", variant_id, state_spec.id),
                AccordionTrigger::new(state_spec.label).icon(state_spec.icon),
                AccordionContent::custom({
                    let tree = tree.clone();
                    move |_, _| tree.clone().into_any_element()
                }),
            )
            .expanded(state_spec.expanded_default)
        }))
        .spawn(cx)
}

impl ThemePropertyTree {
    fn new(
        look: Arc<ShadcnLook>,
        spec: &'static ControlInspectorSpec,
        resolver: SharedInspectorResolver,
        variant_id: SharedString,
        state_id: SharedString,
        categories: Vec<InspectorCategory>,
        cx: &mut Context<Self>,
    ) -> Self {
        let layout = if categories.iter().any(|category| category.id == "layout") && !spec.sizes.is_empty() {
            Some(cx.new(|cx| {
                LayoutSizeInspector::new(look.clone(), resolver.clone(), variant_id.clone(), state_id.clone(), spec, cx)
            }))
        } else {
            None
        };

        let color = if categories.iter().any(|category| category.id == "color") && !spec.value_modes.is_empty() {
            Some(cx.new(|cx| {
                ColorValueInspector::new(look.clone(), spec, resolver.clone(), variant_id.clone(), state_id.clone(), cx)
            }))
        } else {
            None
        };

        let categories = category_accordion(
            look.clone(),
            spec,
            resolver.clone(),
            variant_id.clone(),
            state_id.clone(),
            layout,
            color,
            &categories,
            cx,
        );
        Self { categories }
    }
}

impl Render for ThemePropertyTree {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.categories.clone()
    }
}

fn category_accordion(
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
    variant_id: SharedString,
    state_id: SharedString,
    layout: Option<Entity<LayoutSizeInspector>>,
    color: Option<Entity<ColorValueInspector>>,
    categories: &[InspectorCategory],
    cx: &mut Context<ThemePropertyTree>,
) -> Entity<AccordionControl> {
    look.accordion(format!("{}-{}-categories", spec.id_prefix, variant_id))
        .mode(AccordionSelectionMode::Multiple)
        .item_dividers(false)
        .trigger_min_height(28.0)
        .trigger_padding_y(2.0)
        .content_padding_top(2.0)
        .content_padding_bottom(8.0)
        .items(categories.iter().copied().map(|category| {
            category_item(
                look.clone(),
                spec,
                resolver.clone(),
                variant_id.clone(),
                state_id.clone(),
                layout.clone(),
                color.clone(),
                category,
            )
        }))
        .spawn(cx)
}

fn category_item(
    look: Arc<ShadcnLook>,
    spec: &'static ControlInspectorSpec,
    resolver: SharedInspectorResolver,
    variant_id: SharedString,
    state_id: SharedString,
    layout: Option<Entity<LayoutSizeInspector>>,
    color: Option<Entity<ColorValueInspector>>,
    category: InspectorCategory,
) -> AccordionItem {
    AccordionItem::new(
        category.id,
        AccordionTrigger::new(category.label).icon(category.icon),
        AccordionContent::custom(move |_, _| {
            if category.id == "layout"
                && let Some(layout) = &layout
            {
                return layout.clone().into_any_element();
            }

            if category.id == "color"
                && let Some(color) = &color
            {
                return color.clone().into_any_element();
            }

            let selection = inspector_selection(
                variant_id.as_ref(),
                state_id.as_ref(),
                spec.default_size_id,
                spec.default_value_id,
            );
            let content = resolver.resolve_category(&look, selection, category.id);
            render_category_content(&look, content)
        }),
    )
    .expanded(category.expanded_default)
}

fn inspector_selection<'a>(
    variant_id: &'a str,
    state_id: &'a str,
    size_id: &'a str,
    value_id: &'a str,
) -> InspectorSelection<'a> {
    InspectorSelection { variant_id, state_id, size_id, value_id }
}

impl LayoutSizeInspector {
    fn new(
        look: Arc<ShadcnLook>,
        resolver: SharedInspectorResolver,
        variant_id: SharedString,
        state_id: SharedString,
        spec: &'static ControlInspectorSpec,
        cx: &mut Context<Self>,
    ) -> Self {
        let size_tabs = look
            .tabs_navigation(format!("{}-{}-size-tabs", spec.id_prefix, variant_id))
            .items(size_tab_items(spec))
            .active(spec.default_size_id)
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&size_tabs, |inspector, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event else {
                return;
            };
            inspector.active_size_id = tab_id.clone();
            cx.notify();
        }));

        Self {
            look,
            resolver,
            variant_id,
            state_id,
            size_tabs,
            active_size_id: SharedString::from(spec.default_size_id),
            _subscriptions: subscriptions,
        }
    }
}

impl Render for LayoutSizeInspector {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let selection =
            inspector_selection(self.variant_id.as_ref(), self.state_id.as_ref(), self.active_size_id.as_ref(), "");
        let content = self.resolver.resolve_category(&self.look, selection, "layout");

        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(layout::DETAIL_GAP))
            .child(self.size_tabs.clone())
            .child(render_category_content(&self.look, content))
    }
}

impl ColorValueInspector {
    fn new(
        look: Arc<ShadcnLook>,
        spec: &'static ControlInspectorSpec,
        resolver: SharedInspectorResolver,
        variant_id: SharedString,
        state_id: SharedString,
        cx: &mut Context<Self>,
    ) -> Self {
        let value_tabs = look
            .tabs_navigation(format!("{}-{}-value-tabs", spec.id_prefix, variant_id))
            .items(value_tab_items(spec))
            .active(spec.default_value_id)
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&value_tabs, |inspector, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event else {
                return;
            };
            inspector.active_value_id = tab_id.clone();
            cx.notify();
        }));

        Self {
            look,
            spec,
            resolver,
            variant_id,
            state_id,
            value_tabs,
            active_value_id: SharedString::from(spec.default_value_id),
            _subscriptions: subscriptions,
        }
    }
}

impl Render for ColorValueInspector {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let selection = inspector_selection(
            self.variant_id.as_ref(),
            self.state_id.as_ref(),
            self.spec.default_size_id,
            self.active_value_id.as_ref(),
        );
        let content = self.resolver.resolve_category(&self.look, selection, "color");

        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(layout::DETAIL_GAP))
            .child(self.value_tabs.clone())
            .child(render_category_content(&self.look, content))
    }
}

fn size_tab_items(spec: &ControlInspectorSpec) -> Vec<TabsNavigationItem> {
    spec.sizes.iter().map(|size| TabsNavigationItem::new(size.id).label(size.label)).collect()
}

fn value_tab_items(spec: &ControlInspectorSpec) -> Vec<TabsNavigationItem> {
    spec.value_modes.iter().map(|value| TabsNavigationItem::new(value.id).label(value.label)).collect()
}
