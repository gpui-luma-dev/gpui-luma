use std::rc::Rc;
use std::sync::Arc;

use gpui::{AnyElement, AppContext, Entity, IntoElement, SharedString};
use lucide_svg_static::Icon as LucideIcon;

use super::control::SidebarControl;
use super::engine::{NavNode, SidebarPanelTemplate, default_sidebar_panel_template, modified_sidebar_panel_template};
use super::template::{SidebarTemplate, default_sidebar_template};
use super::theme::{SidebarCollapsible, SidebarVariant};
use crate::controls::scroll_container::{ScrollbarAutoHideActivate, ScrollbarPlacement, ScrollbarVisibility};
use crate::controls::icon::DisclosureIcons;
use crate::controls::scrollbar::{ScrollbarTemplate, default_scrollbar_template};

pub type SidebarPaneRender = Rc<dyn Fn() -> AnyElement>;

#[derive(Clone)]
pub struct SidebarMenuItemModel {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) icon: Option<LucideIcon>,
    pub(crate) badge: Option<SharedString>,
    pub(crate) action_label: Option<SharedString>,
    pub(crate) active: bool,
    pub(crate) disabled: bool,
    pub(crate) expanded: Option<bool>,
    pub(crate) children: Vec<SidebarMenuItemModel>,
}

#[derive(Clone)]
pub struct SidebarMenuModel {
    #[allow(dead_code)]
    pub(crate) id: SharedString,
    pub(crate) items: Vec<SidebarMenuItemModel>,
}

#[derive(Clone)]
pub struct SidebarGroupModel {
    pub(crate) label: Option<SharedString>,
    pub(crate) action_label: Option<SharedString>,
    pub(crate) menu: Option<SidebarMenuModel>,
}

#[derive(Clone)]
pub struct SidebarHeaderModel {
    pub(crate) title: Option<SharedString>,
    pub(crate) subtitle: Option<SharedString>,
}

#[derive(Clone)]
pub struct SidebarContentModel {
    pub(crate) groups: Vec<SidebarGroupModel>,
}

#[derive(Clone)]
pub struct SidebarFooterModel {
    pub(crate) items: Vec<SidebarMenuItemModel>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SidebarRailModel {
    pub(crate) _enabled: bool,
}

#[derive(Clone)]
pub struct SidebarPanelModel {
    pub(crate) id: SharedString,
    pub(crate) header: Option<SidebarHeaderModel>,
    pub(crate) content: Option<SidebarContentModel>,
    pub(crate) footer: Option<SidebarFooterModel>,
    pub(crate) rail: Option<SidebarRailModel>,
    pub(crate) disclosure_icons: DisclosureIcons,
}

#[derive(Clone)]
pub struct SidebarInsetModel {
    pub(crate) header: Option<SidebarPaneRender>,
    pub(crate) content: Option<SidebarPaneRender>,
    pub(crate) footer: Option<SidebarPaneRender>,
}

#[derive(Clone)]
pub struct SidebarControlModel {
    pub(crate) id: SharedString,
    pub(crate) default_open: bool,
    pub(crate) collapsible: SidebarCollapsible,
    pub(crate) variant: SidebarVariant,
    pub(crate) enabled: bool,
    pub(crate) animated: bool,
    pub(crate) selected_id: Option<SharedString>,
    pub(crate) sidebar: Option<SidebarPanelModel>,
    pub(crate) inset: Option<SidebarInsetModel>,
    pub(crate) template: Arc<dyn SidebarTemplate>,
    pub(crate) panel_template: Arc<dyn SidebarPanelTemplate>,
    pub(crate) scrollbar_template: Arc<dyn ScrollbarTemplate>,
    pub(crate) scrollbar_placement: ScrollbarPlacement,
    pub(crate) scrollbar_visibility: ScrollbarVisibility,
    pub(crate) scrollbar_auto_hide_activate: ScrollbarAutoHideActivate,
}

pub struct SidebarMenuItemBuilder {
    model: SidebarMenuItemModel,
}

impl SidebarMenuItemBuilder {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        let id = id.into();
        Self {
            model: SidebarMenuItemModel {
                id: id.clone(),
                label: label.into(),
                icon: None,
                badge: None,
                action_label: None,
                active: false,
                disabled: false,
                expanded: None,
                children: Vec::new(),
            },
        }
    }

