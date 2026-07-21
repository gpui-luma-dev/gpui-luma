#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, Context, Entity, FontWeight, Hsla, IntoElement, Render, SharedString, Subscription, Window, div, hsla,
    relative,
};
use gpui::{prelude::*, px};
use gpui_luma::controls::accordion::{
    AccordionContent, AccordionControl, AccordionItem, AccordionSelectionMode, AccordionTrigger,
};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::tabs_navigation::{TabsNavigation, TabsNavigationEvent, TabsNavigationItem};
use gpui_luma::theme::{ControlSize, InteractionState, ThemeMode};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook, ShadcnLookControlExt};
use gpui_luma_look_shadcn_inspect::{
    ButtonInspectMetrics, ResolvedColor, ResolvedMetric, ShadcnInspect, format_inspect_css_key,
    format_inspect_box_shadow_layer, format_inspect_metric_provenance, format_inspect_metric_source,
    format_inspect_provenance, format_metric_px,
};
use lucide_icons::Icon as LucideIcon;

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::format_hex_color;
use crate::gallery::panes::shared::inspector::box_model::{
    BoxModelLayerColors, InspectBoxModelSnapshot, MetricFieldHighlight, render_box_model_diagram,
};
use crate::gallery::panes::shared::inspector::occupation::{button_family_occupation, occupation_metric_properties};

mod layout {
    pub(super) const PANEL_PADDING: f32 = 12.0;
    pub(super) const DETAIL_GAP: f32 = 8.0;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ThemeVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ThemePropertyCategory {
    Color,
    Layout,
    Elevation,
    Typography,
}

impl ThemeVariant {
    fn id(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Secondary => "Secondary",
            Self::Outline => "Outline",
            Self::Ghost => "Ghost",
        }
    }

    fn style(self) -> ShadcnButtonStyle {
        match self {
            Self::Primary => ShadcnButtonStyle::Primary,
            Self::Secondary => ShadcnButtonStyle::Secondary,
            Self::Outline => ShadcnButtonStyle::Outline,
            Self::Ghost => ShadcnButtonStyle::Ghost,
        }
    }

    fn from_id(id: &str) -> Self {
        match id {
            "secondary" => Self::Secondary,
            "outline" => Self::Outline,
            "ghost" => Self::Ghost,
            _ => Self::Primary,
        }
    }
}

impl ThemePropertyCategory {
    fn id(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Layout => "layout",
            Self::Elevation => "elevation",
            Self::Typography => "typography",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Color => "Color",
            Self::Layout => "Layout",
            Self::Elevation => "Elevation",
            Self::Typography => "Typography",
        }
    }

    fn icon(self) -> LucideIcon {
        match self {
            Self::Color => LucideIcon::Palette,
            Self::Layout => LucideIcon::Ruler,
            Self::Elevation => LucideIcon::Layers,
            Self::Typography => LucideIcon::Type,
        }
    }
}

pub(in crate::gallery) struct ThemeInspector {
    look: Arc<ShadcnLook>,
    tabs: Entity<TabsNavigation>,
    variants: Vec<VariantInspectorControls>,
    active_variant: ThemeVariant,
    synced_mode: gpui_luma::theme::ThemeMode,
    _subscriptions: Vec<Subscription>,
}

struct VariantInspectorControls {
    variant: ThemeVariant,
    state: Entity<AccordionControl>,
}

struct ThemePropertyTree {
    categories: Entity<AccordionControl>,
}

struct LayoutSizeInspector {
    look: Arc<ShadcnLook>,
    tabs: Entity<TabsNavigation>,
    variant: ThemeVariant,
    state: InteractionState,
    active_size: ControlSize,
    _subscriptions: Vec<Subscription>,
}

#[derive(Clone, Copy)]
struct ThemeStateSpec {
    id: &'static str,
    label: &'static str,
    icon: LucideIcon,
    state: InteractionState,
    expanded: bool,
}

