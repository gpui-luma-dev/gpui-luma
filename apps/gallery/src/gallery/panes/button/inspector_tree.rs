#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{AnyElement, App, Div, IntoElement, SharedString, Stateful, Window, div, prelude::*, px};
use gpui_luma::controls::tree_view::{
    FlatTreeNode, TreeNode, TreeViewRenderModel, TreeViewSelectionMode, TreeViewTemplate, TreeViewTemplateHandlers,
};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{
    ButtonInspectElevation, ButtonInspectMetrics, ButtonInspectPalette, format_inspect_box_shadow_layer,
    format_inspect_css_key, format_inspect_metric_provenance, format_inspect_metric_source, format_inspect_provenance,
    format_inspect_typography_provenance, format_inspect_typography_source, format_metric_px, format_typography_px,
};

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::format_hex_color;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::panes::shared::inspector::box_model::{InspectBoxModelSnapshot, InspectOccupationSnapshot};
use crate::gallery::panes::shared::inspector::occupation::{button_family_occupation, occupation_metric_properties};

mod layout {
    pub(super) const ROW_HEIGHT: f32 = 32.0;
    pub(super) const ROW_PADDING_RIGHT: f32 = 8.0;

    pub(super) const INDENT_BASE: f32 = 4.0;
    pub(super) const INDENT_PER_DEPTH: f32 = 10.0;

    pub(super) const CHEVRON_SLOT_SIZE: f32 = 14.0;
    pub(super) const CHEVRON_LABEL_GAP: f32 = 4.0;

    pub(super) const FIELD_SWATCH_SIZE: f32 = 12.0;
    pub(super) const FIELD_SWATCH_GAP: f32 = 6.0;
    pub(super) const FIELD_LABEL_GAP: f32 = 6.0;
}

fn row_indent(depth: usize) -> gpui::Pixels {
    px(layout::INDENT_BASE + depth as f32 * layout::INDENT_PER_DEPTH)
}