    pub fn icon(mut self, icon: LucideIcon) -> Self {
        self.model.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.model.badge = Some(badge.into());
        self
    }

    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.model.action_label = Some(label.into());
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.model.active = active;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.model.disabled = disabled;
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.model.expanded = Some(expanded);
        self
    }

    /// Attach a nested submenu branch (Shadcn `SidebarMenuSub` ergonomics).
    #[allow(clippy::should_implement_trait)]
    pub fn sub(mut self, sub: SidebarMenuSubBuilder) -> Self {
        self.model.children = sub.into_items();
        self
    }

    pub(crate) fn build(self) -> SidebarMenuItemModel {
        self.model
    }
}

pub struct SidebarMenuSubBuilder {
    items: Vec<SidebarMenuItemModel>,
}

impl SidebarMenuSubBuilder {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.items.push(item.build());
        self
    }

    pub(crate) fn into_items(self) -> Vec<SidebarMenuItemModel> {
        self.items
    }
}

impl Default for SidebarMenuSubBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarMenuBuilder {
    model: SidebarMenuModel,
}

impl SidebarMenuBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self { model: SidebarMenuModel { id: id.into(), items: Vec::new() } }
    }

    pub fn item(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.model.items.push(item.build());
        self
    }

    pub(crate) fn build(self) -> SidebarMenuModel {
        self.model
    }
}

pub struct SidebarGroupBuilder {
    model: SidebarGroupModel,
}

impl SidebarGroupBuilder {
    pub fn new() -> Self {
        Self { model: SidebarGroupModel { label: None, action_label: None, menu: None } }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.model.label = Some(label.into());
        self
    }

    pub fn action(mut self, label: impl Into<SharedString>) -> Self {
        self.model.action_label = Some(label.into());
        self
    }

    pub fn menu(mut self, menu: SidebarMenuBuilder) -> Self {
        self.model.menu = Some(menu.build());
        self
    }

    pub(crate) fn build(self) -> SidebarGroupModel {
        self.model
    }
}

impl Default for SidebarGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarHeaderBuilder {
    model: SidebarHeaderModel,
}

impl SidebarHeaderBuilder {
    pub fn new() -> Self {
        Self { model: SidebarHeaderModel { title: None, subtitle: None } }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.model.title = Some(title.into());
        self
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.model.subtitle = Some(subtitle.into());
        self
    }

    pub(crate) fn build(self) -> SidebarHeaderModel {
        self.model
    }
}

impl Default for SidebarHeaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarContentBuilder {
    model: SidebarContentModel,
}

impl SidebarContentBuilder {
    pub fn new() -> Self {
        Self { model: SidebarContentModel { groups: Vec::new() } }
    }

    pub fn group(mut self, group: SidebarGroupBuilder) -> Self {
        self.model.groups.push(group.build());
        self
    }

    pub(crate) fn build(self) -> SidebarContentModel {
        self.model
    }
}

impl Default for SidebarContentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarFooterBuilder {
    model: SidebarFooterModel,
}

impl SidebarFooterBuilder {
    pub fn new() -> Self {
        Self { model: SidebarFooterModel { items: Vec::new() } }
    }

    pub fn child(mut self, item: SidebarMenuItemBuilder) -> Self {
        self.model.items.push(item.build());
        self
    }

    pub(crate) fn build(self) -> SidebarFooterModel {
        self.model
    }
}

impl Default for SidebarFooterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarRailBuilder {
    model: SidebarRailModel,
}

impl SidebarRailBuilder {
    pub fn new() -> Self {
        Self { model: SidebarRailModel { _enabled: true } }
    }

    pub(crate) fn build(self) -> SidebarRailModel {
        self.model
    }
}

impl Default for SidebarRailBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarBuilder {
    model: SidebarPanelModel,
}

impl SidebarBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SidebarPanelModel {
                id: id.into(),
                header: None,
                content: None,
                footer: None,
                rail: None,
                disclosure_icons: DisclosureIcons::default(),
            },
        }
    }

    pub fn header(mut self, header: SidebarHeaderBuilder) -> Self {
        self.model.header = Some(header.build());
        self
    }

    pub fn content(mut self, content: SidebarContentBuilder) -> Self {
        self.model.content = Some(content.build());
        self
    }

    pub fn footer(mut self, footer: SidebarFooterBuilder) -> Self {
        self.model.footer = Some(footer.build());
        self
    }

    pub fn rail(mut self, rail: SidebarRailBuilder) -> Self {
        self.model.rail = Some(rail.build());
        self
    }

    /// Sets the expanded and collapsed disclosure icons for nested menu rows.
    pub fn disclosure_icons(mut self, icons: DisclosureIcons) -> Self {
        self.model.disclosure_icons = icons;
        self
    }

    pub fn build(self) -> SidebarPanelModel {
        self.model
    }
}

