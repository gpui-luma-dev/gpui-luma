use std::sync::Arc;

use gpui::{
    AnyElement, App, Div, IntoElement, SharedString, Stateful, Window, div, prelude::*, px,
};
use gpui_luma::controls::tree_view::{
    FlatTreeNode, TreeNode, TreeViewRenderModel, TreeViewSelectionMode, TreeViewTemplate, TreeViewTemplateHandlers,
};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::theme::InteractionState;
use gpui_luma_look_shadcn::{
    ButtonInspectPalette, ShadcnButtonStyle, ShadcnLook, format_inspect_css_key, format_inspect_provenance,
};

use crate::fonts::gallery_mono_font;
use lucide_icons::Icon as LucideIcon;

mod layout {
    pub(super) const ROW_HEIGHT: f32 = 32.0;
    pub(super) const ROW_PADDING_RIGHT: f32 = 8.0;

    pub(super) const INDENT_BASE: f32 = 8.0;
    pub(super) const INDENT_PER_DEPTH: f32 = 14.0;

    pub(super) const CHEVRON_SLOT_SIZE: f32 = 14.0;
    pub(super) const CHEVRON_LABEL_GAP: f32 = 6.0;

    pub(super) const FIELD_SWATCH_SIZE: f32 = 12.0;
    pub(super) const FIELD_SWATCH_GAP: f32 = 8.0;
    pub(super) const FIELD_LABEL_GAP: f32 = 8.0;
}

fn row_indent(depth: usize) -> gpui::Pixels {
    px(layout::INDENT_BASE + depth as f32 * layout::INDENT_PER_DEPTH)
}

#[derive(Clone)]
pub(in crate::gallery) enum InspectTreeData {
    Branch,
    Field(InspectFieldData),
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectFieldData {
    pub css_key: SharedString,
    pub provenance: Option<SharedString>,
    pub swatch: gpui::Hsla,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectFieldSelection {
    pub id: SharedString,
    pub label: SharedString,
    pub field: InspectFieldData,
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
        ("focused", InteractionState { focused: true, ..InteractionState::default() }),
        (
            "pressed",
            InteractionState { hovered: true, pressed: true, ..InteractionState::default() },
        ),
        ("disabled", InteractionState { disabled: true, ..InteractionState::default() }),
    ];

    button_styles
        .iter()
        .map(|(style, label)| {
            let style_slug = slug(label);
            let state_nodes: Vec<_> = matrix_states
                .iter()
                .map(|(state_label, state)| {
                    state_palette_branch(&style_slug, label, state_label, *style, *state, look)
                })
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
    let palette = look.inspect_button_color_palette(style, ButtonFamilyRole::Text, state);
    TreeNode::new(id.clone(), state_label.to_owned(), InspectTreeData::Branch)
        .branch(true)
        .expanded(style_label == "Primary" && state_label == "default")
        .children(field_nodes(&id.to_string(), &palette))
}

fn field_nodes(prefix: &str, palette: &ButtonInspectPalette) -> Vec<TreeNode<InspectTreeData>> {
    let mut fields: Vec<(&str, &gpui_luma_look_shadcn::ResolvedColor)> = vec![
        ("background", &palette.background),
        ("foreground", &palette.foreground),
        ("border", &palette.border),
    ];
    if let Some(ring) = &palette.focus_ring {
        fields.push(("focus ring", ring));
    }

    fields
        .into_iter()
        .map(|(name, color)| field_node(prefix, name, color))
        .collect()
}

fn field_node(prefix: &str, name: &str, color: &gpui_luma_look_shadcn::ResolvedColor) -> TreeNode<InspectTreeData> {
    let id: SharedString = format!("{prefix}-{}", name.replace(' ', "-")).into();
    TreeNode::new(
        id,
        name.to_owned(),
        InspectTreeData::Field(InspectFieldData {
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
            if let InspectTreeData::Field(field) = &node.data {
                return Some(InspectFieldSelection {
                    id: node.id.clone(),
                    label: node.label.clone(),
                    field: field.clone(),
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
            if matches!(node.data, InspectTreeData::Field(_)) {
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
        let palette = theme.resolve_row(node.state.interaction_state(), node.state.selected);
        let body = &self.look.mode_tokens().typography.text.body;
        let mono = &self.look.mode_tokens().typography.text.caption;
        let mono_font = gallery_mono_font();
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
            InspectTreeData::Field(field) => {
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

                row.child(div().size(px(layout::CHEVRON_SLOT_SIZE)))
                    .child(color_circle(field.swatch, chrome.border))
                    .child(div().w(px(layout::FIELD_SWATCH_GAP)))
                    .child(
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
                            .max_w(px(140.0))
                            .truncate()
                            .font_family(mono_font)
                            .text_size(px(mono.size))
                            .line_height(px(mono.line_height))
                            .text_color(chrome.muted_text)
                            .child(field.css_key.clone()),
                    )
                    .into_any_element()
            }
        }
    }
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
    let template: Arc<dyn TreeViewTemplate<InspectTreeData>> =
        Arc::new(InspectorTreeTemplate::new(look.clone()));
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
