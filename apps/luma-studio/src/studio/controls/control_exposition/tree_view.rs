//! Tree view control exposition — gallery-aligned virtualized file tree with scroll shell.

use std::sync::Arc;

use gpui::{Context, Entity, IntoElement, Render, SharedString, Subscription, Window, div, prelude::*, px};
use gpui_luma::controls::scroll_container::ScrollContainer;
use gpui_luma::controls::scrollbar::ScrollbarEvent;
use gpui_luma::controls::tree_view::{TreeNode, TreeView, TreeViewEvent, TreeViewSelectionMode};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::collection_theme_inspectors::TreeViewThemeInspector;
use super::event_stream::ControlEventStream;
use super::exposition_inspector::{spawn_viewport_inspector, sync_viewport_inspector, ViewportInspectorPane};
use super::inspector_split::InspectorSplitShell;
use super::model::{ControlExpositionLayout, EventReferenceSpec, PublicInterfaceSpec};
use super::public_interface::render_exposition_doc_sections;
use super::template::render_control_exposition_card;
use super::tree_view_inspector_adapter::{TreeViewInspectorAdapter, TREE_VIEW_INSPECTOR_SPEC};

const TREE_DEPTH: usize = 5;

const EVENT_SPECS: &[EventReferenceSpec] = &[
    EventReferenceSpec {
        event: "TreeViewEvent::NodeExpanded { node_id, data }",
        trigger: "User expands a branch row",
        notes: "Carries the node id and typed data payload.",
    },
    EventReferenceSpec {
        event: "TreeViewEvent::NodeCollapsed { node_id, data }",
        trigger: "User collapses a branch row",
        notes: "Carries the node id and typed data payload.",
    },
    EventReferenceSpec {
        event: "TreeViewEvent::SelectionChanged { selected_ids }",
        trigger: "Selection set changes",
        notes: "Depends on TreeViewSelectionMode policy.",
    },
    EventReferenceSpec {
        event: "TreeViewEvent::ActiveNodeChanged { node_id }",
        trigger: "Keyboard or pointer moves active node",
        notes: "Distinct from selection in multi-select modes.",
    },
    EventReferenceSpec {
        event: "TreeViewEvent::FocusChanged { focused }",
        trigger: "Focus enters or leaves the tree",
        notes: "Useful for form-level focus coordination.",
    },
];

const PUBLIC_INTERFACE_SPECS: &[PublicInterfaceSpec] = &[
    PublicInterfaceSpec {
        symbol: "TreeView<T>",
        surface: "Type",
        notes: "Entity<TreeViewControl<T>> — virtualized hierarchical list.",
    },
    PublicInterfaceSpec {
        symbol: "TreeViewEvent<T>",
        surface: "Event",
        notes: "Expand/collapse, selection, active node, and focus lifecycle.",
    },
    PublicInterfaceSpec { symbol: "look.tree_view(id)", surface: "Look", notes: "ShadcnLookControlExt factory." },
    PublicInterfaceSpec {
        symbol: "TreeViewBuilder::items / selection_mode",
        surface: "Builder",
        notes: "TreeNode forest and single or multi selection.",
    },
    PublicInterfaceSpec {
        symbol: "TreeNode::new / children / expanded / icon",
        surface: "Model",
        notes: "Branch and leaf node construction helpers.",
    },
    PublicInterfaceSpec {
        symbol: "TreeViewBuilder::spawn(cx)",
        surface: "Builder",
        notes: "Materialize entity; subscribe for TreeViewEvent.",
    },
];

pub struct TreeViewControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    left_pane: Entity<TreeViewExpositionLeftPane>,
    theme_inspector: Entity<TreeViewThemeInspector>,
    inspector_split: Entity<InspectorSplitShell>,
    _subscriptions: Vec<Subscription>,
}

struct TreeViewExpositionLeftPane {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    shell: Entity<TreeViewScrollShell>,
    event_stream: Entity<ControlEventStream>,
}

impl TreeViewExpositionLeftPane {
    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.shell.update(cx, |shell, cx| shell.sync_look(look.clone(), cx));
        self.event_stream.update(cx, |stream, cx| stream.sync_look(look, cx));
        cx.notify();
    }
}

impl Render for TreeViewExpositionLeftPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let preview =
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(12.0))
                    .child(div().text_size(px(12.0)).text_color(chrome.muted_text).child(format!(
                        "Mock tree: {TREE_DEPTH} levels deep — click branch rows to expand or collapse"
                    )))
                    .child(
                        div()
                            .w(px(360.0))
                            .h(px(480.0))
                            .flex()
                            .flex_col()
                            .min_h(px(0.0))
                            .overflow_hidden()
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(chrome.border)
                            .bg(chrome.panel_background)
                            .child(self.shell.clone()),
                    )
                    .child(self.event_stream.clone());

            div()
                .id("controls-doc-tree-view-left-pane")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .overflow_y_scroll()
                .child(render_control_exposition_card(
                    &self.look,
                    self.entry,
                    preview.into_any_element(),
                    Some(render_exposition_doc_sections(&self.look, EVENT_SPECS, PUBLIC_INTERFACE_SPECS)),
                    ControlExpositionLayout::BORDERLESS,
                ))
        })
    }
}