impl ThemeInspector {
    pub fn new(look: Arc<ShadcnLook>, cx: &mut Context<Self>) -> Self {
        let tabs = look
            .tabs_navigation("button-theme-inspector-variant-tabs")
            .items(variant_tabs())
            .active(ThemeVariant::Primary.id())
            .spawn(cx);

        let variants =
            theme_variants().into_iter().map(|variant| variant_controls(look.clone(), variant, cx)).collect();

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |inspector, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            inspector.active_variant = ThemeVariant::from_id(tab_id.as_ref());
            cx.notify();
        }));

        Self {
            synced_mode: look.mode(),
            look,
            tabs,
            variants,
            active_variant: ThemeVariant::Primary,
            _subscriptions: subscriptions,
        }
    }

    fn active_tree(&self) -> Entity<AccordionControl> {
        self.variants
            .iter()
            .find(|controls| controls.variant == self.active_variant)
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

        div()
            .id("button-theme-inspector")
            .size_full()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .overflow_hidden()
            .border_1()
            .border_color(chrome.border)
            .rounded(px(8.0))
            .bg(chrome.panel_background)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .border_b_1()
                    .border_color(chrome.border)
                    .p(px(layout::PANEL_PADDING))
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .justify_between()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_size(px(body.size + 3.0))
                                    .line_height(px(body.line_height + 3.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(chrome.title_text)
                                    .child("ThemeInspector"),
                            )
                            .child(
                                div()
                                    .text_size(px(caption.size))
                                    .line_height(px(caption.line_height))
                                    .text_color(chrome.muted_text)
                                    .child("Button"),
                            ),
                    )
                    .child(self.tabs.clone()),
            )
            .child(
                div()
                    .id("button-theme-inspector-scroll")
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .overflow_y_scroll()
                    .p(px(layout::PANEL_PADDING))
                    .child(self.active_tree()),
            )
    }
}

fn variant_tabs() -> [TabsNavigationItem; 4] {
    theme_variants().map(|variant| TabsNavigationItem::new(variant.id()).label(variant.label()))
}

fn theme_variants() -> [ThemeVariant; 4] {
    [ThemeVariant::Primary, ThemeVariant::Secondary, ThemeVariant::Outline, ThemeVariant::Ghost]
}

fn variant_controls(
    look: Arc<ShadcnLook>,
    variant: ThemeVariant,
    cx: &mut Context<ThemeInspector>,
) -> VariantInspectorControls {
    let state_trees = state_specs()
        .into_iter()
        .map(|spec| {
            let tree = cx.new(|cx| ThemePropertyTree::new(look.clone(), variant, spec.state, cx));
            (spec, tree)
        })
        .collect();
    let state = state_accordion(look, variant, state_trees, cx);
    VariantInspectorControls { variant, state }
}

fn state_accordion(
    look: Arc<ShadcnLook>,
    variant: ThemeVariant,
    state_trees: Vec<(ThemeStateSpec, Entity<ThemePropertyTree>)>,
    cx: &mut Context<ThemeInspector>,
) -> Entity<AccordionControl> {
    look.accordion(format!("button-theme-inspector-{}-states", variant.id()))
        .mode(AccordionSelectionMode::Multiple)
        .item_dividers(false)
        .trigger_min_height(30.0)
        .trigger_padding_y(3.0)
        .content_padding_top(2.0)
        .content_padding_bottom(6.0)
        .items(state_trees.into_iter().map(|(spec, tree)| state_item(variant, spec, tree)))
        .spawn(cx)
}

fn state_item(variant: ThemeVariant, spec: ThemeStateSpec, tree: Entity<ThemePropertyTree>) -> AccordionItem {
    AccordionItem::new(
        format!("{}-{}", variant.id(), spec.id),
        AccordionTrigger::new(spec.label).icon(spec.icon),
        AccordionContent::custom({
            let tree = tree.clone();
            move |_, _| tree.clone().into_any_element()
        }),
    )
    .expanded(spec.expanded)
}