#[derive(Clone)]
pub(in crate::gallery) enum InspectTreeData {
    Branch,
    ColorField(InspectColorFieldData),
    LayoutSize(InspectLayoutSizeData),
    Typography(InspectTypographyData),
    Elevation(InspectElevationData),
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectColorFieldData {
    pub css_key: SharedString,
    pub provenance: Option<SharedString>,
    pub swatch: gpui::Hsla,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectMetricPropertyData {
    pub name: SharedString,
    pub value: SharedString,
    pub source: SharedString,
    pub provenance: Option<SharedString>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectLayoutSizeData {
    pub size: ControlSize,
    pub box_model: InspectBoxModelSnapshot,
    pub occupation: InspectOccupationSnapshot,
    pub properties: Vec<InspectMetricPropertyData>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectElevationLayerData {
    pub label: SharedString,
    pub css: SharedString,
    pub color: gpui::Hsla,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectElevationData {
    pub applied: bool,
    pub rule_shadow: SharedString,
    pub style_key: SharedString,
    pub token: Option<SharedString>,
    pub catalog_value: Option<SharedString>,
    pub layers: Vec<InspectElevationLayerData>,
    pub properties: Vec<InspectMetricPropertyData>,
    pub shadows: Option<Vec<gpui::BoxShadow>>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectTypographyPropertyData {
    pub name: SharedString,
    pub value: SharedString,
    pub source: SharedString,
    pub provenance: Option<SharedString>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectTypographyData {
    pub font_family: SharedString,
    pub font_size: f32,
    pub font_weight: gpui::FontWeight,
    pub line_height: f32,
    pub properties: Vec<InspectTypographyPropertyData>,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectFieldSelection {
    pub id: SharedString,
    pub label: SharedString,
    pub kind: InspectFieldKind,
}

#[derive(Clone)]
pub(in crate::gallery) enum InspectFieldKind {
    Color(InspectColorFieldData),
    Layout(InspectLayoutSizeData),
    Typography(InspectTypographyData),
    Elevation(InspectElevationData),
}

pub(in crate::gallery) fn build_button_inspect_tree(look: &ShadcnLook) -> Vec<TreeNode<InspectTreeData>> {
    let button_styles = [
        (ShadcnButtonStyle::Primary, "Primary"),
        (ShadcnButtonStyle::Secondary, "Secondary"),
        (ShadcnButtonStyle::Outline, "Outline"),
        (ShadcnButtonStyle::Ghost, "Ghost"),
    ];
    let matrix_states = [
        ("default", InteractionState::default()),
        ("hover", InteractionState { hovered: true, ..InteractionState::default() }),
        ("pressed", InteractionState { hovered: true, pressed: true, ..InteractionState::default() }),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    button_styles
        .iter()
        .map(|(style, label)| {
            let style_slug = slug(label);
            let state_nodes: Vec<_> = matrix_states
                .iter()
                .map(|(state_label, state)| state_palette_branch(&style_slug, label, state_label, *style, *state, look))
                .collect();
            let id: SharedString = format!("inspect-{style_slug}").into();
            TreeNode::new(id.clone(), *label, InspectTreeData::Branch)
                .branch(true)
                .expanded(*label == "Primary")
                .children(state_nodes)
        })
        .collect()
}

fn state_palette_branch(
    prefix: &str,
    style_label: &str,
    state_label: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-{}-{}", slug(style_label), state_label).into();
    let palette = ShadcnInspect::new(look).inspect_button_color_palette(style, ButtonFamilyRole::Text, state);
    let expand_layout = style_label == "Primary" && state_label == "default";
    let mut children = field_nodes(id.as_ref(), &palette);
    children.push(layout_branch(id.as_ref(), style, state, look, expand_layout));
    children.push(elevation_branch(id.as_ref(), style, state, look));
    children.push(typography_branch(id.as_ref(), look));
    TreeNode::new(id.clone(), state_label.to_owned(), InspectTreeData::Branch)
        .branch(true)
        .expanded(style_label == "Primary" && state_label == "default")
        .children(children)
}

fn layout_branch(
    prefix: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
    expand_md: bool,
) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-layout").into();
    let sizes = [(ControlSize::Sm, "sm"), (ControlSize::Md, "md"), (ControlSize::Lg, "lg")];
    let children: Vec<_> = sizes
        .into_iter()
        .map(|(size, label)| size_metrics_branch(id.as_ref(), style, state, size, label, look, expand_md))
        .collect();
    TreeNode::new(id.clone(), "layout", InspectTreeData::Branch)
        .branch(true)
        .expanded(expand_md)
        .children(children)
}

fn size_metrics_branch(
    prefix: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    size: ControlSize,
    size_label: &str,
    look: &ShadcnLook,
    _expand_md: bool,
) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-{size_label}").into();
    let metrics = ShadcnInspect::new(look).inspect_button_metrics(style, ButtonFamilyRole::Text, size, state);
    let box_model = InspectBoxModelSnapshot::from_button_metrics(&metrics);
    let occupation =
        button_family_occupation(look, style, ButtonFamilyRole::Text, size, state, metrics.focus_ring_offset.value_px);
    let occupation_rows: Vec<InspectMetricPropertyData> = occupation_metric_properties(box_model.height, &occupation)
        .into_iter()
        .map(|row| InspectMetricPropertyData {
            name: row.name,
            value: row.value,
            source: row.source,
            provenance: row.provenance,
        })
        .collect();
    let properties = [layout_metric_properties(&metrics), occupation_rows].concat();
    TreeNode::new(
        id,
        size_label.to_owned(),
        InspectTreeData::LayoutSize(InspectLayoutSizeData { size, box_model, occupation, properties }),
    )
}

fn elevation_branch(
    prefix: &str,
    style: ShadcnButtonStyle,
    state: InteractionState,
    look: &ShadcnLook,
) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-elevation").into();
    let elevation = ShadcnInspect::new(look).inspect_button_elevation(style, ButtonFamilyRole::Text, state);
    TreeNode::new(id, "elevation", InspectTreeData::Elevation(elevation_data_from(&elevation)))
}

fn elevation_data_from(elevation: &ButtonInspectElevation) -> InspectElevationData {
    let layers = elevation
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| InspectElevationLayerData {
            label: format!("layer {}", index + 1).into(),
            css: format_inspect_box_shadow_layer(layer).into(),
            color: layer.color,
        })
        .collect();

    let token_display = elevation.token.as_ref().map(|token| format!("--{token}")).unwrap_or_else(|| "—".into());

    let properties = vec![
        elevation_property(
            "applied",
            if elevation.applied { "yes" } else { "no" },
            if elevation.applied {
                "resolved button look"
            } else if elevation.rule_shadow == "none" {
                "style.toml · shadow = none"
            } else {
                "disabled · template skips shadow"
            },
            None,
        ),
        elevation_property("rule", elevation.rule_shadow.clone(), "style.toml · button.elevation_rules", None),
        elevation_property(
            "style",
            elevation.style_key.clone(),
            "button.elevation_rules[].style",
            Some(format!("style = {}", elevation.style_key)),
        ),
        elevation_property("token", token_display.to_string(), "catalog token key", elevation.token.clone()),
    ];

    InspectElevationData {
        applied: elevation.applied,
        rule_shadow: elevation.rule_shadow.clone().into(),
        style_key: elevation.style_key.clone().into(),
        token: elevation.token.clone().map(SharedString::from),
        catalog_value: elevation.catalog_value.clone().map(SharedString::from),
        layers,
        properties,
        shadows: elevation.shadows.clone(),
    }
}

fn elevation_property(
    name: &'static str,
    value: impl Into<SharedString>,
    source: impl Into<SharedString>,
    provenance: Option<String>,
) -> InspectMetricPropertyData {
    InspectMetricPropertyData {
        name: SharedString::from(name),
        value: value.into(),
        source: source.into(),
        provenance: provenance.map(SharedString::from),
    }
}

fn typography_branch(prefix: &str, look: &ShadcnLook) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-typography").into();
    let typography = ShadcnInspect::new(look).inspect_button_typography();
    TreeNode::new(
        id,
        "typography".to_owned(),
        InspectTreeData::Typography(InspectTypographyData {
            font_family: typography.font_family.value.clone().into(),
            font_size: typography.font_size.value.parse().unwrap_or(13.0),
            font_weight: gpui::FontWeight(typography.font_weight.value.parse().unwrap_or(500.0)),
            line_height: typography.line_height.value.parse().unwrap_or(18.0),
            properties: typography_properties(&typography),
        }),
    )
}

fn typography_properties(
    typography: &gpui_luma_look_shadcn_inspect::ButtonInspectTypography,
) -> Vec<InspectTypographyPropertyData> {
    [
        ("font family", &typography.font_family),
        ("font size", &typography.font_size),
        ("font weight", &typography.font_weight),
        ("line height", &typography.line_height),
    ]
    .into_iter()
    .map(|(name, field)| InspectTypographyPropertyData {
        name: name.into(),
        value: typography_display_value(name, field).into(),
        source: format_inspect_typography_source(&field.source).into(),
        provenance: format_inspect_typography_provenance(&field.source).map(SharedString::from),
    })
    .collect()
}

fn typography_display_value(name: &str, field: &gpui_luma_look_shadcn_inspect::ResolvedTypography) -> String {
    match name {
        "font size" | "line height" => format_typography_px(field.value.parse().unwrap_or(0.0)),
        _ => field.value.clone(),
    }
}

fn layout_metric_properties(metrics: &ButtonInspectMetrics) -> Vec<InspectMetricPropertyData> {
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
    .map(|(name, metric)| InspectMetricPropertyData {
        name: name.into(),
        value: format_metric_px(metric.value_px).into(),
        source: format_inspect_metric_source(&metric.source).into(),
        provenance: format_inspect_metric_provenance(&metric.source).map(SharedString::from),
    })
    .collect()
}

fn field_nodes(prefix: &str, palette: &ButtonInspectPalette) -> Vec<TreeNode<InspectTreeData>> {
    let mut fields: Vec<(&str, &gpui_luma_look_shadcn_inspect::ResolvedColor)> = vec![
        ("background", &palette.background),
        ("foreground", &palette.foreground),
        ("border", &palette.border),
    ];

    fields.into_iter().map(|(name, color)| field_node(prefix, name, color)).collect()
}

fn field_node(
    prefix: &str,
    name: &str,
    color: &gpui_luma_look_shadcn_inspect::ResolvedColor,
) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-{}", name.replace(' ', "-")).into();
    TreeNode::new(
        id,
        name.to_owned(),
        InspectTreeData::ColorField(InspectColorFieldData {
            css_key: format_inspect_css_key(&color.source).into(),
            provenance: format_inspect_provenance(&color.source).map(SharedString::from),
            swatch: color.value,
        }),
    )
}

pub(in crate::gallery) fn find_field_selection(
    items: &[TreeNode<InspectTreeData>],
    id: &SharedString,
) -> Option<InspectFieldSelection> {
    for node in items {
        if node.id == *id {
            if let InspectTreeData::ColorField(field) = &node.data {
                return Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Color(field.clone()),
                });
            }
            if let InspectTreeData::LayoutSize(field) = &node.data {
                return Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Layout(field.clone()),
                });
            }
            if let InspectTreeData::Typography(field) = &node.data {
                return Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Typography(field.clone()),
                });
            }
            if let InspectTreeData::Elevation(field) = &node.data {
                return Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    kind: InspectFieldKind::Elevation(field.clone()),
                });
            }
            return None;
        }
        if let Some(selection) = find_field_selection(&node.children, id) {
            return Some(selection);
        }
    }
    None
}