pub struct SidebarInsetBuilder {
    model: SidebarInsetModel,
}

impl SidebarInsetBuilder {
    pub fn new() -> Self {
        Self { model: SidebarInsetModel { header: None, content: None, footer: None } }
    }

    pub fn header<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.header = Some(pane_render(render));
        self
    }

    pub fn content<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.content = Some(pane_render(render));
        self
    }

    pub fn footer<E, F>(mut self, render: F) -> Self
    where
        E: IntoElement,
        F: Fn() -> E + 'static,
    {
        self.model.footer = Some(pane_render(render));
        self
    }

    pub(crate) fn build(self) -> SidebarInsetModel {
        self.model
    }
}

impl Default for SidebarInsetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SidebarControlBuilder {
    pub(crate) model: SidebarControlModel,
}

impl SidebarControlBuilder {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            model: SidebarControlModel {
                id: id.into(),
                default_open: true,
                collapsible: SidebarCollapsible::Icon,
                variant: SidebarVariant::Sidebar,
                enabled: true,
                animated: true,
                selected_id: None,
                sidebar: None,
                inset: None,
                template: default_sidebar_template(),
                panel_template: default_sidebar_panel_template(),
                scrollbar_template: default_scrollbar_template(),
                scrollbar_placement: ScrollbarPlacement::Inset,
                scrollbar_visibility: ScrollbarVisibility::AlwaysVisible,
                scrollbar_auto_hide_activate: ScrollbarAutoHideActivate::HoverOrMove,
            },
        }
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.model.default_open = open;
        self
    }

    pub fn collapsible(mut self, collapsible: SidebarCollapsible) -> Self {
        self.model.collapsible = collapsible;
        self
    }

    pub fn variant(mut self, variant: SidebarVariant) -> Self {
        self.model.variant = variant;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.model.enabled = enabled;
        self
    }

    pub fn animated(mut self, animated: bool) -> Self {
        self.model.animated = animated;
        self
    }

    pub fn selected_id(mut self, selected_id: impl Into<SharedString>) -> Self {
        self.model.selected_id = Some(selected_id.into());
        self
    }

    pub fn sidebar(mut self, sidebar: SidebarBuilder) -> Self {
        self.model.sidebar = Some(sidebar.build());
        self
    }

    pub fn inset(mut self, inset: SidebarInsetBuilder) -> Self {
        self.model.inset = Some(inset.build());
        self
    }

    pub fn template(mut self, template: Arc<dyn SidebarTemplate>) -> Self {
        self.model.template = template;
        self
    }

    pub fn panel_template(mut self, template: Arc<dyn SidebarPanelTemplate>) -> Self {
        self.model.panel_template = template;
        self
    }

    pub fn with_panel_template_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(gpui::Stateful<gpui::Div>) -> gpui::Stateful<gpui::Div> + Send + Sync + 'static,
    {
        self.model.panel_template = modified_sidebar_panel_template(Arc::clone(&self.model.panel_template), modifier);
        self
    }

    pub fn scrollbar_template(mut self, template: Arc<dyn ScrollbarTemplate>) -> Self {
        self.model.scrollbar_template = template;
        self
    }

    pub fn scrollbar_placement(mut self, placement: ScrollbarPlacement) -> Self {
        self.model.scrollbar_placement = placement;
        self
    }

    pub fn overlay_scrollbar(mut self, overlay: bool) -> Self {
        self.model.scrollbar_placement = if overlay {
            ScrollbarPlacement::Overlay
        } else {
            ScrollbarPlacement::Inset
        };
        self
    }

    pub fn scrollbar_visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.model.scrollbar_visibility = visibility;
        self
    }

    pub fn auto_hide_scrollbar(mut self, auto_hide: bool) -> Self {
        self.model.scrollbar_visibility = if auto_hide {
            ScrollbarVisibility::AutoHide
        } else {
            ScrollbarVisibility::AlwaysVisible
        };
        self
    }

    pub fn auto_hide_scrollbar_activate(mut self, activate: ScrollbarAutoHideActivate) -> Self {
        self.model.scrollbar_auto_hide_activate = activate;
        self
    }

    pub fn spawn(self, cx: &mut impl AppContext) -> Entity<SidebarControl> {
        cx.new(|cx| SidebarControl::from_builder(self, cx))
    }
}

