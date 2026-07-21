use std::sync::Arc;

use gpui::{AnyElement, App, FontWeight, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use gpui_luma::hstack;
use gpui_luma_look_shadcn::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    Sm,
    Md,
}

pub fn format_hex_color(color: Hsla) -> String {
    let (r, g, b) = hsla_to_rgb8(color);
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn hsla_to_rgb8(color: Hsla) -> (u8, u8, u8) {
    let h = color.h.fract() * 6.0;
    let s = color.s.clamp(0.0, 1.0);
    let l = color.l.clamp(0.0, 1.0);

    if s <= f32::EPSILON {
        let gray = (l * 255.0).round() as u8;
        return (gray, gray, gray);
    }

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_channel(p, q, h + 2.0);
    let g = hue_to_channel(p, q, h);
    let b = hue_to_channel(p, q, h - 2.0);
    ((r * 255.0).round() as u8, (g * 255.0).round() as u8, (b * 255.0).round() as u8)
}

fn rgb_hex(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

fn hue_to_channel(p: f32, q: f32, t: f32) -> f32 {
    let mut t = t;
    if t < 0.0 {
        t += 6.0;
    }
    if t >= 6.0 {
        t -= 6.0;
    }
    if t < 1.0 {
        p + (q - p) * t
    } else if t < 3.0 {
        q
    } else if t < 4.0 {
        p + (q - p) * (4.0 - t)
    } else {
        p
    }
}

pub fn parse_hex_color(raw: &str) -> Option<Hsla> {
    let hex = raw.trim().trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(gpui::rgb(rgb_hex(r, g, b)).into())
        }
        _ => None,
    }
}

pub fn card(
    id: impl Into<SharedString>,
    look: &Arc<ShadcnLook>,
    _width: f32,
    content: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .w_full()
        .max_w_full()
        .child(look.card(id).child_render(content).render(window, cx))
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
pub fn titled_card(
    id: impl Into<SharedString>,
    look: &Arc<ShadcnLook>,
    _width: f32,
    title: &'static str,
    subtitle: &'static str,
    content: impl Fn(&mut Window, &mut App) -> AnyElement + Send + Sync + 'static,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    div()
        .w_full()
        .max_w_full()
        .child(look.card(id).title(title).description(subtitle).child_render(content).render(window, cx))
        .into_any_element()
}

pub fn or_divider(label: &'static str, border: Hsla, text: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().flex_1().h(px(1.0)).bg(border))
        .child(div().text_xs().line_height(px(12.0)).text_color(text).child(label))
        .child(div().flex_1().h(px(1.0)).bg(border))
}

pub fn avatar(initials: &'static str, size: AvatarSize) -> impl IntoElement {
    let size_px = match size {
        AvatarSize::Sm => 32.0,
        AvatarSize::Md => 36.0,
    };

    div()
        .size(px(size_px))
        .flex()
        .items_center()
        .justify_center()
        .font_weight(FontWeight::SEMIBOLD)
        .bg_cn(ShadcnToken::Muted)
        .text_cn(ShadcnToken::MutedForeground)
        .font_cn(ShadcnFont::Sans)
        .rounded_full()
        .text_size(px(size_px * 0.38))
        .child(initials)
}

pub fn message_bubble(text: &'static str, align_end: bool, bg: Hsla, fg: Hsla) -> impl IntoElement {
    let bubble = div()
        .max_w(px(220.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(10.0))
        .bg(bg)
        .text_sm()
        .line_height(px(16.0))
        .text_color(fg)
        .child(text);

    if align_end {
        hstack! { justify=end; bubble }
    } else {
        div().child(bubble)
    }
}