pub(in crate::gallery) fn first_field_id(items: &[TreeNode<InspectTreeData>]) -> Option<SharedString> {
    fn walk(nodes: &[TreeNode<InspectTreeData>]) -> Option<SharedString> {
        for node in nodes {
            if matches!(
                node.data,
                InspectTreeData::ColorField(_)
                    | InspectTreeData::LayoutSize(_)
                    | InspectTreeData::Typography(_)
                    | InspectTreeData::Elevation(_)
            ) {
                return Some(node.id.clone());
            }
            if let Some(id) = walk(&node.children) {
                return Some(id);
            }
        }
        None
    }
    walk(items)
}

fn slug(label: &str) -> String {
    label.to_ascii_lowercase()
}

pub(in crate::gallery) struct InspectorTreeTemplate {
    look: Arc<ShadcnLook>,
}

impl InspectorTreeTemplate {
    pub fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl TreeViewTemplate<InspectTreeData> for InspectorTreeTemplate {
    fn render(
        &self,
        model: &TreeViewRenderModel<'_>,
        body: AnyElement,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Stateful<Div> {
        div().id(model.id.clone()).size_full().overflow_hidden().child(body)
    }

    fn render_node(
        &self,
        node: &FlatTreeNode<'_, InspectTreeData>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let chrome = self.look.chrome();
        let theme = self.look.tree_view_theme();
        let palette = theme.resolve_row(node.state.interaction_state(), node.state.selected, node.size);
        let body = &self.look.mode_tokens().typography.text.body;
        let mono = &self.look.mode_tokens().typography.text.caption;
        let left_padding = row_indent(node.depth);

        match node.data {
            InspectTreeData::Branch => {
                let mut row = branch_row_shell(node, palette.foreground, left_padding);
                if node.enabled {
                    row = row
                        .on_hover(handlers.hover)
                        .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
                        .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
                        .on_click(handlers.click);
                }
                if let Some(background) = palette.background {
                    row = row.bg(background);
                }
                row.child(branch_chevron(node, palette.chevron_color, window, cx))
                    .child(div().w(px(layout::CHEVRON_LABEL_GAP)))
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(body.weight)
                            .text_color(chrome.body_text)
                            .child(node.label.clone()),
                    )
                    .into_any_element()
            }
            InspectTreeData::ColorField(field) => {
                let value = format_hex_color(field.swatch);
                render_leaf_row(node, handlers, palette, left_padding, chrome, body, mono, Some(field.swatch), value)
            }
            InspectTreeData::LayoutSize(_) => {
                render_leaf_row(node, handlers, palette, left_padding, chrome, body, mono, None, node.label.to_string())
            }
            InspectTreeData::Typography(_) => {
                let mut row = branch_row_shell(node, palette.foreground, left_padding);
                if node.enabled {
                    row = row
                        .on_hover(handlers.hover)
                        .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
                        .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
                        .on_click(handlers.click);
                }
                if let Some(background) = palette.background {
                    row = row.bg(background);
                }
                row.child(div().size(px(layout::CHEVRON_SLOT_SIZE)))
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(body.weight)
                            .text_color(chrome.body_text)
                            .child(node.label.clone()),
                    )
                    .into_any_element()
            }
            InspectTreeData::Elevation(field) => render_leaf_row(
                node,
                handlers,
                palette,
                left_padding,
                chrome,
                body,
                mono,
                None,
                field.rule_shadow.to_string(),
            ),
        }
    }
}

