use std::sync::Arc;

use gpui::{Context, Entity, EventEmitter, IntoElement, Pixels, Render, SharedString, Subscription, Window};

use super::engine::{SidebarPanelEngine, SidebarPanelEngineEvent, SidebarPanelTemplate};
use super::model::{
    SidebarBuilder, SidebarControlBuilder, SidebarControlModel, SidebarInsetModel, SidebarPanelModel, find_active_id,
    panel_to_nav_nodes,
};
use super::template::{SidebarRenderModel, SidebarTemplate, default_sidebar_template, render_inset_column};
use super::theme::{SidebarCollapsible, SidebarVariant};
use crate::controls::scrollbar::ScrollbarTemplate;
use crate::theme::observe_theme_revision;

/// Semantic events emitted by [`SidebarControl`].
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SidebarEvent {
    /// Fired when sidebar expands, collapses to icon rail, or slides offcanvas.
    OpenChanged { open: bool, collapsible: SidebarCollapsible },

    /// Fired when the mobile drawer or offcanvas sidebar is dismissed.
    Dismissed,

    /// Fired when an item or sub-item is selected.
    Select { id: SharedString },

    /// Fired when an item action button (e.g. "+") is activated.
    Activate { id: SharedString },

    /// Fired when keyboard or pointer focus moves to an item.
    ItemFocused { id: SharedString },

    /// Fired when item hover state changes (used for collapsed popover tracking).
    HoverChanged { id: Option<SharedString> },

    /// Fired when a sub-menu branch expands or collapses.
    SubMenuToggle { id: SharedString, open: bool },

    /// Fired when rail drag resizing begins.
    ResizeStart,

    /// Fired during interactive rail drag-resizing.
    Resized { width: Pixels },

    /// Fired when rail drag resizing completes.
    ResizeEnd { width: Pixels },

    /// Fired when sidebar enabled state changes.
    EnabledChanged { enabled: bool },
}

/// Root layout controller for the Shadcn-style sidebar composition.
///
/// Panel chrome and flush menu rows are rendered by the internal flush presentation
/// engine so visual parity with the former SidebarPanelEngine is preserved.
pub struct SidebarControl {
    model: SidebarControlModel,
    open: bool,
    panel: Entity<SidebarPanelEngine>,
    inset: Option<SidebarInsetModel>,
    template: Arc<dyn SidebarTemplate>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<SidebarEvent> for SidebarControl {}

impl SidebarControl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> SidebarControlBuilder {
        SidebarControlBuilder::new(id)
    }

    pub(crate) fn from_builder(builder: SidebarControlBuilder, cx: &mut Context<Self>) -> Self {
        let mut model = builder.model;
        let open = model.default_open;

        if model.selected_id.is_none()
            && let Some(panel) = &model.sidebar
        {
            model.selected_id = find_active_id(panel);
        }

        let panel = spawn_panel_engine(&model, open, cx);
        let inset = model.inset.clone();

        let mut subscriptions = vec![observe_theme_revision(cx, |_, cx| cx.notify())];
        subscriptions.push(cx.subscribe(&panel, |this, _, event: &SidebarPanelEngineEvent, cx| {
            this.handle_panel_engine_event(event, cx);
        }));

        Self { model, open, panel, inset, template: default_sidebar_template(), _subscriptions: subscriptions }
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn collapsible(&self) -> SidebarCollapsible {
        self.model.collapsible
    }

    pub fn variant(&self) -> SidebarVariant {
        self.model.variant
    }

    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open == open {
            return;
        }
        self.open = open;
        self.sync_panel_collapsed_state(cx);
        cx.emit(SidebarEvent::OpenChanged { open, collapsible: self.model.collapsible });
        if !open && matches!(self.model.collapsible, SidebarCollapsible::Offcanvas) {
            cx.emit(SidebarEvent::Dismissed);
        }
        cx.notify();
    }

