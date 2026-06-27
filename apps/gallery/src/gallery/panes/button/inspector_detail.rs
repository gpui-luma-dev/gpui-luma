#![allow(clippy::too_many_arguments)]

use std::sync::Arc;

use gpui::{
    AnyElement, App, ClipboardItem, Context, Div, Entity, FontWeight, IntoElement, Render, SharedString, Window, div,
    prelude::*, px,
};
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::button_family::ButtonSize;
use gpui_luma::controls::command::button::{Button, ButtonRenderModel, ControlPresenter};
use gpui_luma::controls::presenter::HasPresenter;
use gpui_luma::controls::tree_view::TreeViewControl;
use gpui_luma::theme::pack::LumaChrome;
use gpui_luma::theme::{ControlSize, InteractionState, LumaTextStyle};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};

use crate::fonts::gallery_mono_font;
use crate::gallery::panes::shared::{format_compact_hsla, format_hex_color, format_inspector_hsl, format_inspector_rgba};
use lucide_icons::Icon as LucideIcon;

use crate::gallery::panes::shared::inspector::box_model::{
    BoxModelLayerColors, MetricFieldHighlight, render_box_model_diagram,
};
use super::inspector_tree::{
    InspectColorFieldData, InspectElevationData, InspectFieldKind, InspectFieldSelection, InspectLayoutSizeData,
    InspectMetricPropertyData, InspectTreeData, InspectTypographyData, InspectTypographyPropertyData,
    find_field_selection,
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

pub(in crate::gallery) struct ButtonInspectorDetail {
    look: Arc<ShadcnLook>,
    tree: Entity<TreeViewControl<InspectTreeData>>,
    preview_buttons: [Entity<Button>; 3],
}

impl ButtonInspectorDetail {
    pub fn new(look: Arc<ShadcnLook>, tree: Entity<TreeViewControl<InspectTreeData>>, cx: &mut Context<Self>) -> Self {
        Self {
            look: look.clone(),
            tree,
            preview_buttons: [
                look.primary_button("button-inspector-preview-sm").size(ButtonSize::Sm).label("Button").spawn(cx),
                look.primary_button("button-inspector-preview-md").size(ButtonSize::Md).label("Button").spawn(cx),
                look.primary_button("button-inspector-preview-lg").size(ButtonSize::Lg).label("Button").spawn(cx),
            ],
        }
    }

    pub(in crate::gallery) fn notify_preview_buttons(&self, cx: &mut Context<Self>) {
        for button in &self.preview_buttons {
            button.update(cx, |_, cx| cx.notify());
        }
    }

    fn preview_button(&self, size: ControlSize) -> Entity<Button> {
        self.preview_buttons[preview_button_index(size)].clone()
    }

    fn selected_field(&self, cx: &App) -> Option<InspectFieldSelection> {
        let tree = self.tree.read(cx);
        tree.selected_ids().iter().next().and_then(|id| find_field_selection(tree.items(), id))
    }
}

impl Render for ButtonInspectorDetail {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selection = self.selected_field(cx);
        render_inspector_detail(selection, |size| self.preview_button(size), &self.look, window, cx)
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

fn render_inspector_detail(
    selection: Option<InspectFieldSelection>,
    preview_button: impl Fn(ControlSize) -> Entity<Button>,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
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
            .child("Select a field")
            .into_any_element();
    };

    match &selection.kind {
        InspectFieldKind::Color(field) => {
            render_color_detail(selection, field, look, chrome, body, mono, mono_font, window, cx)
        }
        InspectFieldKind::Layout(field) => {
            render_layout_detail(selection, field, preview_button, look, chrome, body, mono, mono_font, window, cx)
        }
        InspectFieldKind::Typography(field) => {
            render_typography_detail(selection, field, preview_button, look, chrome, body, mono, mono_font, window, cx)
        }
        InspectFieldKind::Elevation(field) => {
            render_elevation_detail(selection, field, preview_button, look, chrome, body, mono, mono_font, window, cx)
        }
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
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
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
        .child(field_header(
            selection,
            None,
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
    preview_button: impl Fn(ControlSize) -> Entity<Button>,
    look: &Arc<ShadcnLook>,
    chrome: LumaChrome,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
) -> AnyElement {
    let box_colors = BoxModelLayerColors::from_look(look);
    let chrome_outline = look.token_color("foreground").unwrap_or(chrome.title_text);

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

    div()
        .id("button-inspector-detail")
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
            mono_font.clone(),
        ))
        .child(
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
                    SharedString::from("button-inspector-box-model"),
                    &field.box_model,
                    Some(&field.occupation),
                    MetricFieldHighlight::None,
                    box_colors,
                    chrome_outline,
                    chrome.muted_text,
                    mono,
                    mono_font.clone(),
                )),
        )
        .child(values_card)
        .child(render_layout_size_preview(
            preview_button(field.size),
            chrome.border,
            chrome.panel_background,
            chrome.muted_text,
            body,
            field.size,
        ))
        .into_any_element()
}