fn state_specs() -> [ThemeStateSpec; 5] {
    [
        ThemeStateSpec {
            id: "default",
            label: "Default",
            icon: LucideIcon::Circle,
            state: InteractionState::default(),
            expanded: true,
        },
        ThemeStateSpec {
            id: "hover",
            label: "Hover",
            icon: LucideIcon::MousePointer2,
            state: InteractionState { hovered: true, ..InteractionState::default() },
            expanded: false,
        },
        ThemeStateSpec {
            id: "focused",
            label: "Focused",
            icon: LucideIcon::Focus,
            state: InteractionState { focused: true, ..InteractionState::default() },
            expanded: false,
        },
        ThemeStateSpec {
            id: "pressed",
            label: "Pressed",
            icon: LucideIcon::MousePointerClick,
            state: InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
            expanded: false,
        },
        ThemeStateSpec {
            id: "disabled",
            label: "Disabled",
            icon: LucideIcon::CircleOff,
            state: InteractionState { disabled: true, ..InteractionState::default() },
            expanded: false,
        },
    ]
}

impl ThemePropertyTree {
    fn new(look: Arc<ShadcnLook>, variant: ThemeVariant, state: InteractionState, cx: &mut Context<Self>) -> Self {
        let layout = cx.new(|cx| LayoutSizeInspector::new(look.clone(), variant, state, cx));
        let categories = category_accordion(look, variant, state, layout, cx);
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
    variant: ThemeVariant,
    state: InteractionState,
    layout: Entity<LayoutSizeInspector>,
    cx: &mut Context<ThemePropertyTree>,
) -> Entity<AccordionControl> {
    look.accordion(format!("button-theme-inspector-{}-categories", variant.id()))
        .mode(AccordionSelectionMode::Multiple)
        .item_dividers(false)
        .trigger_min_height(28.0)
        .trigger_padding_y(2.0)
        .content_padding_top(2.0)
        .content_padding_bottom(8.0)
        .items(
            property_categories()
                .into_iter()
                .map(|category| category_item(look.clone(), variant, state, layout.clone(), category)),
        )
        .spawn(cx)
}

fn property_categories() -> [ThemePropertyCategory; 4] {
    [
        ThemePropertyCategory::Color,
        ThemePropertyCategory::Layout,
        ThemePropertyCategory::Elevation,
        ThemePropertyCategory::Typography,
    ]
}

fn category_item(
    look: Arc<ShadcnLook>,
    variant: ThemeVariant,
    state: InteractionState,
    layout: Entity<LayoutSizeInspector>,
    category: ThemePropertyCategory,
) -> AccordionItem {
    AccordionItem::new(
        category.id(),
        AccordionTrigger::new(category.label()).icon(category.icon()),
        AccordionContent::custom(move |_, _| match category {
            ThemePropertyCategory::Color => render_color_category(variant, state, &look),
            ThemePropertyCategory::Layout => layout.clone().into_any_element(),
            ThemePropertyCategory::Elevation => render_elevation_detail(variant, state, &look),
            ThemePropertyCategory::Typography => render_typography_category(&look),
        }),
    )
    .expanded(true)
}

impl LayoutSizeInspector {
    fn new(look: Arc<ShadcnLook>, variant: ThemeVariant, state: InteractionState, cx: &mut Context<Self>) -> Self {
        let tabs = look
            .tabs_navigation(format!("button-theme-inspector-{}-size-tabs", variant.id()))
            .items(size_tabs())
            .active("md")
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tabs, |inspector, _, event: &TabsNavigationEvent, cx| {
            let TabsNavigationEvent::Activate { tab_id, .. } = event;
            inspector.active_size = control_size_from_id(tab_id.as_ref());
            cx.notify();
        }));

        Self { look, tabs, variant, state, active_size: ControlSize::Md, _subscriptions: subscriptions }
    }
}

impl Render for LayoutSizeInspector {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .min_w(px(0.0))
            .flex()
            .flex_col()
            .gap(px(layout::DETAIL_GAP))
            .child(self.tabs.clone())
            .child(render_size_detail(self.variant, self.state, self.active_size, &self.look))
    }
}