    pub fn toggle_open(&mut self, cx: &mut Context<Self>) {
        self.set_open(!self.open, cx);
    }

    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.model.enabled == enabled {
            return;
        }
        self.model.enabled = enabled;
        self.panel.update(cx, |panel, cx| panel.set_enabled(enabled, cx));
        cx.emit(SidebarEvent::EnabledChanged { enabled });
        cx.notify();
    }

    pub fn set_selected_id(&mut self, selected_id: impl Into<SharedString>, cx: &mut Context<Self>) {
        let selected_id = selected_id.into();
        self.model.selected_id = Some(selected_id.clone());
        self.panel.update(cx, |panel, cx| panel.set_selected_id(selected_id, cx));
        cx.notify();
    }

    pub fn set_panel_template(&mut self, template: Arc<dyn SidebarPanelTemplate>, cx: &mut Context<Self>) {
        self.model.panel_template = template.clone();
        self.panel.update(cx, |panel, cx| panel.set_template(template, cx));
    }

    pub fn set_scrollbar_template(&mut self, template: Arc<dyn ScrollbarTemplate>, cx: &mut Context<Self>) {
        self.model.scrollbar_template = template.clone();
        self.panel.update(cx, |panel, cx| panel.set_scrollbar_template(template, cx));
    }

    /// Replace the sidebar panel tree (header / groups / footer) at runtime.
    pub fn set_sidebar(&mut self, sidebar: SidebarBuilder, cx: &mut Context<Self>) {
        let panel = sidebar.build();
        self.apply_panel_model(panel, cx);
    }

    fn apply_panel_model(&mut self, panel: SidebarPanelModel, cx: &mut Context<Self>) {
        let (title, subtitle) = panel
            .header
            .as_ref()
            .map(|header| (header.title.clone(), header.subtitle.clone()))
            .unwrap_or((None, None));
        let (nodes, footer_nodes) = panel_to_nav_nodes(&panel);
        if self.model.selected_id.is_none() {
            self.model.selected_id = find_active_id(&panel);
        }
        let selected_id = self.model.selected_id.clone();
        self.model.sidebar = Some(panel);

        self.panel.update(cx, |engine, cx| {
            match title {
                Some(title) => engine.set_title(title, cx),
                None => engine.clear_title(cx),
            }
            match subtitle {
                Some(subtitle) => engine.set_subtitle(subtitle, cx),
                None => engine.clear_subtitle(cx),
            }
            engine.set_items(nodes, cx);
            engine.set_footer_nodes(footer_nodes, cx);
            if let Some(selected_id) = selected_id {
                engine.set_selected_id(selected_id, cx);
            }
        });
        cx.notify();
    }

    fn sync_panel_collapsed_state(&mut self, cx: &mut Context<Self>) {
        let collapsed = match self.model.collapsible {
            SidebarCollapsible::Icon | SidebarCollapsible::Responsive => !self.open,
            SidebarCollapsible::Offcanvas | SidebarCollapsible::None => false,
        };
        self.panel.update(cx, |panel, cx| {
            if panel.collapsed() != collapsed {
                panel.set_collapsed(collapsed, cx);
            }
        });
    }

    fn handle_panel_engine_event(&mut self, event: &SidebarPanelEngineEvent, cx: &mut Context<Self>) {
        match event {
            SidebarPanelEngineEvent::Activate { node_id, .. } => {
                self.model.selected_id = Some(node_id.clone());
                cx.emit(SidebarEvent::Select { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::BranchExpandedChanged { node_id, expanded } => {
                cx.emit(SidebarEvent::SubMenuToggle { id: node_id.clone(), open: *expanded });
            }
            SidebarPanelEngineEvent::CollapsedChanged { collapsed } => {
                let open = !*collapsed;
                if self.open != open {
                    self.open = open;
                    cx.emit(SidebarEvent::OpenChanged { open, collapsible: self.model.collapsible });
                    cx.notify();
                }
            }
            SidebarPanelEngineEvent::ItemFocused { node_id, .. } => {
                cx.emit(SidebarEvent::ItemFocused { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::ItemHoverChanged { node_id, hovered } => {
                let id = if *hovered { Some(node_id.clone()) } else { None };
                cx.emit(SidebarEvent::HoverChanged { id });
            }
            SidebarPanelEngineEvent::RailSubmenuOpenChanged { node_id } => {
                cx.emit(SidebarEvent::HoverChanged { id: node_id.clone() });
            }
            SidebarPanelEngineEvent::EnabledChanged { enabled } => {
                if self.model.enabled != *enabled {
                    self.model.enabled = *enabled;
                    cx.emit(SidebarEvent::EnabledChanged { enabled: *enabled });
                }
            }
            SidebarPanelEngineEvent::FocusChanged { .. } => {}
        }
    }
}

impl Render for SidebarControl {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let inset = self
            .inset
            .as_ref()
            .map(|inset| render_inset_column(inset.header.as_ref(), inset.content.as_ref(), inset.footer.as_ref()));

        self.template.render(
            SidebarRenderModel {
                id: &self.model.id,
                open: self.open,
                collapsible: self.model.collapsible,
                variant: self.model.variant,
                enabled: self.model.enabled,
                has_inset: self.inset.is_some(),
            },
            self.panel.clone().into_any_element(),
            inset,
        )
    }
}

fn spawn_panel_engine(
    model: &SidebarControlModel,
    open: bool,
    cx: &mut Context<SidebarControl>,
) -> Entity<SidebarPanelEngine> {
    let panel_model = model.sidebar.clone();
    let panel_id = panel_model
        .as_ref()
        .map(|panel| panel.id.clone())
        .unwrap_or_else(|| SharedString::from(format!("{}-panel", model.id)));

    let (title, subtitle) = panel_model
        .as_ref()
        .and_then(|panel| panel.header.as_ref())
        .map(|header| (header.title.clone(), header.subtitle.clone()))
        .unwrap_or((None, None));

    let (nodes, footer_nodes) = panel_model.as_ref().map(panel_to_nav_nodes).unwrap_or_default();

    let collapsed = match model.collapsible {
        SidebarCollapsible::Icon | SidebarCollapsible::Responsive => !open,
        SidebarCollapsible::Offcanvas | SidebarCollapsible::None => false,
    };

    let mut builder = SidebarPanelEngine::new(panel_id)
        .enabled(model.enabled)
        .collapsed(collapsed)
        .template(model.panel_template.clone())
        .scrollbar_template(model.scrollbar_template.clone())
        .items(nodes)
        .footer_nodes(footer_nodes);

    if let Some(title) = title {
        builder = builder.title(title);
    }
    if let Some(subtitle) = subtitle {
        builder = builder.subtitle(subtitle);
    }
    if let Some(selected_id) = &model.selected_id {
        builder = builder.selected_id(selected_id.clone());
    }

    builder.spawn(cx)
}
