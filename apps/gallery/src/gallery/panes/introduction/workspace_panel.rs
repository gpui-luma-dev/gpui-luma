use std::cell::Cell;
use std::sync::Arc;

use gpui::{
    App, Context, Entity, IntoElement, MouseButton, Render, SharedString, Window, div, prelude::*, transparent_black,
    px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::button_group::{IconGroup, IconGroupEvent, IconGroupItem, IconGroupItemLike};
use gpui_luma::controls::control_group::toggle_button_item_template;
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate};
use gpui_luma::controls::icon::lucide_glyph;
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{PopupMenu, PopupMenuEvent, PopupMenuPlacement};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma::{declare_form, hstack, vstack};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma::controls::radio_group::{
    self as radio_group, RadioGroup, RadioGroupEvent, RadioGroupItem, RadioGroupItemLike, RadioGroupRenderModel,
    RadioGroupTemplate, RadioGroupTemplateHandlers,
};
use lucide_icons::Icon as LucideIcon;

use super::pane::{AppEvent, EventBus};

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkspaceDensity {
    Compact,
    Balanced,
    Comfortable,
}

impl WorkspaceDensity {
    fn label(&self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Balanced => "Balanced",
            Self::Comfortable => "Comfortable",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        match id {
            "compact" => Some(Self::Compact),
            "balanced" => Some(Self::Balanced),
            "comfortable" => Some(Self::Comfortable),
            _ => None,
        }
    }
}

declare_form! {
    pub(super) struct WorkspacePanel {
        controls: {
            popup_menu: Entity<PopupMenu> = look
                .popup_menu("intro-workspace-popup")
                .label("Workspace Menu")
                .items(menu_items())
                .placement(PopupMenuPlacement::BelowStart)
                => PopupMenuEvent |this, event, cx| {
                    this.handle_popup_menu_event(event, cx);
                },
            layout_icon_group: IconGroup<IconGroupItem> = {
                let toggle_primary = look.toggle_template(ShadcnButtonStyle::Primary);

                look
                    .button_group("intro-workspace-layout")
                    .horizontal()
                    .with_template_modifier(|element, _| element.bg(transparent_black()))
                    .managed_selected("grid")
                    .items(layout_items())
                    .item_template(toggle_button_item_template(toggle_primary.clone(), true, |item: &IconGroupItem| {
                        let icon = match item.id().as_ref() {
                            "grid" => LucideIcon::PanelTop,
                            "list" => LucideIcon::List,
                            "kanban" => LucideIcon::Columns3,
                            _ => LucideIcon::Settings,
                        };

                        lucide_glyph(icon)
                    }))
                }
                => IconGroupEvent |this, event, cx| {
                    this.handle_layout_event(event, cx);
                },
            icon_demo_icon_group: IconGroup<IconGroupItem> = {
                let toggle_secondary = look.toggle_template(ShadcnButtonStyle::Secondary);

                look
                    .button_group("intro-workspace-icon-demo")
                    .horizontal()
                    .managed_selected("left")
                    .items(icon_demo_items())
                    .item_template(toggle_button_item_template(toggle_secondary, true, |item: &IconGroupItem| {
                        let icon = match item.id().as_ref() {
                            "left" => LucideIcon::List,
                            "center" => LucideIcon::PanelTop,
                            "right" => LucideIcon::Columns3,
                            _ => LucideIcon::Settings,
                        };

                        lucide_glyph(icon)
                    }))
                }
                => IconGroupEvent |this, event, cx| {
                    this.handle_icon_demo_event(event, cx);
                },
            density_radio_group: RadioGroup<RadioGroupItem> = {
                let radio_template = look.radio_button_template(ShadcnButtonStyle::Primary);

                radio_group::horizontal("intro-workspace-density")
                    .items(density_items())
                    .selected("balanced")
                    .template(density_template(radio_template))
                }
                => RadioGroupEvent |this, event, cx| {
                    this.handle_density_event(event, cx);
                },
        },
        args: {
            look: Arc<ShadcnLook>,
            event_bus: Entity<EventBus>,
        },
        fields: {
            layout: SharedString = SharedString::from("Grid"),
            density: SharedString = SharedString::from("Balanced"),
            icon_demo: SharedString = SharedString::from("Left"),
            action: SharedString = SharedString::from("None"),
        }
    }
}

impl WorkspacePanel {
    fn handle_popup_menu_event(&mut self, event: &PopupMenuEvent, cx: &mut Context<Self>) {
        match event {
            PopupMenuEvent::Select { label, .. } => {
                self.action = label.clone();
                self.emit_change("PopupMenu::Select", cx);
                cx.notify();
            }
        }
    }

