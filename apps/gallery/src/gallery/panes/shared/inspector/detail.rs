#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, ClipboardItem, Context, Div, Entity, FontWeight, IntoElement, Render, SharedString, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::command::button::{ButtonRenderModel, ControlPresenter};
use gpui_luma::controls::tree_view::TreeViewControl;
use gpui_luma::theme::pack::LumaChrome;
use gpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, format_inspector_hsl, format_inspector_rgba};
use lucide_icons::Icon as LucideIcon;

use super::box_model::{BoxModelLayerColors, MetricFieldHighlight, render_box_model_diagram};
use super::types::{
    ColorInspectTreeData, InspectColorFieldData, InspectFieldKind, InspectFieldSelection, InspectLayoutSizeData,
    InspectMetricPropertyData, find_inspect_field_selection,
};

mod layout {
    pub(super) const PANEL_PADDING: f32 = 12.0;
    pub(super) const SWATCH_HEIGHT: f32 = 88.0;
    pub(super) const SWATCH_RADIUS: f32 = 8.0;
    pub(super) const BOX_MODEL_GAP: f32 = 8.0;
    pub(super) const SECTION_GAP: f32 = 12.0;
    pub(super) const CARD_PADDING: f32 = 10.0;
    pub(super) const CARD_GAP: f32 = 6.0;
    pub(super) const VALUE_ROW_HEIGHT: f32 = 32.0;
}

#[derive(Clone)]
struct ColorValueRow {
    label: &'static str,
    value: SharedString,
}

pub(in crate::gallery) struct ColorInspectorDetail {
    look: Arc<ShadcnLook>,
    tree: Entity<TreeViewControl<ColorInspectTreeData>>,
    detail_id: &'static str,
}

impl ColorInspectorDetail {
    pub fn new(
        look: Arc<ShadcnLook>,
        tree: Entity<TreeViewControl<ColorInspectTreeData>>,
        detail_id: &'static str,
        _cx: &mut Context<Self>,
    ) -> Self {
        Self { look, tree, detail_id }
    }

    fn selected_field(&self, cx: &gpui::App) -> Option<InspectFieldSelection> {
        let tree = self.tree.read(cx);
        tree.selected_ids().iter().next().and_then(|id| find_inspect_field_selection(tree.items(), id))
    }
}

impl Render for ColorInspectorDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selection = self.selected_field(cx);
        let chrome = self.look.chrome();
        let body = &self.look.mode_tokens().typography.text.body;
        let mono = &self.look.mode_tokens().typography.text.caption;
        let mono_font = gallery_mono_font();

        let Some(selection) = selection.as_ref() else {
            return div()
                .id(format!("{}-empty", self.detail_id))
                .h_full()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_center()
                .p(px(layout::PANEL_PADDING))
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.muted_text)
                .child("Select a field")
                .into_any_element();
        };

        match &selection.kind {
            InspectFieldKind::Color(field) => render_color_detail(
                selection,
                field,
                &self.look,
                chrome,
                body,
                mono,
                mono_font,
                self.detail_id,
                window,
                cx,
            ),
            InspectFieldKind::Layout(field) => {
                render_layout_detail(selection, field, &self.look, chrome, body, mono, self.detail_id, window, cx)
            }
        }
    }
}

fn render_copy_icon_button<M: 'static>(
    look: &Arc<ShadcnLook>,
    id: SharedString,
    clipboard_text: String,
    window: &mut Window,
    cx: &mut Context<M>,
) -> AnyElement {
    let disabled = clipboard_text.is_empty();
    let content: ControlPresenter<ButtonRenderModel<()>> = Arc::new(|_, _| {
        div()
            .font_family("lucide")
            .text_size(px(16.0))
            .line_height(px(16.0))
            .child(char::from(LucideIcon::Copy).to_string())
            .into_any_element()
    });
    let model = ButtonRenderModel {
        id: id.clone(),
        data: (),
        content,
        role: ButtonFamilyRole::Icon,
        size: ButtonSize::Sm,
        state: InteractionState { disabled, ..InteractionState::default() },
        round: true,
        radius_override: std::cell::Cell::new(None),
        elevation: true,
        compact: false,
        look: None,
    };
    let button = look.button_template(ShadcnButtonStyle::Ghost).render(&model, window, cx);

    if disabled {
        button.into_any_element()
    } else {
        button
            .on_click(cx.listener(move |_, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(clipboard_text.clone()));
            }))
            .into_any_element()
    }
}

fn render_color_detail(
    selection: &InspectFieldSelection,
    field: &InspectColorFieldData,
    look: &Arc<ShadcnLook>,
    chrome: LumaChrome,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    detail_id: &'static str,
    window: &mut Window,
    cx: &mut Context<ColorInspectorDetail>,
) -> AnyElement {
    let color = field.swatch;
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
            detail_id,
            chrome.border,
            chrome.muted_text,
            body.size,
            body.line_height,
            mono.size,
            mono.line_height,
            mono_font.clone(),
            look,
            window,
            cx,
        ));
    }

    div()
        .id(detail_id)
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
        .child(color_field_header(
            selection,
            &field.css_key,
            field.provenance.clone(),
            chrome.title_text,
            chrome.muted_text,
            body,
            mono,
            mono_font,
        ))
        .child(values_card)
        .into_any_element()
}