impl TreeViewControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("tree-view").expect("tree-view catalog entry");

        let tree = look
            .tree_view("controls-doc-tree-view")
            .selection_mode(TreeViewSelectionMode::Single)
            .items(mock_file_tree())
            .spawn(cx);

        let shell = cx.new(|cx| TreeViewScrollShell::new(look.clone(), tree.clone(), cx));

        let event_stream = cx.new(|cx| {
            ControlEventStream::new(
                cx,
                look.clone(),
                "controls-tree-view-event-log",
                "Expand branches and select leaves; TreeViewEvent variants appear below.",
            )
        });

        let left_pane = cx.new(|_| TreeViewExpositionLeftPane {
            look: look.clone(),
            entry,
            shell: shell.clone(),
            event_stream: event_stream.clone(),
        });
        let ViewportInspectorPane { theme_inspector, inspector_split } = spawn_viewport_inspector(
            cx,
            look.clone(),
            "controls-doc-tree-view-pane",
            {
                let left_pane = left_pane.clone();
                move || left_pane.clone().into_any_element()
            },
            &TREE_VIEW_INSPECTOR_SPEC,
            TreeViewInspectorAdapter::shared(),
        );

        let subscription = cx.subscribe(&tree, {
            let event_stream = event_stream.clone();
            move |_, _, event: &TreeViewEvent<SharedString>, cx| {
                if let Some(line) = format_tree_view_event(event) {
                    event_stream.update(cx, |stream, cx| stream.append_line(&line, cx));
                }
            }
        });

        Self { look, entry, left_pane, theme_inspector, inspector_split, _subscriptions: vec![subscription] }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn fills_viewport(&self) -> bool {
        true
    }

    pub fn request_layout_refresh(&mut self, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.request_layout_refresh(cx));
    }

    pub fn set_viewport_size(&mut self, size: gpui::Size<gpui::Pixels>, cx: &mut Context<Self>) {
        self.inspector_split.update(cx, |split, cx| split.set_viewport_size(size, cx));
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look.clone();
        self.left_pane.update(cx, |pane, cx| pane.sync_look(look.clone(), cx));
        sync_viewport_inspector(look, &self.theme_inspector, &self.inspector_split, cx);
        cx.notify();
    }
}

impl Render for TreeViewControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            div()
                .id("controls-doc-tree-view-exposition")
                .size_full()
                .min_h(px(0.0))
                .min_w(px(0.0))
                .child(self.inspector_split.clone())
        })
    }
}

struct TreeViewScrollShell {
    look: Arc<ShadcnLook>,
    scroll: ScrollContainer,
    tree: TreeView<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl TreeViewScrollShell {
    fn new(look: Arc<ShadcnLook>, tree: TreeView<SharedString>, cx: &mut Context<Self>) -> Self {
        let scroll = ScrollContainer::new("controls-doc-tree-scroll", look.scrollbar_template(), cx);
        let scrollbar = scroll.scrollbar();
        let subscriptions = vec![cx.subscribe(&scrollbar, |this, _, event: &ScrollbarEvent, cx| {
            if let ScrollbarEvent::Change { value } = event {
                this.scroll.set_vertical_offset(*value, cx);
            }
        })];

        Self { look, scroll, tree, _subscriptions: subscriptions }
    }

    fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.tree.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for TreeViewScrollShell {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scroll.sync_scrollbar(cx);

        self.scroll.render(
            div().id("controls-doc-tree-scroll-content").size_full().child(self.tree.clone()).into_any_element(),
        )
    }
}

fn format_tree_view_event(event: &TreeViewEvent<SharedString>) -> Option<String> {
    match event {
        TreeViewEvent::NodeExpanded { node_id, .. } => {
            Some(format!("TreeViewEvent::NodeExpanded {{ node_id: \"{node_id}\" }}"))
        }
        TreeViewEvent::NodeCollapsed { node_id, .. } => {
            Some(format!("TreeViewEvent::NodeCollapsed {{ node_id: \"{node_id}\" }}"))
        }
        TreeViewEvent::SelectionChanged { selected_ids } => {
            let ids: Vec<_> = selected_ids.iter().map(|id| id.as_ref()).collect();
            Some(format!("TreeViewEvent::SelectionChanged {{ selected_ids: {ids:?} }}"))
        }
        TreeViewEvent::ActiveNodeChanged { node_id } => Some(match node_id {
            Some(id) => format!("TreeViewEvent::ActiveNodeChanged {{ node_id: \"{id}\" }}"),
            None => "TreeViewEvent::ActiveNodeChanged { node_id: None }".to_string(),
        }),
        TreeViewEvent::FocusChanged { focused } => {
            Some(format!("TreeViewEvent::FocusChanged {{ focused: {focused} }}"))
        }
        _ => None,
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