fn render_elevation_detail(
    selection: &InspectFieldSelection,
    field: &InspectElevationData,
    preview_button: impl Fn(ControlSize) -> Entity<Button>,
    look: &Arc<ShadcnLook>,
    chrome: LumaChrome,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
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
                .child("Elevation"),
        );

    for property in &field.properties {
        values_card = values_card.child(render_layout_property_row(
            property,
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

    if let Some(catalog_value) = field.catalog_value.as_ref() {
        values_card = values_card.child(
            div()
                .pt(px(layout::CARD_GAP))
                .font_family(mono_font.clone())
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(chrome.muted_text)
                .child(catalog_value.clone()),
        );
    }

    let mut layers_card = div()
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
                .child("Shadow layers"),
        );

    if field.layers.is_empty() {
        layers_card = layers_card.child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .text_color(chrome.muted_text)
                .child("No shadow layers resolved for this state."),
        );
    } else {
        for layer in &field.layers {
            layers_card = layers_card.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_size(px(body.size))
                            .line_height(px(body.line_height))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(chrome.title_text)
                            .child(layer.label.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .size(px(12.0))
                                    .flex_shrink_0()
                                    .rounded_full()
                                    .bg(layer.color)
                                    .border_1()
                                    .border_color(chrome.border),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .font_family(mono_font.clone())
                                    .text_size(px(mono.size))
                                    .line_height(px(mono.line_height))
                                    .text_color(chrome.muted_text)
                                    .child(layer.css.clone()),
                            ),
                    ),
            );
        }
    }

    let mut preview_chip = div()
        .w(px(120.0))
        .h(px(36.0))
        .rounded(px(6.0))
        .bg(chrome.panel_background)
        .border_1()
        .border_color(chrome.border);

    if field.applied
        && let Some(shadows) = field.shadows.as_ref().filter(|shadows| !shadows.is_empty())
    {
        preview_chip = preview_chip.shadow(shadows.clone());
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
        .child(elevation_field_header(
            selection,
            field.style_key.clone(),
            field.token.clone(),
            chrome.title_text,
            chrome.muted_text,
            body,
            mono,
            mono_font.clone(),
        ))
        .child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(layout::SWATCH_RADIUS))
                .bg(chrome.panel_background)
                .p(px(layout::CARD_PADDING))
                .flex()
                .flex_col()
                .gap(px(layout::CARD_GAP))
                .child(
                    div()
                        .text_size(px(body.size))
                        .line_height(px(body.line_height))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.muted_text)
                        .child("Shadow preview"),
                )
                .child(
                    div()
                        .w_full()
                        .h(px(layout::SWATCH_HEIGHT))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(preview_chip),
                ),
        )
        .child(values_card)
        .child(layers_card)
        .child(render_layout_size_preview(
            preview_button(ControlSize::Md),
            chrome.border,
            chrome.panel_background,
            chrome.muted_text,
            body,
            ControlSize::Md,
        ))
        .into_any_element()
}

fn elevation_field_header(
    selection: &InspectFieldSelection,
    style_key: SharedString,
    token: Option<SharedString>,
    title_text: gpui::Hsla,
    muted_text: gpui::Hsla,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
) -> Div {
    let mut header = div()
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
                .font_family(mono_font.clone())
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(format!("style.toml · button.elevation_rules[style={style_key}]")),
        );

    if let Some(token) = token {
        header = header.child(
            div()
                .font_family(mono_font)
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(format!("catalog · --{token}")),
        );
    }

    header
}

