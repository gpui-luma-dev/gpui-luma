use std::collections::HashMap;
use std::sync::Arc;

use gpui::{Context, Subscription};
use gpui_luma::controls::command::button::ButtonEvent;
use gpui_luma::controls::command::icon_button::IconButton;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;

const INSPECTOR_PAGE_IDS: &[&str] = &[
    "badge",
    "button",
    "toolbar",
    "toggle",
    "toggle-group",
    "switch",
    "checkbox",
    "accordion",
    "tree-view",
    "radio-button",
    "radio-group",
    "listbox",
    "scrolling-list-view",
    "paging-list-view",
    "slider",
    "scrollbar",
    "textarea",
    "textfield",
    "floating-menu",
    "popup-menu",
    "context-menu",
    "navigation-sidebar",
    "tabs-navigation",
    "progress",
    "resizable-panels",
    "split-view-unified",
    "split-view-inset",
    "split-view-icon-rail",
    "split-view-detached",
    "autocomplete-textfield",
    "combobox",
    "search-selector",
    "popup-selector",
    "selection-panel",
];

#[derive(Clone)]
pub(in crate::gallery) struct InspectorToggleEntry {
    pub(in crate::gallery) toggle: IconButton,
    pub(in crate::gallery) visible: bool,
}

#[derive(Clone)]
pub(in crate::gallery) struct InspectorToggleRegistry {
    entries: HashMap<&'static str, InspectorToggleEntry>,
}

impl InspectorToggleRegistry {
    pub(in crate::gallery) fn new(look: Arc<ShadcnLook>, cx: &mut Context<GalleryApp>) -> Self {
        let mut entries = HashMap::new();
        for &page_id in INSPECTOR_PAGE_IDS {
            let toggle =
                look.ghost_icon_button(format!("{page_id}-inspector-toggle"), LucideIcon::PanelRight).spawn(cx);
            entries.insert(page_id, InspectorToggleEntry { toggle, visible: true });
        }
        Self { entries }
    }

    pub(in crate::gallery) fn get(&self, page_id: &'static str) -> &InspectorToggleEntry {
        self.entries
            .get(page_id)
            .unwrap_or_else(|| panic!("missing inspector toggle for page id: {page_id}"))
    }

    pub(in crate::gallery) fn toggle(&mut self, page_id: &'static str) {
        if let Some(entry) = self.entries.get_mut(page_id) {
            entry.visible = !entry.visible;
        }
    }

    pub(in crate::gallery) fn subscribe(
        registry: &Self,
        cx: &mut Context<GalleryApp>,
        subscriptions: &mut Vec<Subscription>,
    ) {
        for (page_id, entry) in &registry.entries {
            let page_id = *page_id;
            let toggle = entry.toggle.clone();
            subscriptions.push(cx.subscribe(&toggle, move |app, _, _: &ButtonEvent, cx| {
                app.panes.inspector_toggles.toggle(page_id);
                cx.notify();
            }));
        }
    }
}