fn render_leaf_row(
    node: &FlatTreeNode<'_, InspectTreeData>,
    handlers: TreeViewTemplateHandlers,
    palette: gpui_luma::controls::tree_view::TreeViewPalette,
    left_padding: gpui::Pixels,
    chrome: gpui_luma::theme::pack::LumaChrome,
    body: &gpui_luma::theme::LumaTextStyle,
    mono: &gpui_luma::theme::LumaTextStyle,
    swatch: Option<gpui::Hsla>,
    value: String,
) -> AnyElement {
    let mono_font = gallery_mono_font();
    let mut row = field_row_shell(node, palette.foreground, left_padding);
    if node.enabled {
        row = row
            .on_hover(handlers.hover)
            .on_mouse_down(gpui::MouseButton::Left, handlers.mouse_down)
            .on_mouse_up(gpui::MouseButton::Left, handlers.mouse_up)
            .on_click(handlers.click);
    }
    if let Some(background) = palette.background {
        row = row.bg(background);
    }

    row = row.child(div().size(px(layout::CHEVRON_SLOT_SIZE)));
    if let Some(color) = swatch {
        row = row.child(color_circle(color, chrome.border)).child(div().w(px(layout::FIELD_SWATCH_GAP)));
    }
    row.child(
        div()
            .flex_1()
            .min_w(px(0.0))
            .truncate()
            .text_size(px(body.size))
            .line_height(px(body.line_height))
            .font_weight(body.weight)
            .text_color(chrome.body_text)
            .child(node.label.clone()),
    )
    .child(div().w(px(layout::FIELD_LABEL_GAP)))
    .child(
        div()
            .flex_shrink_0()
            .font_family(mono_font)
            .text_size(px(mono.size))
            .line_height(px(mono.line_height))
            .text_color(chrome.muted_text)
            .child(value),
    )
    .into_any_element()
}