fn size_tabs() -> [TabsNavigationItem; 3] {
    [
        TabsNavigationItem::new("sm").label("sm"),
        TabsNavigationItem::new("md").label("md"),
        TabsNavigationItem::new("lg").label("lg"),
    ]
}

fn control_size_from_id(id: &str) -> ControlSize {
    match id {
        "sm" => ControlSize::Sm,
        "lg" => ControlSize::Lg,
        _ => ControlSize::Md,
    }
}

fn render_color_category(variant: ThemeVariant, state: InteractionState, look: &Arc<ShadcnLook>) -> AnyElement {
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(variant.style(), ButtonFamilyRole::Text, state);
    let mut rows = vec![
        color_property("background", &palette.background),
        color_property("foreground", &palette.foreground),
        color_property("border", &palette.border),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_property("focus ring", focus_ring));
    }

    render_rows_panel(
        look,
        rows.into_iter()
            .enumerate()
            .map(|(index, row)| render_color_row(row, look, index > 0))
            .collect::<Vec<_>>(),
    )
}

fn render_size_detail(
    variant: ThemeVariant,
    state: InteractionState,
    size: ControlSize,
    look: &Arc<ShadcnLook>,
) -> AnyElement {
    let inspect = ShadcnInspect::new(look);
    let metrics = inspect.inspect_button_metrics(variant.style(), ButtonFamilyRole::Text, size, state);
    let box_model = InspectBoxModelSnapshot::from_button_metrics(&metrics);
    let occupation = button_family_occupation(
        look,
        variant.style(),
        ButtonFamilyRole::Text,
        size,
        state,
        metrics.focus_ring_offset.value_px,
    );
    let mut rows = metric_rows(&metrics);
    rows.extend(occupation_metric_properties(box_model.height, &occupation).into_iter().map(|row| PropertyRow {
        label: row.name.to_string(),
        value: row.value.to_string(),
        source: row.source.to_string(),
        detail: row.provenance.map(|value| value.to_string()),
    }));

    let chrome = look.chrome();
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(layout::DETAIL_GAP))
        .child(render_box_model_diagram(
            SharedString::from("button-theme-inspector-box-model"),
            &box_model,
            Some(&occupation),
            MetricFieldHighlight::None,
            neutral_box_model_colors(look.mode()),
            chrome.border,
            neutral_box_model_label_color(look.mode()),
            caption,
            mono_font.clone(),
        ))
        .child(render_rows_panel(
            look,
            rows.into_iter()
                .enumerate()
                .map(|(index, row)| render_property_row(row, look, index > 0))
                .collect::<Vec<_>>(),
        ))
        .into_any_element()
}

fn neutral_box_model_colors(mode: ThemeMode) -> BoxModelLayerColors {
    match mode {
        ThemeMode::Dark => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.16, 1.0),
            border: hsla(0.0, 0.0, 0.24, 1.0),
            padding: hsla(0.0, 0.0, 0.32, 1.0),
            content: hsla(0.0, 0.0, 0.40, 1.0),
            stroke: hsla(0.0, 0.0, 0.88, 0.92),
            highlight: hsla(0.0, 0.0, 0.96, 1.0),
        },
        ThemeMode::Light => BoxModelLayerColors {
            margin: hsla(0.0, 0.0, 0.94, 1.0),
            border: hsla(0.0, 0.0, 0.86, 1.0),
            padding: hsla(0.0, 0.0, 0.78, 1.0),
            content: hsla(0.0, 0.0, 0.70, 1.0),
            stroke: hsla(0.0, 0.0, 0.18, 0.88),
            highlight: hsla(0.0, 0.0, 0.08, 1.0),
        },
    }
}

fn neutral_box_model_label_color(mode: ThemeMode) -> Hsla {
    match mode {
        ThemeMode::Dark => hsla(0.0, 0.0, 0.94, 0.96),
        ThemeMode::Light => hsla(0.0, 0.0, 0.14, 0.96),
    }
}

