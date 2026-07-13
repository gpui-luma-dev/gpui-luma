use std::sync::Arc;

use gpui::{AnyElement, Context, Entity, IntoElement, SharedString, Subscription, div, prelude::*, px};
use gpui_luma::controls::tree_view::{TreeNode, TreeViewEvent, TreeViewSelectionMode};
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

use super::inspector_tree::build_tree_view_inspect_tree;
use super::scroll_shell::TreeViewScrollShell;
use super::super::shared::inspector::{ColorInspectorShell, spawn_color_inspector_tree};
use super::super::shared::{gallery_pane_with_inspector_description, notify_entity, InspectorToggleRegistry};

const TREE_VIEW_DESCRIPTION: &str = concat!(
    "Virtualized tree view for hierarchical data such as file explorers. ",
    "Click a branch row to expand or collapse; click a leaf to select. ",
    "Arrow keys move focus; Left and Right collapse or expand the focused branch."
);

/// Number of nesting levels in the gallery mock tree (depth 0 .. TREE_DEPTH - 1).
const TREE_DEPTH: usize = 5;

#[derive(Clone)]
pub(in crate::gallery) struct TreeViewPane {
    shell: Entity<TreeViewScrollShell>,
    last_event: SharedString,
    inspector: Entity<ColorInspectorShell>,
}

impl TreeViewPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, look: Arc<ShadcnLook>) -> Self {
        let tree =
            spawn_color_inspector_tree("tree-view-inspector-tree", look.clone(), build_tree_view_inspect_tree, cx);
        let inspector = cx.new(|cx| {
            ColorInspectorShell::new(
                look.clone(),
                tree,
                "tree-view-inspector",
                "tree-view-inspector-split",
                "tree-view-inspector-detail",
                build_tree_view_inspect_tree,
                cx,
            )
        });

        let tree = look
            .tree_view("gallery-tree-view")
            .selection_mode(TreeViewSelectionMode::Single)
            .items(mock_file_tree())
            .spawn(cx);

        let shell = cx.new(|cx| TreeViewScrollShell::new(look, tree, cx));

        Self { shell, last_event: "None".into(), inspector }
    }

    pub(in crate::gallery) fn subscribe(&self, cx: &mut Context<GalleryApp>, subscriptions: &mut Vec<Subscription>) {
        subscriptions.push(cx.subscribe(
            &self.shell.read(cx).tree(),
            |app, _, event: &TreeViewEvent<SharedString>, cx| {
                app.panes.tree_view.handle_event(event, cx);
            },
        ));
    }

    pub(in crate::gallery) fn render(&self, look: &ShadcnLook, toggles: &InspectorToggleRegistry) -> AnyElement {
        let chrome = look.chrome();

        gallery_pane_with_inspector_description(
            "tree-view",
            "Tree View",
            Some(TREE_VIEW_DESCRIPTION),
            div()
                .w(px(360.0))
                .h(px(480.0))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div().text_size(px(12.0)).text_color(chrome.muted_text).child(format!(
                        "Mock tree: {TREE_DEPTH} levels deep — click branch rows to expand or collapse"
                    )),
                )
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .w_full()
                        .overflow_hidden()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(chrome.border)
                        .bg(chrome.panel_background)
                        .child(self.shell.clone()),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(chrome.muted_text)
                        .child(format!("Last event: {}", self.last_event)),
                )
                .into_any_element(),
            self.inspector.clone(),
            toggles,
            look,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.shell, cx);
        notify_entity(&self.shell.read(cx).tree(), cx);
        notify_entity(&self.inspector, cx);
        notify_entity(&self.inspector.read(cx).tree(), cx);
        notify_entity(&self.inspector.read(cx).detail(), cx);
        notify_entity(&self.inspector.read(cx).split(), cx);
    }

    fn handle_event(&mut self, event: &TreeViewEvent<SharedString>, cx: &mut Context<GalleryApp>) {
        self.last_event = match event {
            TreeViewEvent::NodeExpanded { node_id, .. } => format!("expanded {node_id}").into(),
            TreeViewEvent::NodeCollapsed { node_id, .. } => format!("collapsed {node_id}").into(),
            TreeViewEvent::SelectionChanged { selected_ids } => {
                let ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
                format!("selection {:?}", ids).into()
            }
        };
        cx.notify();
    }
}

fn mock_file_tree() -> Vec<TreeNode<SharedString>> {
    (0..4)
        .map(|root| {
            let root_id: SharedString = format!("root-{root}").into();
            TreeNode::new(root_id.clone(), format!("Workspace {root}"), root_id)
                .icon(LucideIcon::Folder)
                .expanded(root == 0)
                .children(vec![mock_branch(&format!("root-{root}"), 1)])
        })
        .collect()
}

fn mock_branch(prefix: &str, depth: usize) -> TreeNode<SharedString> {
    if depth >= TREE_DEPTH - 1 {
        let id: SharedString = format!("{prefix}-file").into();
        return TreeNode::new(id.clone(), format!("{prefix}.txt"), id).icon(LucideIcon::File);
    }

    let id: SharedString = format!("{prefix}-folder").into();
    let child_count = if depth == 1 { 4 } else { 3 };
    let children: Vec<_> = (0..child_count).map(|index| mock_branch(&format!("{prefix}-{index}"), depth + 1)).collect();

    TreeNode::new(id.clone(), format!("Folder {prefix}"), id)
        .icon(LucideIcon::Folder)
        .children(children)
}