fn render_layout_detail(
    selection: &InspectFieldSelection,
    field: &InspectLayoutSizeData,
    look: &Arc<ShadcnLook>,
    chrome: LumaChrome,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    detail_id: &'static str,
    window: &mut Window,
    cx: &mut Context<ColorInspectorDetail>,
) -> AnyElement {
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
                .child("Layout"),
        );

    for property in &field.properties {
        values_card = values_card.child(render_layout_property_row(
            property,
            detail_id,
            chrome.border,
            chrome.muted_text,
            body.size,
            body.line_height,
            mono.size,
            mono.line_height,
            look,
            window,
            cx,
        ));
    }

    let mono_font = gallery_mono_font();

    let mut detail = div()
        .id(detail_id)
        .h_full()
        .min_w(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(layout::SECTION_GAP))
        .p(px(layout::PANEL_PADDING))
        .child(layout_field_header(
            selection,
            field.size,
            chrome.title_text,
            chrome.muted_text,
            body,
            mono,
            gallery_mono_font(),
        ));

    if let Some(box_model) = field.box_model.as_ref() {
        let box_colors = BoxModelLayerColors::from_look(look);
        let chrome_outline = look.token_color("foreground").unwrap_or(chrome.title_text);
        detail = detail.child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(layout::SWATCH_RADIUS))
                .bg(chrome.panel_background)
                .p(px(layout::CARD_PADDING))
                .child(
                    div()
                        .pb(px(layout::BOX_MODEL_GAP))
                        .text_size(px(body.size))
                        .line_height(px(body.line_height))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.title_text)
                        .child("Box model"),
                )
                .child(render_box_model_diagram(
                    SharedString::from(format!("{detail_id}-box-model")),
                    box_model,
                    field.occupation.as_ref(),
                    MetricFieldHighlight::None,
                    box_colors,
                    chrome_outline,
                    chrome.muted_text,
                    mono,
                    mono_font.clone(),
                )),
        );
    }

    detail.child(values_card).into_any_element()
}

fn color_field_header(
    selection: &InspectFieldSelection,
    source: &SharedString,
    provenance: Option<SharedString>,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> Div {
    let mut header = div().flex().flex_col().gap(px(4.0)).child(
        div()
            .text_size(px(body.size))
            .line_height(px(body.line_height))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(title_text)
            .child(selection.label.clone()),
    );

    header = header.child(
        div()
            .font_family(mono_font.clone())
            .text_size(px(mono.size))
            .line_height(px(mono.line_height))
            .text_color(muted_text)
            .child(source.clone()),
    );

    if let Some(provenance) = provenance {
        header = header.child(
            div()
                .font_family(mono_font)
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(provenance),
        );
    }

    header
}

fn layout_field_header(
    selection: &InspectFieldSelection,
    size: ControlSize,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_text)
                .child(selection.label.clone()),
        )
        .child(
            div()
                .font_family(mono_font)
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(format!("{} · layout", control_size_label(size))),
        )
}

fn render_layout_property_row(
    property: &InspectMetricPropertyData,
    detail_id: &str,
    border: gpui::Hsla,
    muted_text: gpui::Hsla,
    label_size: f32,
    label_line_height: f32,
    mono_size: f32,
    mono_line_height: f32,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut Context<ColorInspectorDetail>,
) -> impl IntoElement {
    let value = property.value.clone();
    let value_for_copy = value.to_string();
    let row_id = property.name.replace(' ', "-");

    div()
        .id(format!("{detail_id}-layout-{row_id}"))
        .flex()
        .flex_col()
        .gap(px(2.0))
        .pb(px(layout::CARD_GAP))
        .border_b_1()
        .border_color(border)
        .child(
            div()
                .flex()
                .items_center()
                .h(px(layout::VALUE_ROW_HEIGHT))
                .cursor_pointer()
                .child(
                    div()
                        .w(px(96.0))
                        .flex_shrink_0()
                        .text_size(px(label_size))
                        .line_height(px(label_line_height))
                        .text_color(muted_text)
                        .child(property.name.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .truncate()
                        .text_size(px(mono_size))
                        .line_height(px(mono_line_height))
                        .text_color(muted_text)
                        .child(value),
                )
                .child(render_copy_icon_button(
                    look,
                    format!("{detail_id}-layout-copy-{row_id}").into(),
                    value_for_copy.clone(),
                    window,
                    cx,
                )),
        )
        .child(
            div()
                .pl(px(96.0))
                .font_family(gallery_mono_font())
                .text_size(px(mono_size))
                .line_height(px(mono_line_height))
                .text_color(muted_text)
                .child(property.source.clone()),
        )
        .when_some(property.provenance.clone(), |row, provenance| {
            row.child(
                div()
                    .pl(px(96.0))
                    .font_family(gallery_mono_font())
                    .text_size(px(mono_size))
                    .line_height(px(mono_line_height))
                    .text_color(muted_text)
                    .child(provenance),
            )
        })
}

fn control_size_label(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "Sm",
        ControlSize::Md => "Md",
        ControlSize::Lg => "Lg",
    }
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
    detail_id: &'static str,
    border: gpui::Hsla,
    muted_text: gpui::Hsla,
    label_size: f32,
    label_line_height: f32,
    mono_size: f32,
    mono_line_height: f32,
    mono_font: SharedString,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut Context<ColorInspectorDetail>,
) -> impl IntoElement {
    let value = row.value.clone();
    let row_key = row.label.to_ascii_lowercase();

    div()
        .id(format!("{detail_id}-value-{row_key}"))
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
        .child(render_copy_icon_button(
            look,
            format!("{detail_id}-copy-{row_key}").into(),
            value.to_string(),
            window,
            cx,
        ))
}
