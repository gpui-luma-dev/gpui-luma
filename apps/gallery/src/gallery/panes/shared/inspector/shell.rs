use std::sync::Arc;

use gpui::{Context, Entity, FontWeight, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation,
};
use gpui_luma::controls::tree_view::{TreeNode, TreeViewControl, TreeViewEvent};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;

use super::detail::ColorInspectorDetail;
use super::types::{ColorInspectTreeData, find_inspect_field_selection};

const TREE_PANEL_MIN_PX: f32 = 180.0;
const DETAIL_PANEL_MIN_PX: f32 = 180.0;

pub(in crate::gallery) struct ColorInspectorShell {
    look: Arc<ShadcnLook>,
    tree: Entity<TreeViewControl<ColorInspectTreeData>>,
    detail: Entity<ColorInspectorDetail>,
    split: Entity<ResizablePanels>,
    synced_mode: ThemeMode,
    inspector_id: &'static str,
    rebuild_tree: fn(&ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>>,
    _subscriptions: Vec<Subscription>,
}

impl ColorInspectorShell {
    pub fn tree(&self) -> Entity<TreeViewControl<ColorInspectTreeData>> {
        self.tree.clone()
    }

    pub fn detail(&self) -> Entity<ColorInspectorDetail> {
        self.detail.clone()
    }

    pub fn split(&self) -> Entity<ResizablePanels> {
        self.split.clone()
    }

    pub fn new(
        look: Arc<ShadcnLook>,
        tree: Entity<TreeViewControl<ColorInspectTreeData>>,
        inspector_id: &'static str,
        split_id: &'static str,
        detail_id: &'static str,
        rebuild_tree: fn(&ShadcnLook) -> Vec<TreeNode<ColorInspectTreeData>>,
        cx: &mut Context<Self>,
    ) -> Self {
        let synced_mode = look.mode();
        let panel_bg = look.chrome().panel_background;
        let detail = cx.new(|cx| ColorInspectorDetail::new(look.clone(), tree.clone(), detail_id, cx));

        let tree_panel = {
            let tree = tree.clone();
            let tree_host_id = format!("{inspector_id}-tree-host");
            ResizablePanelSpec::new_render(move || {
                div()
                    .id(tree_host_id.clone())
                    .size_full()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .child(tree.clone())
            })
            .weight(5.0)
            .min(px(TREE_PANEL_MIN_PX))
            .bg(panel_bg)
        };

        let detail_panel = {
            let detail = detail.clone();
            let detail_host_id = format!("{inspector_id}-detail-host");
            ResizablePanelSpec::new_render(move || {
                div()
                    .id(detail_host_id.clone())
                    .size_full()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .overflow_hidden()
                    .child(detail.clone())
            })
            .weight(5.0)
            .min(px(DETAIL_PANEL_MIN_PX))
            .bg(panel_bg)
        };

        let split = look
            .resizable_panels(split_id)
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_border(false)
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Sm)
            .handle_grip(true)
            .panels([tree_panel, detail_panel])
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tree, |shell, _, event: &TreeViewEvent<ColorInspectTreeData>, cx| {
            if matches!(event, TreeViewEvent::SelectionChanged { .. }) {
                shell.detail.update(cx, |_, cx| cx.notify());
                cx.notify();
            }
        }));

        Self { look, tree, detail, split, synced_mode, inspector_id, rebuild_tree, _subscriptions: subscriptions }
    }

    fn sync_tree_if_needed(&mut self, cx: &mut Context<Self>) {
        let mode = self.look.mode();
        if self.synced_mode == mode {
            return;
        }
        self.synced_mode = mode;
        let items = (self.rebuild_tree)(&self.look);
        let preserved_id = self
            .tree
            .read(cx)
            .selected_ids()
            .iter()
            .next()
            .and_then(|id| find_inspect_field_selection(self.tree.read(cx).items(), id))
            .map(|field| field.id);
        self.tree.update(cx, |tree, cx| {
            tree.set_items(items, cx);
            if let Some(id) = preserved_id {
                if find_inspect_field_selection(tree.items(), &id).is_some() {
                    tree.select_node_by_id(id, cx);
                }
            }
        });
        self.detail.update(cx, |_, cx| cx.notify());
    }
}

impl Render for ColorInspectorShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_tree_if_needed(cx);

        let chrome = self.look.chrome();
        let body = &self.look.mode_tokens().typography.text.body;

        div()
            .id(self.inspector_id)
            .h_full()
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .border_1()
            .border_color(chrome.border)
            .rounded(px(6.0))
            .bg(chrome.panel_background)
            .p(px(12.0))
            .child(
                div()
                    .text_size(px(body.size))
                    .line_height(px(body.line_height))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(chrome.title_text)
                    .child("Inspector"),
            )
            .child(
                div()
                    .id(format!("{}-body", self.inspector_id))
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .child(self.split.clone()),
            )
    }
}
