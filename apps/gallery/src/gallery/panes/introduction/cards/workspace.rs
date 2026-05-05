use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px};

use crate::gallery::theme::GalleryThemePack;

use super::common::{card_container, card_title};
use super::super::pane::IntroductionPane;

pub(in crate::gallery) fn render_workspace_card(pane: &IntroductionPane, theme: &GalleryThemePack) -> AnyElement {
    let chrome = theme.chrome();

    card_container(chrome.border, chrome.panel_background)
        .child(card_title(
            "Workspace",
            "Toggle groups, radio groups, icon actions, and popup menus.",
            chrome.title_text,
            chrome.muted_text,
        ))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(format!("Layout: {}", pane.workspace_layout)),
                )
                .child(pane.workspace.workspace_layout_choice_group.clone()),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(12.0))
                        .line_height(px(16.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.body_text)
                        .child(format!("Density: {}", pane.workspace_density)),
                )
                .child(pane.workspace.workspace_density_choice_group.clone()),
        )
        .child(div().flex().items_center().gap(px(8.0)).child(pane.workspace.workspace_popup_menu.clone()))
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(6.0))
                .child(
                    div()
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(chrome.muted_text)
                        .child(format!("Icon demo: {}", pane.workspace_icon_demo)),
                )
                .child(pane.workspace.workspace_icon_demo_choice_group.clone()),
        )
        .child(
            div()
                .pt(px(2.0))
                .text_size(px(11.0))
                .line_height(px(16.0))
                .text_color(chrome.muted_text)
                .child(format!(
                    "Workspace action: {} | Icon demo: {}",
                    pane.workspace_action, pane.workspace_icon_demo
                )),
        )
        .into_any_element()
}