fn render_elevation_detail(variant: ThemeVariant, state: InteractionState, look: &Arc<ShadcnLook>) -> AnyElement {
    let elevation = ShadcnInspect::new(look).inspect_button_elevation(variant.style(), ButtonFamilyRole::Text, state);
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    let rows = [
        PropertyRow::new("applied", if elevation.applied { "yes" } else { "no" }, "resolved button look"),
        PropertyRow::new("rule", elevation.rule_shadow.as_str(), "style.toml - elevation_rules"),
        PropertyRow::new("style", elevation.style_key.as_str(), "button.elevation_rules[].style"),
        PropertyRow::new(
            "token",
            elevation.token.as_deref().unwrap_or("none"),
            elevation.catalog_value.as_deref().unwrap_or("catalog token"),
        ),
    ];

    let mut preview_chip = div()
        .w(px(120.0))
        .h(px(36.0))
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .border_1()
        .border_color(chrome.border);

    if elevation.applied
        && let Some(shadows) = elevation.shadows.as_ref().filter(|shadows| !shadows.is_empty())
    {
        preview_chip = preview_chip.shadow(shadows.clone());
    }

    div()
        .w_full()
        .min_w(px(0.0))
        .flex()
        .flex_col()
        .gap(px(layout::DETAIL_GAP))
        .child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(6.0))
                .bg(chrome.content_background)
                .p(px(10.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(body.size))
                        .line_height(px(body.line_height))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child("Shadow preview"),
                )
                .child(div().w_full().h(px(78.0)).flex().items_center().justify_center().child(preview_chip)),
        )
        .child(render_rows_panel(
            look,
            rows.into_iter()
                .enumerate()
                .map(|(index, row)| render_property_row(row, look, index > 0))
                .collect::<Vec<_>>(),
        ))
        .when_some(elevation.catalog_value.clone(), |stack, catalog_value| {
            stack.child(
                div()
                    .w_full()
                    .border_1()
                    .border_color(chrome.border)
                    .rounded(px(6.0))
                    .bg(chrome.content_background)
                    .p(px(10.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child("Catalog value"),
                    )
                    .child(
                        div()
                            .font_family(mono_font.clone())
                            .text_size(px(caption.size))
                            .line_height(px(caption.line_height))
                            .text_color(chrome.muted_text)
                            .child(catalog_value),
                    ),
            )
        })
        .child(render_shadow_layers(look, &elevation.layers))
        .into_any_element()
}

fn render_shadow_layers(
    look: &Arc<ShadcnLook>,
    layers: &[gpui_luma_look_shadcn_inspect::ButtonInspectElevationLayer],
) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    let mut stack = div()
        .w_full()
        .min_w(px(0.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.content_background)
        .p(px(10.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.title_text)
                .child("Shadow layers"),
        );

    if layers.is_empty() {
        stack = stack.child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.muted_text)
                .child("No shadow layers resolved for this state."),
        );
    } else {
        for (index, layer) in layers.iter().enumerate() {
            stack = stack.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child(format!("layer {}", index + 1)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .size(px(12.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(layer.color)
                                    .border_1()
                                    .border_color(chrome.border),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .font_family(mono_font.clone())
                                    .text_size(px(caption.size))
                                    .line_height(px(caption.line_height))
                                    .text_color(chrome.muted_text)
                                    .child(format_inspect_box_shadow_layer(layer)),
                            ),
                    ),
            );
        }
    }

    stack.into_any_element()
}

fn render_typography_category(look: &Arc<ShadcnLook>) -> AnyElement {
    let typography = ShadcnInspect::new(look).inspect_button_typography();
    let rows = [
        PropertyRow::new("font family", typography.font_family.value.as_str(), "typography.text.body.font_family"),
        PropertyRow::new("font size", typography.font_size.value.as_str(), "typography.text.body.size"),
        PropertyRow::new("font weight", typography.font_weight.value.as_str(), "typography.text.body.weight"),
        PropertyRow::new("line height", typography.line_height.value.as_str(), "typography.text.body.line_height"),
    ];
    render_rows_panel(
        look,
        rows.into_iter()
            .enumerate()
            .map(|(index, row)| render_property_row(row, look, index > 0))
            .collect::<Vec<_>>(),
    )
}

