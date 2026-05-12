use std::sync::Arc;

use gpui::{
    AnyElement, App, ClickEvent, Context, Entity, FontWeight, IntoElement, Render, SharedString, Subscription, Window,
    div, prelude::*, px,
};
use gpui_luma::controls::selection_panel::{
    SelectionPanelAppearance, SelectionPanelClickHandler, SelectionPanelHoverHandler, SelectionPanelItem,
    SelectionPanelRenderModel, SelectionPanelTemplate, default_selection_panel_appearance,
    default_selection_panel_template, render_selection_panel,
};
use gpui_luma::controls::state::ControlFocusState;
use gpui_luma::theme::ControlSize;
use lucide_icons::Icon as LucideIcon;

use crate::gallery::control::GalleryApp;
use crate::gallery::panes::shared::{gallery_pane_with_usage_descriptions, notify_entity};
use crate::gallery::theme::GalleryThemePack;

#[derive(Clone)]
pub(in crate::gallery) struct SelectionPanelPane {
    template_preview: Entity<SelectionPanelTemplatePreview>,
    interactive_panel: Entity<InteractiveSelectionPanel>,
}

impl SelectionPanelPane {
    pub(in crate::gallery) fn new(cx: &mut Context<GalleryApp>, theme: &GalleryThemePack) -> Self {
        Self {
            template_preview: cx.new(|_| SelectionPanelTemplatePreview::new(theme.clone())),
            interactive_panel: cx.new(|_| InteractiveSelectionPanel::new(theme.clone())),
        }
    }

    pub(in crate::gallery) fn subscribe(&self, _cx: &mut Context<GalleryApp>, _subscriptions: &mut Vec<Subscription>) {}

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();

        gallery_pane_with_usage_descriptions(
            "Selection Panel",
            Some("Template-rendered preview and interactive panel for the new selection_panel control."),
            &["Selector"],
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(self.template_preview.clone())
                .child(
                    div().flex().flex_col().items_center().gap(px(8.0)).child(self.interactive_panel.clone()).child(
                        div()
                            .text_size(px(11.0))
                            .line_height(px(15.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.muted_text)
                            .child("Interactive: click any row to toggle selected state"),
                    ),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self, cx: &mut Context<GalleryApp>) {
        notify_entity(&self.template_preview, cx);
        notify_entity(&self.interactive_panel, cx);
    }
}

#[derive(Clone)]
struct SelectionPanelTemplatePreview {
    theme: GalleryThemePack,
    template: Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
}

impl SelectionPanelTemplatePreview {
    fn new(theme: GalleryThemePack) -> Self {
        Self { theme, template: default_selection_panel_template() }
    }
}

impl Render for SelectionPanelTemplatePreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.theme.chrome();
        let appearance = default_selection_panel_appearance(&self.theme.tokens(), ControlSize::Md);

        let standard_items = vec![
            SelectionPanelItem::new("draft-note").label("Draft note").icon(LucideIcon::FilePenLine),
            SelectionPanelItem::new("pin-sidebar").label("Pin in sidebar").icon(LucideIcon::Pin),
            SelectionPanelItem::new("mark-complete").label("Mark complete").icon(LucideIcon::CircleCheck),
        ];

        let active_items = vec![
            SelectionPanelItem::new("review").label("Review pending").icon(LucideIcon::ClipboardCheck),
            SelectionPanelItem::new("escalate").label("Escalate").icon(LucideIcon::TriangleAlert),
            SelectionPanelItem::new("defer").label("Defer 24h").icon(LucideIcon::Clock3),
        ];

        let disabled_items = vec![
            SelectionPanelItem::new("diagnostics").label("Open diagnostics").icon(LucideIcon::ScanSearch),
            SelectionPanelItem::new("sync").label("Sync records").icon(LucideIcon::RefreshCcw).enabled(false),
            SelectionPanelItem::new("export").label("Export bundle").icon(LucideIcon::PackageOpen),
        ];

        div()
            .flex()
            .flex_wrap()
            .items_start()
            .justify_center()
            .gap(px(16.0))
            .child(render_template_sample(
                "Standard",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-standard",
                &standard_items,
                None,
                None,
                cx,
            ))
            .child(render_template_sample(
                "Hover / active item",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-active",
                &active_items,
                Some(1),
                Some(1),
                cx,
            ))
            .child(render_template_sample(
                "Disabled item",
                &self.template,
                &appearance,
                chrome.muted_text,
                "selection-panel-disabled",
                &disabled_items,
                None,
                None,
                cx,
            ))
    }
}