    fn handle_layout_event(&mut self, event: &IconGroupEvent, cx: &mut Context<Self>) {
        match event {
            IconGroupEvent::Change { changed_id, selected_ids, selected, .. } => {
                self.layout = if *selected {
                    layout_label(changed_id.as_ref())
                } else {
                    SharedString::from("None")
                };

                let next_selected_ids = selected_ids.clone();
                self.layout_icon_group.update(cx, |group, cx| {
                    group.set_managed_selected_ids(next_selected_ids, cx);
                });

                self.emit_change("IconGroup::Layout", cx);
                cx.notify();
            }
        }
    }

    fn handle_icon_demo_event(&mut self, event: &IconGroupEvent, cx: &mut Context<Self>) {
        match event {
            IconGroupEvent::Change { changed_id, selected_ids, selected, .. } => {
                self.icon_demo = if *selected {
                    icon_demo_label(changed_id.as_ref())
                } else {
                    SharedString::from("None")
                };

                let next_selected_ids = selected_ids.clone();
                self.icon_demo_icon_group.update(cx, |group, cx| {
                    group.set_managed_selected_ids(next_selected_ids, cx);
                });

                self.emit_change("IconGroup::IconDemo", cx);
                cx.notify();
            }
        }
    }

    fn handle_density_event(&mut self, event: &RadioGroupEvent, cx: &mut Context<Self>) {
        match event {
            RadioGroupEvent::Change { selected_ids, .. } => {
                self.density = selected_ids.first().map_or_else(
                    || SharedString::from("None"),
                    |selected_id| {
                        WorkspaceDensity::from_id(selected_id.as_ref())
                            .map_or_else(|| selected_id.clone(), |density| SharedString::from(density.label()))
                    },
                );

                self.emit_change("RadioGroup::Density", cx);
                cx.notify();
            }
        }
    }

    fn emit_change(&self, event_name: &'static str, cx: &mut Context<Self>) {
        let layout = self.layout.clone();
        let density = self.density.clone();
        let icon_demo = self.icon_demo.clone();
        let action = self.action.clone();

        self.event_bus.update(cx, |_bus, cx| {
            cx.emit(AppEvent::WorkspaceChanged {
                layout,
                density,
                icon_demo,
                action,
                event_name: SharedString::from(event_name),
            });
        });
    }
}

impl Render for WorkspacePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();

        let layout_icon_group = self.layout_icon_group.clone();
        let density_radio_group = self.density_radio_group.clone();
        let popup_menu = self.popup_menu.clone();
        let icon_demo_icon_group = self.icon_demo_icon_group.clone();
        let layout = self.layout.clone();
        let density = self.density.clone();
        let icon_demo = self.icon_demo.clone();
        let action = self.action.clone();
        let body_style = self.look.typography_scale(gpui_luma_look_shadcn::ShadcnTextSize::Sm);
        let caption_style = self.look.typography_scale(gpui_luma_look_shadcn::ShadcnTextSize::Xs);

        div().w(px(360.0)).max_w_full().h_full().child(
            self.look
                .card("intro-workspace-card")
                .title("Workspace")
                .description("Toggle groups, radio groups, icon actions, and popup menus.")
                .elevated(false)
                .full_height(true)
                .body_fill(true)
                .child_render(move |_, _| {
                    vstack! {
                        gap=10.0;
                        vstack! {
                            gap=6.0 align=start;
                            div()
                                .typography_style(body_style)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(chrome.body_text)
                                .child(format!("Layout: {}", layout)),
                            layout_icon_group.clone(),
                        },
                        vstack! {
                            gap=6.0;
                            div()
                                .typography_style(body_style)
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .text_color(chrome.body_text)
                                .child(format!("Density: {}", density)),
                            density_radio_group.clone(),
                        },
                        hstack! {
                            gap=8.0 align=center;
                            popup_menu.clone(),
                        },
                        vstack! {
                            gap=6.0 align=start;
                            div()
                                .typography_style(caption_style)
                                .text_color(chrome.muted_text)
                                .child(format!("Icon demo: {}", icon_demo)),
                            icon_demo_icon_group.clone(),
                        },
                        div()
                            .pt(px(2.0))
                            .typography_style(caption_style)
                            .text_color(chrome.muted_text)
                            .child(format!("Workspace action: {} | Icon demo: {}", action, icon_demo)),
                    }
                    .into_any_element()
                })
                .render(window, cx),
        )
    }
}

