use gpui::{Context, Div, FontWeight, IntoElement, MouseButton, div, hsla, prelude::*, px};
use gpui_luma::controls::switch::SwitchScale;
use gpui_luma::theme::LumaChrome;

use super::app::ThemeStudioApp;
use super::export::{catalog_color_for_token, export_stylesheet, token_css_name};
use super::inspectable::{InspectableId, parts_for};
use super::panels::{format_hsla, panel_box_shadow};

pub fn render_inspector(
    this: &ThemeStudioApp,
    scale_factor: f32,
    cx: &mut Context<ThemeStudioApp>,
) -> Option<impl IntoElement + use<>> {
    let id = this.selected?;
    let chrome = this.radix_theme.chrome();
    let sans = this.radix_theme.mode_tokens().typography.font.sans.family.clone();

    let panel = inspector_panel(chrome)
        .font_family(sans)
        .child(header_row(&id.settings_title(), chrome, cx))
        .child(div().flex().flex_col().gap(px(12.0)).children({
            let mut body: Vec<gpui::AnyElement> = Vec::new();
            if let Some(scale) = scale_rows(this, id, scale_factor, chrome, cx) {
                body.push(scale.into_any_element());
            }
            body.push(token_rows(this, id, chrome).into_any_element());
            body
        }))
        .child(footer_actions(&this.export_status, chrome, cx));

    Some(panel)
}

fn inspector_panel(chrome: LumaChrome) -> Div {
    div()
        .absolute()
        .top(px(72.0))
        .right(px(20.0))
        .w(px(360.0))
        .max_h(px(720.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
        .p(px(16.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.panel_background)
        .shadow(panel_box_shadow())
}

fn header_row(title: &str, chrome: LumaChrome, cx: &mut Context<ThemeStudioApp>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(14.0))
                .line_height(px(18.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.title_text)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(chrome.muted_text)
                .cursor_pointer()
                .child("✕")
                .on_mouse_down(MouseButton::Left, cx.listener(|app, _, _, cx| app.close_inspector(cx))),
        )
}

fn scale_rows(
    this: &ThemeStudioApp,
    id: InspectableId,
    scale_factor: f32,
    chrome: LumaChrome,
    cx: &mut Context<ThemeStudioApp>,
) -> Option<impl IntoElement> {
    if id.scale_fields().is_empty() {
        return None;
    }

    let metrics = &this.radix_theme.mode_tokens().metrics;
    let base = SwitchScale::compute(this.control_size, metrics, scale_factor);
    let scale = this.overrides.effective_switch_scale(base);

    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_size(px(11.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(chrome.muted_text)
                    .child("Scale Metrics (SwitchScale)"),
            )
            .children(id.scale_fields().iter().map(|field| {
                let value = match field.key {
                    "track_width" => scale.track_width,
                    "track_height" => scale.track_height,
                    "thumb_size" => scale.thumb_size,
                    _ => 0.0,
                };
                let key = field.key.to_string();
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_size(px(11.0)).text_color(chrome.body_text).child(format!("{}:", field.label)))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .size(px(22.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(4.0))
                                    .border_1()
                                    .border_color(chrome.border)
                                    .text_size(px(12.0))
                                    .text_color(chrome.body_text)
                                    .cursor_pointer()
                                    .child("−")
                                    .on_mouse_down(MouseButton::Left, {
                                        let key = key.clone();
                                        cx.listener(move |app, _, _, cx| app.adjust_scale(id, &key, -1.0, cx))
                                    }),
                            )
                            .child(
                                div().text_size(px(11.0)).text_color(chrome.body_text).child(format!("{value:.0} px")),
                            )
                            .child(
                                div()
                                    .size(px(22.0))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(4.0))
                                    .border_1()
                                    .border_color(chrome.border)
                                    .text_size(px(12.0))
                                    .text_color(chrome.body_text)
                                    .cursor_pointer()
                                    .child("+")
                                    .on_mouse_down(MouseButton::Left, {
                                        let key = key.clone();
                                        cx.listener(move |app, _, _, cx| app.adjust_scale(id, &key, 1.0, cx))
                                    }),
                            ),
                    )
            })),
    )
}

fn token_rows(this: &ThemeStudioApp, id: InspectableId, chrome: LumaChrome) -> impl IntoElement {
    let parts = parts_for(id);

    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(chrome.muted_text)
                .child("Color Tokens"),
        )
        .children(parts.iter().map(|part| {
            let token_name = token_css_name(part.token);
            let catalog = catalog_color_for_token(&this.radix_theme, part.token);
            let effective = this
                .overrides
                .color_override(id, &token_name)
                .or(catalog)
                .unwrap_or_else(|| hsla(0.0, 0.0, 0.5, 1.0));

            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(chrome.muted_text)
                        .child(format!("{} — {}", part.part, token_name)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div().size(px(14.0)).rounded(px(3.0)).bg(effective).border_1().border_color(chrome.border),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(10.0))
                                .font_family("Monaco")
                                .text_color(chrome.body_text)
                                .child(format_hsla(effective)),
                        ),
                )
        }))
}

fn footer_actions(export_status: &str, chrome: LumaChrome, cx: &mut Context<ThemeStudioApp>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(
            div()
                .px(px(10.0))
                .py(px(6.0))
                .rounded(px(6.0))
                .bg(chrome.border)
                .text_size(px(11.0))
                .text_color(chrome.title_text)
                .cursor_pointer()
                .child("Save Adjustments")
                .on_mouse_down(MouseButton::Left, cx.listener(|app, _, _, cx| app.apply_overrides(cx))),
        )
        .child(
            div()
                .px(px(10.0))
                .py(px(6.0))
                .rounded(px(6.0))
                .border_1()
                .border_color(chrome.border)
                .text_size(px(11.0))
                .text_color(chrome.title_text)
                .cursor_pointer()
                .child("Export Theme Stylesheet")
                .on_mouse_down(MouseButton::Left, cx.listener(|app, _, _, cx| app.export_theme(cx))),
        )
        .child(div().text_size(px(10.0)).text_color(chrome.muted_text).child(export_status.to_string()))
}

pub fn run_export(app: &ThemeStudioApp) -> anyhow::Result<std::path::PathBuf> {
    export_stylesheet(&app.overrides)
}
