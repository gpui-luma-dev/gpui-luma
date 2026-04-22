use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};
use gpui_luma::controls::menu_item::{MenuItem, MenuItemIcon};
use gpui_luma::theme::{DefaultPopupMenuTheme, FloatingMenuAppearance, InteractionState, PopupMenuTheme};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::theme::GalleryThemePack;

use super::super::shared::gallery_pane_with_usage;

#[derive(Clone)]
pub(in crate::gallery) struct FloatingMenuPane;

impl FloatingMenuPane {
    pub(in crate::gallery) fn new() -> Self {
        Self
    }

    pub(in crate::gallery) fn render(&self, theme: &GalleryThemePack) -> AnyElement {
        let chrome = theme.chrome();
        let popup_theme = DefaultPopupMenuTheme::new(theme.tokens());
        let appearance = popup_theme.resolve(InteractionState::default()).floating_menu;

        gallery_pane_with_usage(
            "Floating Menu",
            "Floating Menu",
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_start()
                        .justify_center()
                        .gap(px(16.0))
                        .child(render_state_sample(
                            "Default",
                            &appearance,
                            chrome.muted_text,
                            &default_items(),
                            FloatingMenuSampleState::Default,
                        ))
                        .child(render_state_sample(
                            "Hover / active item",
                            &appearance,
                            chrome.muted_text,
                            &default_items(),
                            FloatingMenuSampleState::Active(1),
                        ))
                        .child(render_state_sample(
                            "Disabled item",
                            &appearance,
                            chrome.muted_text,
                            &disabled_items(),
                            FloatingMenuSampleState::Default,
                        ))
                        .child(render_state_sample(
                            "Submenu affordance",
                            &appearance,
                            chrome.muted_text,
                            &submenu_items(),
                            FloatingMenuSampleState::Default,
                        )),
                )
                .into_any_element(),
            theme,
        )
    }

    pub(in crate::gallery) fn notify_controls(&self) {}
}

#[derive(Clone, Copy)]
enum FloatingMenuSampleState {
    Default,
    Active(usize),
}

fn render_state_sample(
    label: &'static str,
    appearance: &FloatingMenuAppearance,
    label_color: gpui::Hsla,
    items: &[MenuItem],
    state: FloatingMenuSampleState,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(7.0))
        .child(render_menu_panel(format!("floating-menu-sample-{}", sample_id(label)), items, appearance, state))
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

fn render_menu_panel(
    id: String,
    items: &[MenuItem],
    appearance: &FloatingMenuAppearance,
    state: FloatingMenuSampleState,
) -> AnyElement {
    div()
        .id(id.clone())
        .min_w(px(appearance.min_width))
        .p(px(appearance.padding))
        .bg(appearance.background)
        .border_1()
        .border_color(appearance.border)
        .rounded(px(appearance.radius))
        .shadow(appearance.shadow.clone())
        .children(
            items
                .iter()
                .enumerate()
                .map(|(index, item)| render_menu_item_row(&id, item, index, appearance, state)),
        )
        .into_any_element()
}

fn render_menu_item_row(
    panel_id: &str,
    item: &MenuItem,
    index: usize,
    appearance: &FloatingMenuAppearance,
    state: FloatingMenuSampleState,
) -> AnyElement {
    let enabled = item.is_enabled();
    let active = matches!(state, FloatingMenuSampleState::Active(active_index) if active_index == index);
    let color = if enabled {
        appearance.foreground
    } else {
        appearance.item_disabled_foreground
    };

    let mut row = div()
        .id(format!("{panel_id}-item-{}", item.id()))
        .flex()
        .items_center()
        .gap(px(appearance.item_gap))
        .min_h(px(appearance.item_height))
        .px(px(appearance.item_padding_x))
        .rounded(px(appearance.item_radius))
        .text_color(color)
        .text_size(px(appearance.item_typography.size))
        .line_height(px(appearance.item_typography.line_height))
        .font_weight(appearance.item_typography.weight)
        .child(render_item_icon(item.icon_ref(), color, appearance.item_icon_size))
        .child(div().flex_1().child(item.label_text().clone()))
        .child(render_submenu_affordance(!item.submenu_items().is_empty(), color, appearance.item_icon_size));

    if active && enabled {
        row = row.bg(appearance.item_hover_background);
    }

    if enabled {
        let hover_background = appearance.item_hover_background;
        row = row.cursor_pointer().hover(move |style| style.bg(hover_background));
    } else {
        row = row.opacity(0.56);
    }

    row.into_any_element()
}

fn render_item_icon(icon: Option<&MenuItemIcon>, color: gpui::Hsla, size: f32) -> AnyElement {
    if let Some(icon) = icon.and_then(MenuItemIcon::lucide) {
        render_lucide_icon(icon, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_submenu_affordance(has_submenu: bool, color: gpui::Hsla, size: f32) -> AnyElement {
    if has_submenu {
        render_lucide_icon(LucideIcon::ChevronRight, color, size)
    } else {
        div().size(px(size)).into_any_element()
    }
}

fn render_lucide_icon(icon: LucideIcon, color: gpui::Hsla, size: f32) -> AnyElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .font_family("lucide")
        .font_weight(FontWeight::NORMAL)
        .text_size(px(size))
        .line_height(px(size))
        .text_color(color)
        .child(char::from(icon).to_string())
        .into_any_element()
}

fn default_items() -> [MenuItem; 3] {
    [
        MenuItem::new("new").label("New file").icon(LucideIcon::FilePlus),
        MenuItem::new("rename").label("Rename").icon(LucideIcon::Pencil),
        MenuItem::new("archive").label("Archive"),
    ]
}

fn disabled_items() -> [MenuItem; 3] {
    [
        MenuItem::new("open").label("Open").icon(LucideIcon::FolderOpen),
        MenuItem::new("download").label("Download").icon(LucideIcon::Download).enabled(false),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2),
    ]
}

fn submenu_items() -> [MenuItem; 3] {
    [
        MenuItem::new("copy").label("Copy").icon(LucideIcon::Copy),
        MenuItem::new("share").label("Share").icon(LucideIcon::Share2).submenu([
            MenuItem::new("copy-link").label("Copy link").icon(LucideIcon::Link),
            MenuItem::new("email").label("Email").icon(LucideIcon::Mail),
        ]),
        MenuItem::new("inspect").label("Inspect"),
    ]
}

fn sample_id(label: &str) -> String {
    label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}
