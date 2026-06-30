use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma_look_shadcn::{LumaTypographyExt, ShadcnLook, ShadcnTextSize};

use super::activity::{ZoneEntry, ZoneTimeProfile};
use super::ride_summary::format_duration;

pub fn render_zones_tab(
    profile: &ZoneTimeProfile,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let chrome = look.chrome();
    let section_title_style = look.typography_scale(ShadcnTextSize::Sm);
    let zone_title_style = look.typography_scale(ShadcnTextSize::Xs);
    let detail_style = look.typography_scale(ShadcnTextSize::Xs);
    let track_color = look.token_color("muted").unwrap_or(chrome.border);

    let mut sections = Vec::new();
    if !profile.heart_rate_zones.is_empty() {
        sections.push(render_zone_section(
            "Heart Rate Zones",
            &profile.heart_rate_zones,
            section_title_style,
            zone_title_style,
            detail_style,
            chrome.title_text,
            chrome.muted_text,
            track_color,
        ));
    }
    if !profile.power_zones.is_empty() {
        sections.push(render_zone_section(
            "Power Zones",
            &profile.power_zones,
            section_title_style,
            zone_title_style,
            detail_style,
            chrome.title_text,
            chrome.muted_text,
            track_color,
        ));
    }

    if sections.is_empty() {
        return look
            .card("graph-viz-zones-empty")
            .title("Time In Zones")
            .description("No zone time data found in this activity.")
            .elevated(false)
            .render(window, cx)
            .into_any_element();
    }

    div()
        .id("graph-viz-zones-tab")
        .w_full()
        .flex()
        .flex_col()
        .gap(px(24.0))
        .children(sections)
        .into_any_element()
}

fn render_zone_section(
    title: &'static str,
    zones: &[ZoneEntry],
    section_title_style: gpui_luma::theme::LumaTextStyle,
    zone_title_style: gpui_luma::theme::LumaTextStyle,
    detail_style: gpui_luma::theme::LumaTextStyle,
    title_color: gpui::Hsla,
    detail_color: gpui::Hsla,
    track_color: gpui::Hsla,
) -> AnyElement {
    let rows: Vec<_> = zones
        .iter()
        .rev()
        .map(|zone| render_zone_row(zone, zone_title_style, detail_style, title_color, detail_color, track_color))
        .collect();

    div()
        .id(SharedString::from(format!("graph-viz-zones-{title}")))
        .w_full()
        .flex()
        .flex_col()
        .gap(px(12.0))
        .child(
            div()
                .typography_style(section_title_style)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title),
        )
        .child(div().w_full().flex().flex_col().gap(px(10.0)).children(rows))
        .into_any_element()
}

fn render_zone_row(
    zone: &ZoneEntry,
    zone_title_style: gpui_luma::theme::LumaTextStyle,
    detail_style: gpui_luma::theme::LumaTextStyle,
    title_color: gpui::Hsla,
    detail_color: gpui::Hsla,
    track_color: gpui::Hsla,
) -> AnyElement {
    let fill_pct = (zone.fill_fraction * 100.0).clamp(0.0, 100.0);
    let percent_label = format!("{}%", fill_pct.round() as u32);
    let time_label = format_duration(zone.time_seconds);

    div()
        .id(SharedString::from(format!("graph-viz-zone-{}-{}", zone.label, zone.zone)))
        .w_full()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .flex()
                .flex_row()
                .items_baseline()
                .gap(px(6.0))
                .child(
                    div()
                        .typography_style(zone_title_style)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(title_color)
                        .child(format!("Zone {}", zone.zone)),
                )
                .child(
                    div()
                        .typography_style(detail_style)
                        .text_color(detail_color)
                        .child(format!("{} • {}", zone.range_label, zone.label)),
                ),
        )
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h(px(8.0))
                        .rounded(px(4.0))
                        .bg(track_color)
                        .overflow_hidden()
                        .child(div().h_full().w(relative_width(fill_pct)).rounded(px(4.0)).bg(zone.color)),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(8.0))
                        .flex_shrink_0()
                        .child(div().typography_style(detail_style).text_color(title_color).child(time_label))
                        .child(
                            div()
                                .typography_style(detail_style)
                                .text_color(detail_color)
                                .min_w(px(32.0))
                                .text_align(gpui::TextAlign::Right)
                                .child(percent_label),
                        ),
                ),
        )
        .into_any_element()
}

fn relative_width(percentage: f32) -> gpui::Length {
    gpui::Length::Definite(gpui::relative(percentage / 100.0))
}