pub fn pane_render<E, F>(render: F) -> SidebarPaneRender
where
    E: IntoElement,
    F: Fn() -> E + 'static,
{
    Rc::new(move || render().into_any_element())
}

pub(crate) fn find_active_id(panel: &SidebarPanelModel) -> Option<SharedString> {
    if let Some(content) = &panel.content {
        for group in &content.groups {
            if let Some(menu) = &group.menu
                && let Some(id) = find_active_in_items(&menu.items)
            {
                return Some(id);
            }
        }
    }
    if let Some(footer) = &panel.footer
        && let Some(id) = find_active_in_items(&footer.items)
    {
        return Some(id);
    }
    None
}

fn find_active_in_items(items: &[SidebarMenuItemModel]) -> Option<SharedString> {
    for item in items {
        if item.active {
            return Some(item.id.clone());
        }
        if let Some(id) = find_active_in_items(&item.children) {
            return Some(id);
        }
    }
    None
}

pub(crate) fn panel_to_nav_nodes(panel: &SidebarPanelModel) -> (Vec<NavNode>, Vec<NavNode>) {
    let mut main_nodes = Vec::new();
    if let Some(content) = &panel.content {
        for (group_index, group) in content.groups.iter().enumerate() {
            if let Some(label) = &group.label {
                let section_id = SharedString::from(format!("{}-group-{group_index}", panel.id));
                main_nodes.push(NavNode::section(section_id, label.clone()));
            }
            if let Some(menu) = &group.menu {
                main_nodes.extend(menu.items.iter().map(menu_item_to_nav_node));
            }
        }
    }

    let footer_nodes = panel
        .footer
        .as_ref()
        .map(|footer| footer.items.iter().map(menu_item_to_nav_node).collect())
        .unwrap_or_default();

    (main_nodes, footer_nodes)
}

fn menu_item_to_nav_node(item: &SidebarMenuItemModel) -> NavNode {
    let mut node = NavNode::new(item.id.clone()).label(item.label.clone()).enabled(!item.disabled);

    if let Some(icon) = item.icon {
        node = node.icon(icon);
    }

    if !item.children.is_empty() {
        let child_active = find_active_in_items(&item.children).is_some();
        let expanded = item.expanded.unwrap_or(child_active || item.active);
        node = node.expanded(expanded).children(item.children.iter().map(menu_item_to_nav_node));
    }

    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::engine::NavNodeKind;

    #[test]
    fn menu_tree_maps_to_nav_nodes_with_sections_and_selection() {
        let panel = SidebarBuilder::new("workbench")
            .content(
                SidebarContentBuilder::new()
                    .group(
                        SidebarGroupBuilder::new().label("Pinned").menu(
                            SidebarMenuBuilder::new("pinned")
                                .item(SidebarMenuItemBuilder::new("summary", "Summary").icon(LucideIcon::Info)),
                        ),
                    )
                    .group(
                        SidebarGroupBuilder::new().label("Properties").menu(
                            SidebarMenuBuilder::new("properties").item(
                                SidebarMenuItemBuilder::new("layout", "Layout").icon(LucideIcon::Ruler).sub(
                                    SidebarMenuSubBuilder::new()
                                        .item(SidebarMenuItemBuilder::new("position", "Position"))
                                        .item(SidebarMenuItemBuilder::new("dimensions", "Dimensions").active(true)),
                                ),
                            ),
                        ),
                    ),
            )
            .footer(
                SidebarFooterBuilder::new()
                    .child(SidebarMenuItemBuilder::new("audit", "Audit Log").icon(LucideIcon::FileText)),
            )
            .build();

        assert_eq!(find_active_id(&panel).as_deref(), Some("dimensions"));

        let (nodes, footer) = panel_to_nav_nodes(&panel);
        assert_eq!(nodes[0].id().as_ref(), "workbench-group-0");
        assert!(matches!(nodes[0].kind, NavNodeKind::Section));
        assert_eq!(nodes[1].id().as_ref(), "summary");
        assert_eq!(nodes[3].id().as_ref(), "layout");
        assert!(nodes[3].is_expanded());
        assert_eq!(footer.len(), 1);
        assert_eq!(footer[0].id().as_ref(), "audit");
    }

    #[test]
    fn test_sidebar_builder_animated_option() {
        let builder_default = SidebarControlBuilder::new("sidebar");
        assert!(builder_default.model.animated);

        let builder_opt_out = SidebarControlBuilder::new("sidebar").animated(false);
        assert!(!builder_opt_out.model.animated);
    }
}