fn render_rows_panel(look: &Arc<ShadcnLook>, rows: Vec<AnyElement>) -> AnyElement {
    let chrome = look.chrome();
    div()
        .w_full()
        .min_w(px(0.0))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .overflow_hidden()
        .children(rows)
        .into_any_element()
}

#[derive(Clone)]
struct ColorRow {
    label: &'static str,
    value: Hsla,
    source: String,
    detail: Option<String>,
}

fn color_property(label: &'static str, color: &ResolvedColor) -> ColorRow {
    ColorRow {
        label,
        value: color.value,
        source: format_inspect_css_key(&color.source),
        detail: format_inspect_provenance(&color.source),
    }
}

fn render_color_row(row: ColorRow, look: &Arc<ShadcnLook>, show_separator: bool) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    div()
        .min_h(px(34.0))
        .min_w(px(0.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .when(show_separator, |row| row.border_t_1().border_color(chrome.border))
        .py(px(6.0))
        .pl(px(9.0))
        .pr(px(8.0))
        .child(
            div()
                .w(px(18.0))
                .h(px(12.0))
                .flex_shrink_0()
                .rounded(px(3.0))
                .border_1()
                .border_color(chrome.border)
                .bg(row.value),
        )
        .child(
            div()
                .w(px(94.0))
                .flex_shrink_0()
                .truncate()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.body_text)
                .child(row.label),
        )
        .child(
            div()
                .w(px(74.0))
                .flex_shrink_0()
                .font_family(mono_font.clone())
                .text_size(px(caption.size))
                .line_height(px(caption.line_height))
                .text_color(chrome.muted_text)
                .child(format_hex_color(row.value)),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .font_family(mono_font)
                .text_size(px(caption.size))
                .line_height(px(caption.line_height))
                .text_color(chrome.muted_text)
                .child(row.detail.unwrap_or(row.source)),
        )
        .into_any_element()
}

#[derive(Clone)]
struct PropertyRow {
    label: String,
    value: String,
    source: String,
    detail: Option<String>,
}

impl PropertyRow {
    fn new(label: impl Into<String>, value: impl Into<String>, source: impl Into<String>) -> Self {
        Self { label: label.into(), value: value.into(), source: source.into(), detail: None }
    }
}

fn metric_rows(metrics: &ButtonInspectMetrics) -> Vec<PropertyRow> {
    [
        ("height", &metrics.height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
        ("radius", &metrics.radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]
    .into_iter()
    .map(|(label, metric)| metric_property(label, metric))
    .collect()
}

fn metric_property(label: &str, metric: &ResolvedMetric) -> PropertyRow {
    PropertyRow {
        label: label.to_owned(),
        value: format_metric_px(metric.value_px),
        source: format_inspect_metric_source(&metric.source),
        detail: format_inspect_metric_provenance(&metric.source),
    }
}

fn render_property_row(row: PropertyRow, look: &Arc<ShadcnLook>, show_separator: bool) -> AnyElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let caption = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    div()
        .min_h(px(34.0))
        .min_w(px(0.0))
        .flex()
        .flex_wrap()
        .items_start()
        .gap(px(4.0))
        .when(show_separator, |row| row.border_t_1().border_color(chrome.border))
        .px(px(9.0))
        .py(px(6.0))
        .child(
            div()
                .w(relative(0.28))
                .min_w(px(180.0))
                .flex_shrink_0()
                .mr(px(6.0))
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.body_text)
                .child(row.label),
        )
        .child(
            div()
                .w(px(54.0))
                .flex_shrink_0()
                .mr(px(6.0))
                .font_family(mono_font.clone())
                .text_size(px(caption.size))
                .line_height(px(caption.line_height))
                .text_color(chrome.title_text)
                .child(row.value),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(260.0))
                .font_family(mono_font)
                .text_size(px(caption.size))
                .line_height(px(caption.line_height))
                .text_color(chrome.muted_text)
                .child(row.detail.unwrap_or(row.source)),
        )
        .into_any_element()
}
