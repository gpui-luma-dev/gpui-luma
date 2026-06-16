#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{AnyElement, App, Div, Entity, IntoElement, SharedString, Stateful, Window, div, prelude::*, px};
use gpui_luma::controls::tree_view::{
    FlatTreeNode, TreeNode, TreeViewControl, TreeViewRenderModel, TreeViewSelectionMode, TreeViewTemplate,
    TreeViewTemplateHandlers,
};
use gpui_luma_look_shadcn::{ShadcnLook};
use gpui_luma_look_shadcn_inspect::{ResolvedColor, format_inspect_css_key, format_inspect_provenance};

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::format_hex_color;
use lucide_icons::Icon as LucideIcon;

use super::types::{ColorInspectTreeData, InspectColorFieldData, first_inspect_field_id};

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

pub(in crate::gallery) fn color_field_node(
    prefix: &str,
    name: &str,
    color: &ResolvedColor,
) -> TreeNode<ColorInspectTreeData> {
    let id: SharedString = format!("{prefix}-{}", name.replace(' ', "-")).into();
    TreeNode::new(
        id,
        name.to_owned(),
        ColorInspectTreeData::ColorField(InspectColorFieldData {
            css_key: format_inspect_css_key(&color.source).into(),
            provenance: format_inspect_provenance(&color.source).map(SharedString::from),
            swatch: color.value,
        }),
    )
}

pub(in crate::gallery) fn color_field_nodes_optional(
    prefix: &str,
    fields: &[(&str, Option<&ResolvedColor>)],
) -> Vec<TreeNode<ColorInspectTreeData>> {
    fields
        .iter()
        .filter_map(|(name, color)| color.map(|color| color_field_node(prefix, name, color)))
        .collect()
}

pub(in crate::gallery) struct ColorInspectorTreeTemplate {
    look: Arc<ShadcnLook>,
}

impl ColorInspectorTreeTemplate {
    pub fn new(look: Arc<ShadcnLook>) -> Self {
        Self { look }
    }
}

impl TreeViewTemplate<ColorInspectTreeData> for ColorInspectorTreeTemplate {
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
        node: &FlatTreeNode<'_, ColorInspectTreeData>,
        handlers: TreeViewTemplateHandlers,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let chrome = self.look.chrome();
        let theme = self.look.tree_view_theme();
        let palette = theme.resolve_row(node.state.interaction_state(), node.state.selected);
        let body = &self.look.mode_tokens().typography.text.body;
        let mono = &self.look.mode_tokens().typography.text.caption;
        let left_padding = row_indent(node.depth);

        match node.data {
            ColorInspectTreeData::Branch => {
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
            ColorInspectTreeData::ColorField(field) => {
                let value = format_hex_color(field.swatch);
                render_leaf_row(node, handlers, palette, left_padding, chrome, body, mono, Some(field.swatch), value)
            }
            ColorInspectTreeData::LayoutSize(_) => {
                render_leaf_row(node, handlers, palette, left_padding, chrome, body, mono, None, node.label.to_string())
            }
        }
    }
}

fn render_leaf_row(
    node: &FlatTreeNode<'_, ColorInspectTreeData>,
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
    node: &FlatTreeNode<'_, ColorInspectTreeData>,
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
    node: &FlatTreeNode<'_, ColorInspectTreeData>,
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
    node: &FlatTreeNode<'_, ColorInspectTreeData>,
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

pub(in crate::gallery) fn spawn_color_inspector_tree(
    tree_id: &'static str,
    look: Arc<ShadcnLook>,
    build_tree: fn(&ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>>,
    cx: &mut impl gpui::AppContext,
) -> Entity<TreeViewControl<ColorInspectTreeData>> {
    let template: Arc<dyn TreeViewTemplate<ColorInspectTreeData>> =
        Arc::new(ColorInspectorTreeTemplate::new(look.clone()));
    let items = build_tree(&look);
    let initial_selection = first_inspect_field_id(&items);
    let tree = gpui_luma::controls::tree_view::new(tree_id)
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