fn render_typography_detail(
    selection: &InspectFieldSelection,
    field: &InspectTypographyData,
    preview_button: impl Fn(ControlSize) -> Entity<Button>,
    look: &Arc<ShadcnLook>,
    chrome: LumaChrome,
    body: &LumaTextStyle,
    mono: &LumaTextStyle,
    mono_font: SharedString,
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
) -> AnyElement {
    let font_family_property = field.properties.iter().find(|property| property.name == "font family");
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
                .child("Typography"),
        );

    for property in &field.properties {
        values_card = values_card.child(render_typography_property_row(
            property,
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

    div()
        .id("button-inspector-detail")
        .h_full()
        .min_w(px(0.0))
        .overflow_y_scroll()
        .flex()
        .flex_col()
        .gap(px(layout::SECTION_GAP))
        .p(px(layout::PANEL_PADDING))
        .child(typography_field_header(
            selection,
            font_family_property.map(|property| property.source.clone()),
            font_family_property.and_then(|property| property.provenance.clone()),
            chrome.title_text,
            chrome.muted_text,
            body,
            mono,
            mono_font.clone(),
        ))
        .child(
            div()
                .w_full()
                .border_1()
                .border_color(chrome.border)
                .rounded(px(layout::SWATCH_RADIUS))
                .bg(chrome.panel_background)
                .p(px(layout::CARD_PADDING))
                .flex()
                .flex_col()
                .gap(px(layout::CARD_GAP))
                .child(
                    div()
                        .text_size(px(body.size))
                        .line_height(px(body.line_height))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(chrome.muted_text)
                        .child("Type sample"),
                )
                .child(
                    div()
                        .font_family(field.font_family.clone())
                        .text_size(px(field.font_size))
                        .line_height(px(field.line_height))
                        .font_weight(field.font_weight)
                        .text_color(chrome.title_text)
                        .child("Button"),
                )
                .child(
                    div()
                        .text_size(px(mono.size))
                        .line_height(px(mono.line_height))
                        .text_color(chrome.muted_text)
                        .child("Label size, weight, and line-height are SDK scaffold until wired to theme tokens."),
                ),
        )
        .child(values_card)
        .child(render_layout_size_preview(
            preview_button(ControlSize::Md),
            chrome.border,
            chrome.panel_background,
            chrome.muted_text,
            body,
            ControlSize::Md,
        ))
        .into_any_element()
}

fn typography_field_header(
    selection: &InspectFieldSelection,
    source: Option<SharedString>,
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
            .child("Label · Text"),
    );

    if let Some(source) = source {
        header = header.child(
            div()
                .font_family(mono_font.clone())
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(source),
        );
    }

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

fn render_typography_property_row(
    property: &InspectTypographyPropertyData,
    border: gpui::Hsla,
    muted_text: gpui::Hsla,
    label_size: f32,
    label_line_height: f32,
    mono_size: f32,
    mono_line_height: f32,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
) -> impl IntoElement {
    let value = property.value.clone();
    let value_for_copy = value.to_string();
    let row_id = property.name.replace(' ', "-");

    div()
        .id(format!("button-inspector-typography-{row_id}"))
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
                        .child(value.clone()),
                )
                .child(render_copy_icon_button(
                    look,
                    format!("button-inspector-typography-copy-{row_id}").into(),
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
                .child(format!("{} · Text", control_size_label(size))),
        )
}

fn render_layout_property_row(
    property: &InspectMetricPropertyData,
    border: gpui::Hsla,
    muted_text: gpui::Hsla,
    label_size: f32,
    label_line_height: f32,
    mono_size: f32,
    mono_line_height: f32,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut Context<ButtonInspectorDetail>,
) -> impl IntoElement {
    let value = property.value.clone();
    let value_for_copy = value.to_string();
    let row_id = property.name.replace(' ', "-");

    div()
        .id(format!("button-inspector-layout-{row_id}"))
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
                    format!("button-inspector-layout-copy-{row_id}").into(),
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

fn render_layout_size_preview(
    button: Entity<Button>,
    border: gpui::Hsla,
    panel_background: gpui::Hsla,
    muted_text: gpui::Hsla,
    body: &LumaTextStyle,
    size: ControlSize,
) -> impl IntoElement {
    div()
        .w_full()
        .border_1()
        .border_color(border)
        .rounded(px(layout::SWATCH_RADIUS))
        .bg(panel_background)
        .p(px(layout::CARD_PADDING))
        .flex()
        .flex_col()
        .items_start()
        .gap(px(layout::CARD_GAP))
        .child(
            div()
                .text_size(px(body.size))
                .line_height(px(body.line_height))
                .font_weight(FontWeight::MEDIUM)
                .text_color(muted_text)
                .child("Primary · Text"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .w(px(28.0))
                        .flex_shrink_0()
                        .text_size(px(11.0))
                        .line_height(px(15.0))
                        .text_color(muted_text)
                        .child(control_size_label(size)),
                )
                .child(button),
        )
}

fn preview_button_index(size: ControlSize) -> usize {
    match size {
        ControlSize::Sm => 0,
        ControlSize::Md => 1,
        ControlSize::Lg => 2,
    }
}

fn control_size_label(size: ControlSize) -> &'static str {
    match size {
        ControlSize::Sm => "Sm",
        ControlSize::Md => "Md",
        ControlSize::Lg => "Lg",
    }
}

fn field_header(
    selection: &InspectFieldSelection,
    size: Option<ControlSize>,
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

    if let Some(size) = size {
        header = header.child(
            div()
                .font_family(mono_font.clone())
                .text_size(px(mono.size))
                .line_height(px(mono.line_height))
                .text_color(muted_text)
                .child(format!("{} · Text", control_size_label(size))),
        );
    }

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
    look: &Arc<ShadcnLook>,
    window: &mut Window,
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
        .child(render_copy_icon_button(
            look,
            format!("button-inspector-copy-{}", row.label.to_ascii_lowercase()).into(),
            value.to_string(),
            window,
            cx,
        ))
}