struct InteractiveSelectionPanel {
    theme: GalleryThemePack,
    template: Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
    items: Vec<SelectionPanelItem>,
    selected_index: Option<usize>,
    active_index: Option<usize>,
}

impl InteractiveSelectionPanel {
    fn new(theme: GalleryThemePack) -> Self {
        Self {
            theme,
            template: default_selection_panel_template(),
            items: vec![
                SelectionPanelItem::new("inspect").label("Inspect summary").icon(LucideIcon::ListChecks),
                SelectionPanelItem::new("publish").label("Publish update").icon(LucideIcon::Send),
                SelectionPanelItem::new("stash").label("Stash snapshot").icon(LucideIcon::ArchiveRestore),
            ],
            selected_index: Some(1),
            active_index: Some(1),
        }
    }

    fn handle_item_hover(&mut self, index: usize, hovered: bool, cx: &mut Context<Self>) {
        if hovered && self.active_index != Some(index) {
            self.active_index = Some(index);
            cx.notify();
        }
    }

    fn handle_item_click(&mut self, index: usize, event: &ClickEvent, cx: &mut Context<Self>) {
        if event.is_keyboard() {
            return;
        }

        self.selected_index = if self.selected_index == Some(index) {
            None
        } else {
            Some(index)
        };
        self.active_index = Some(index);
        cx.notify();
    }
}

impl Render for InteractiveSelectionPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let appearance = default_selection_panel_appearance(&self.theme.tokens(), ControlSize::Md);
        let panel_id = SharedString::from("selection-panel-interactive-menu");
        let control_id = SharedString::from("selection-panel-interactive-selector");
        let visible_indices = (0..self.items.len()).collect::<Vec<_>>();

        let item_hovers = (0..self.items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, hovered, _window, cx| {
                    this.handle_item_hover(index, *hovered, cx);
                })) as SelectionPanelHoverHandler
            })
            .collect::<Vec<_>>();

        let item_clicks = (0..self.items.len())
            .map(|index| {
                Box::new(cx.listener(move |this, event, _window, cx| {
                    this.handle_item_click(index, event, cx);
                })) as SelectionPanelClickHandler
            })
            .collect::<Vec<_>>();

        render_selection_panel(
            self.template.clone(),
            &SelectionPanelRenderModel {
                panel_id: &panel_id,
                control_id: &control_id,
                items: &self.items,
                visible_indices: &visible_indices,
                selected_source_index: self.selected_index,
                active_visible_index: self.active_index,
                hovered_visible_index: self.active_index,
                pressed_visible_index: None,
                open: true,
                enabled: true,
                focus: ControlFocusState::default(),
                presenter: None,
                appearance,
                show_selection_marker: true,
            },
            item_hovers,
            item_clicks,
            cx,
        )
    }
}

fn render_template_sample(
    label: &'static str,
    template: &Arc<dyn SelectionPanelTemplate<SelectionPanelItem>>,
    appearance: &SelectionPanelAppearance,
    label_color: gpui::Hsla,
    sample_id: &'static str,
    items: &[SelectionPanelItem],
    selected_index: Option<usize>,
    active_index: Option<usize>,
    cx: &mut App,
) -> AnyElement {
    let panel_id = SharedString::from(sample_id);
    let control_id = SharedString::from(format!("{sample_id}-control"));
    let visible_indices = (0..items.len()).collect::<Vec<_>>();

    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.0))
        .child(render_selection_panel(
            template.clone(),
            &SelectionPanelRenderModel {
                panel_id: &panel_id,
                control_id: &control_id,
                items,
                visible_indices: &visible_indices,
                selected_source_index: selected_index,
                active_visible_index: active_index,
                hovered_visible_index: active_index,
                pressed_visible_index: None,
                open: true,
                enabled: true,
                focus: ControlFocusState::default(),
                presenter: None,
                appearance: appearance.clone(),
                show_selection_marker: true,
            },
            noop_hovers(items.len()),
            noop_clicks(items.len()),
            cx,
        ))
        .child(
            div()
                .text_size(px(11.0))
                .line_height(px(15.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .into_any_element()
}

fn noop_hovers(count: usize) -> Vec<SelectionPanelHoverHandler> {
    (0..count)
        .map(|_| Box::new(|_: &bool, _: &mut Window, _: &mut App| {}) as SelectionPanelHoverHandler)
        .collect()
}

fn noop_clicks(count: usize) -> Vec<SelectionPanelClickHandler> {
    (0..count)
        .map(|_| Box::new(|_: &ClickEvent, _: &mut Window, _: &mut App| {}) as SelectionPanelClickHandler)
        .collect()
}
