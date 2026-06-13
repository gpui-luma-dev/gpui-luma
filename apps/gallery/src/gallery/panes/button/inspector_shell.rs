use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::card::Card;
use gpui_luma::controls::resizable_panels::{
    ResizeHandleSize, ResizablePanelSpec, ResizablePanels, ResizablePanelsOrientation,
};
use gpui_luma::controls::tree_view::{TreeViewControl, TreeViewEvent};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;

use super::inspector_detail::ButtonInspectorDetail;
use super::inspector_tree::{InspectTreeData, build_button_inspect_tree, find_field_selection};

const TREE_PANEL_MIN_PX: f32 = 180.0;
const DETAIL_PANEL_MIN_PX: f32 = 180.0;

pub(in crate::gallery) struct ButtonInspectorShell {
    look: Arc<ShadcnLook>,
    card: Card,
    tree: Entity<TreeViewControl<InspectTreeData>>,
    detail: Entity<ButtonInspectorDetail>,
    split: Entity<ResizablePanels>,
    synced_mode: ThemeMode,
    _subscriptions: Vec<Subscription>,
}

impl ButtonInspectorShell {
    pub fn tree(&self) -> Entity<TreeViewControl<InspectTreeData>> {
        self.tree.clone()
    }

    pub fn detail(&self) -> Entity<ButtonInspectorDetail> {
        self.detail.clone()
    }

    pub fn split(&self) -> Entity<ResizablePanels> {
        self.split.clone()
    }

    pub fn new(look: Arc<ShadcnLook>, tree: Entity<TreeViewControl<InspectTreeData>>, cx: &mut Context<Self>) -> Self {
        let synced_mode = look.mode();
        let panel_bg = look.chrome().panel_background;
        let detail = cx.new(|cx| ButtonInspectorDetail::new(look.clone(), tree.clone(), cx));

        let tree_panel = {
            let tree = tree.clone();
            ResizablePanelSpec::new_render(move || {
                div()
                    .id("button-inspector-tree-host")
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
            ResizablePanelSpec::new_render(move || {
                div()
                    .id("button-inspector-detail-host")
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
            .resizable_panels("button-inspector-split")
            .orientation(ResizablePanelsOrientation::Horizontal)
            .show_border(false)
            .show_handle(true)
            .resize_handle(ResizeHandleSize::Sm)
            .handle_grip(true)
            .panels([tree_panel, detail_panel])
            .spawn(cx);

        let split_for_card = split.clone();
        let card = look
            .card("button-inspector")
            .title("Inspector")
            .full_height(true)
            .body_fill(true)
            .elevated(false)
            .child_render(move |_, _| {
                div()
                    .id("button-inspector-body")
                    .flex_1()
                    .min_h(px(0.0))
                    .overflow_hidden()
                    .child(split_for_card.clone())
                    .into_any_element()
            })
            .spawn(cx);

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(&tree, |shell, _, event: &TreeViewEvent<InspectTreeData>, cx| {
            if matches!(event, TreeViewEvent::SelectionChanged { .. }) {
                shell.detail.update(cx, |_, cx| cx.notify());
                cx.notify();
            }
        }));

        super::super::shared::inspector::sync_inspector_detail_from_tree(&tree, &detail, cx);

        Self { look, card, tree, detail, split, synced_mode, _subscriptions: subscriptions }
    }

    fn sync_tree_if_needed(&mut self, cx: &mut Context<Self>) {
        let mode = self.look.mode();
        if self.synced_mode == mode {
            return;
        }
        self.synced_mode = mode;
        let items = build_button_inspect_tree(&self.look);
        let preserved_id = self
            .tree
            .read(cx)
            .selected_ids()
            .iter()
            .next()
            .and_then(|id| find_field_selection(self.tree.read(cx).items(), id))
            .map(|field| field.id);
        self.tree.update(cx, |tree, cx| {
            tree.set_items(items, cx);
            if let Some(id) = preserved_id {
                if find_field_selection(tree.items(), &id).is_some() {
                    tree.select_node_by_id(id, cx);
                }
            }
        });
        self.detail.update(cx, |_, cx| cx.notify());
    }
}

impl Render for ButtonInspectorShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_tree_if_needed(cx);
        self.card.clone()
    }
}