fn color_circle(color: gpui::Hsla, border: gpui::Hsla) -> Div {
    div()
        .size(px(layout::FIELD_SWATCH_SIZE))
        .flex_shrink_0()
        .rounded_full()
        .bg(color)
        .border_1()
        .border_color(border)
}

fn branch_row_shell(
    node: &FlatTreeNode<'_, InspectTreeData>,
    foreground: gpui::Hsla,
    left_padding: gpui::Pixels,
) -> Stateful<Div> {
    div()
        .id(format!("{}-row", node.id))
        .flex()
        .items_center()
        .w_full()
        .h(px(layout::ROW_HEIGHT))
        .pl(left_padding)
        .pr(px(layout::ROW_PADDING_RIGHT))
        .text_color(foreground)
        .cursor_pointer()
}

fn field_row_shell(
    node: &FlatTreeNode<'_, InspectTreeData>,
    foreground: gpui::Hsla,
    left_padding: gpui::Pixels,
) -> Stateful<Div> {
    div()
        .id(format!("{}-row", node.id))
        .flex()
        .items_center()
        .w_full()
        .h(px(layout::ROW_HEIGHT))
        .pl(left_padding)
        .pr(px(layout::ROW_PADDING_RIGHT))
        .text_color(foreground)
        .cursor_pointer()
}

fn branch_chevron(
    node: &FlatTreeNode<'_, InspectTreeData>,
    color: gpui::Hsla,
    _window: &mut Window,
    _cx: &mut App,
) -> AnyElement {
    if !node.has_children {
        return div().size(px(layout::CHEVRON_SLOT_SIZE)).into_any_element();
    }

    let icon = if node.expanded {
        LucideIcon::ChevronDown
    } else {
        LucideIcon::ChevronRight
    };

    div()
        .size(px(layout::CHEVRON_SLOT_SIZE))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .text_size(px(layout::CHEVRON_SLOT_SIZE))
        .line_height(px(layout::CHEVRON_SLOT_SIZE))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}

pub(in crate::gallery) fn spawn_button_inspector_tree(
    look: Arc<ShadcnLook>,
    cx: &mut impl gpui::AppContext,
) -> gpui::Entity<gpui_luma::controls::tree_view::TreeViewControl<InspectTreeData>> {
    let template: Arc<dyn TreeViewTemplate<InspectTreeData>> = Arc::new(InspectorTreeTemplate::new(look.clone()));
    let items = build_button_inspect_tree(&look);
    let initial_selection = first_field_id(&items);
    let tree = gpui_luma::controls::tree_view::new("button-inspector-tree")
        .selection_mode(TreeViewSelectionMode::Single)
        .template(template)
        .items(items)
        .spawn(cx);

    if let Some(id) = initial_selection {
        tree.update(cx, |tree, cx| {
            tree.select_node_by_id(id, cx);
        });
    }

    tree
}