fn layout_items() -> [IconGroupItem; 4] {
    [
        IconGroupItem::new("grid").label("Grid"),
        IconGroupItem::new("list").label("List"),
        IconGroupItem::new("kanban").label("Kanban"),
        IconGroupItem::new("another").label("Another"),
    ]
}

fn layout_label(id: &str) -> SharedString {
    match id {
        "grid" => SharedString::from("Grid"),
        "list" => SharedString::from("List"),
        "kanban" => SharedString::from("Kanban"),
        "another" => SharedString::from("Another"),
        _ => SharedString::from(id.to_owned()),
    }
}

fn density_items() -> [RadioGroupItem; 3] {
    [
        RadioGroupItem::new("compact").label("Compact"),
        RadioGroupItem::new("balanced").label("Balanced"),
        RadioGroupItem::new("comfortable").label("Comfortable"),
    ]
}

fn icon_demo_items() -> [IconGroupItem; 3] {
    [
        IconGroupItem::new("left").label("Left"),
        IconGroupItem::new("center").label("Center"),
        IconGroupItem::new("right").label("Right"),
    ]
}

fn icon_demo_label(id: &str) -> SharedString {
    match id {
        "left" => SharedString::from("Left"),
        "center" => SharedString::from("Center"),
        "right" => SharedString::from("Right"),
        _ => SharedString::from(id.to_owned()),
    }
}

fn menu_items() -> [MenuItem; 5] {
    [
        MenuItem::new("sync-now").label("Sync now").icon(LucideIcon::RefreshCw),
        MenuItem::new("share-workspace").label("Share workspace").icon(LucideIcon::Share2),
        MenuItem::new("duplicate").label("Duplicate").icon(LucideIcon::Copy),
        MenuItem::new("move").label("Move to…").icon(LucideIcon::FolderInput).submenu([
            MenuItem::new("team-space").label("Team Space").icon(LucideIcon::Users),
            MenuItem::new("archive-space").label("Archive").icon(LucideIcon::Archive),
        ]),
        MenuItem::new("delete").label("Delete").icon(LucideIcon::Trash2),
    ]
}

fn density_template(button_template: Arc<dyn ButtonTemplate<bool>>) -> RadioGroupTemplate<RadioGroupItem> {
    Arc::new(move |model, handlers, window, cx| render_density_group(model, handlers, &button_template, window, cx))
}

fn render_density_group(
    model: &RadioGroupRenderModel<'_, RadioGroupItem>,
    handlers: RadioGroupTemplateHandlers,
    button_template: &Arc<dyn ButtonTemplate<bool>>,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let RadioGroupTemplateHandlers { item_hovers, item_mouse_downs, item_mouse_ups, item_mouse_up_outs, item_clicks } =
        handlers;

    let mut item_hovers = item_hovers.into_iter();
    let mut item_mouse_downs = item_mouse_downs.into_iter();
    let mut item_mouse_ups = item_mouse_ups.into_iter();
    let mut item_mouse_up_outs = item_mouse_up_outs.into_iter();
    let mut item_clicks = item_clicks.into_iter();

    let mut root = div().id(model.id.clone()).flex().items_center().gap_3();

    for item in &model.items {
        let Some(item_hover) = item_hovers.next() else {
            break;
        };
        let Some(item_mouse_down) = item_mouse_downs.next() else {
            break;
        };
        let Some(item_mouse_up) = item_mouse_ups.next() else {
            break;
        };
        let Some(item_mouse_up_out) = item_mouse_up_outs.next() else {
            break;
        };
        let Some(item_click) = item_clicks.next() else {
            break;
        };

        let render_model = ButtonRenderModel {
            id: format!("{}-{}", model.id, RadioGroupItemLike::id(item.item)).into(),
            data: item.selected,
            content: Arc::new({
                let label = RadioGroupItemLike::label(item.item).clone();
                move |_, _| div().child(label.clone()).into_any_element()
            }),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: item.state.interaction_state(),
            round: false,
            radius_override: Cell::new(None),
            look: None,
        };

        let mut button = button_template
            .render(&render_model, window, cx)
            .on_hover(item_hover)
            .on_mouse_down(MouseButton::Left, item_mouse_down)
            .on_mouse_up(MouseButton::Left, item_mouse_up)
            .on_mouse_up_out(MouseButton::Left, item_mouse_up_out)
            .on_click(item_click);

        if item.enabled {
            button = button.cursor_pointer();
        } else {
            button = button.opacity(0.56);
        }

        root = root.child(button);
    }

    root
}
