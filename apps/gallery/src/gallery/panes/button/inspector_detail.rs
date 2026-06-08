use std::sync::Arc;

use gpui::{
    App, ClipboardItem, Context, Entity, FontWeight, IntoElement, Render, SharedString, Window, div, prelude::*, px,
};
use gpui_luma::controls::tree_view::TreeViewControl;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, format_inspector_hsl, format_inspector_rgba};
use lucide_icons::Icon as LucideIcon;

use super::inspector_tree::{InspectFieldSelection, InspectTreeData, find_field_selection};

mod layout {
    pub(super) const PANEL_PADDING: f32 = 12.0;
    pub(super) const SWATCH_HEIGHT: f32 = 88.0;
    pub(super) const SWATCH_RADIUS: f32 = 8.0;
    pub(super) const SECTION_GAP: f32 = 12.0;
    pub(super) const CARD_PADDING: f32 = 10.0;
    pub(super) const CARD_GAP: f32 = 6.0;
    pub(super) const VALUE_ROW_HEIGHT: f32 = 32.0;
    pub(super) const COPY_SLOT: f32 = 28.0;
}

#[derive(Clone)]
struct ColorValueRow {
    label: &'static str,
    value: SharedString,
}

pub(in crate::gallery) struct ButtonInspectorDetail {
    look: Arc<ShadcnLook>,
    tree: Entity<TreeViewControl<InspectTreeData>>,
}

impl ButtonInspectorDetail {
    pub fn new(
        look: Arc<ShadcnLook>,
        tree: Entity<TreeViewControl<InspectTreeData>>,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self { look, tree }
    }

    fn selected_field(&self, cx: &App) -> Option<InspectFieldSelection> {
        let tree = self.tree.read(cx);
        tree.selected_ids()
            .iter()
            .next()
            .and_then(|id| find_field_selection(tree.items(), id))
    }
}

impl Render for ButtonInspectorDetail {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selection = self.selected_field(cx);
        render_inspector_detail(selection, &self.look, cx)
    }
}

fn render_inspector_detail(
    selection: Option<InspectFieldSelection>,
    look: &Arc<ShadcnLook>,
    cx: &mut Context<ButtonInspectorDetail>,
) -> impl IntoElement {
    let chrome = look.chrome();
    let body = &look.mode_tokens().typography.text.body;
    let mono = &look.mode_tokens().typography.text.caption;
    let mono_font = gallery_mono_font();

    let Some(selection) = selection.as_ref() else {
        return div()
            .id("button-inspector-detail-empty")
            .h_full()
            .min_w(px(0.0))
            .flex()
            .items_center()
            .justify_center()
            .p(px(layout::PANEL_PADDING))
            .text_size(px(body.size))
            .line_height(px(body.line_height))
            .text_color(chrome.muted_text)
            .child("Select a color field");
    };

    let color = selection.field.swatch;
    let rows = color_value_rows(color);
    let mut values_card = div()
        .flex()
        .flex_col()
        .gap(px(layout::CARD_GAP))
        .border_1()
        .border_color(chrome.border)
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .p(px(layout::CARD_PADDING))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::MEDIUM)
                .text_color(chrome.title_text)
                .child("Color values"),
        );

    for row in rows {
        values_card = values_card.child(render_color_value_row(
            row,
            color,
            chrome.border,
            chrome.muted_text,
            body.size,
            body.line_height,
            mono.size,
            mono.line_height,
            mono_font.clone(),
            cx,
        ));
    }

    let mut header = div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(selection.label.clone()),
        )
        .child(
            div()
                .font_family(mono_font.clone())
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(chrome.muted_text)
                .child(selection.field.css_key.clone()),
        );

    if let Some(provenance) = selection.field.provenance.clone() {
        header = header.child(
            div()
                .font_family(mono_font)
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(chrome.muted_text)
                .child(provenance),
        );
    }

    div()
        .id("button-inspector-detail")
        .h_full()
        .min_w(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(layout::SECTION_GAP))
        .p(px(layout::PANEL_PADDING))
        .child(
            div()
                .w_full()
                .h(px(layout::SWATCH_HEIGHT))
                .rounded(px(layout::SWATCH_RADIUS))
                .bg(color)
                .border_1()
                .border_color(chrome.border),
        )
        .child(header)
        .child(values_card)
}

fn color_value_rows(color: gpui::Hsla) -> Vec<ColorValueRow> {
    vec![
        ColorValueRow { label: "HSLA", value: format_compact_hsla(color).into() },
        ColorValueRow { label: "HEX", value: format_hex_color(color).into() },
        ColorValueRow { label: "RGBA", value: format_inspector_rgba(color).into() },
        ColorValueRow { label: "HSL", value: format_inspector_hsl(color).into() },
    ]
}

fn render_color_value_row(
    row: ColorValueRow,
    accent: gpui::Hsla,
    border: gpui::Hsla,
    muted_text: gpui::Hsla,
    label_size: f32,
    label_line_height: f32,
    mono_size: f32,
    mono_line_height: f32,
    mono_font: SharedString,
    cx: &mut Context<ButtonInspectorDetail>,
) -> impl IntoElement {
    let value = row.value.clone();

    div()
        .id(format!("button-inspector-value-{}", row.label.to_ascii_lowercase()))
        .flex()
        .items_center()
        .h(px(layout::VALUE_ROW_HEIGHT))
        .border_b_1()
        .border_color(border)
        .cursor_pointer()
        .child(
            div()
                .w(px(52.0))
                .flex_shrink_0()
                .text_size(px(label_size))
                .line_height(px(label_line_height))
                .text_color(muted_text)
                .child(row.label),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .font_family(mono_font)
                .text_size(px(mono_size))
                .line_height(px(mono_line_height))
                .text_color(accent)
                .child(row.value),
        )
        .child(
            div()
                .id(format!("button-inspector-copy-{}", row.label.to_ascii_lowercase()))
                .size(px(layout::COPY_SLOT))
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.0))
                .cursor_pointer()
                .hover(|style| style.bg(border))
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(value.to_string()));
                }))
                .child(
                    div()
                        .font_family("lucide")
                        .text_size(px(mono_size))
                        .line_height(px(mono_size))
                        .text_color(muted_text)
                        .child(char::from(LucideIcon::Copy).to_string()),
                ),
        )
}
