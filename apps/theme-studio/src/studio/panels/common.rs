use gpui::{AnyElement, FontWeight, Hsla, IntoElement, div, prelude::*, px};
use gpui_luma::theme::LumaChrome;
use gpui_luma::hstack;

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
    let r = hue_to_channel(p, q, h + 0.0);
    let g = hue_to_channel(p, q, h + 2.0);
    let b = hue_to_channel(p, q, h + 4.0);
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

pub fn format_hsla(color: Hsla) -> String {
    format!(
        "hsla({} {}% {}% / {})",
        rounded_channel(color.h * 360.0),
        rounded_channel(color.s * 100.0),
        rounded_channel(color.l * 100.0),
        compact_alpha(color.a)
    )
}

fn rounded_channel(value: f32) -> i32 {
    value.round() as i32
}

fn compact_alpha(alpha: f32) -> String {
    if (alpha - 1.0).abs() < 0.001 {
        "1".to_string()
    } else {
        format!("{alpha:.2}")
    }
}

pub fn panel_drag_handle(chrome: LumaChrome) -> gpui::Div {
    div()
        .w_full()
        .h(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_t(px(10.0))
        .bg(gpui::hsla(0.0, 0.0, 1.0, 0.05))
        .text_size(px(11.0))
        .text_color(chrome.muted_text)
        .cursor_pointer()
        .child("⋮⋮")
}

pub fn card(width: f32, border: Hsla, background: Hsla, content: impl IntoElement) -> gpui::Div {
    div()
        .w(px(width))
        .max_w_full()
        .overflow_hidden()
        .border_1()
        .border_t_0()
        .border_color(border)
        .rounded_b(px(12.0))
        .bg(background)
        .p(px(16.0))
        .child(content)
}

pub fn card_header(title: &'static str, subtitle: &'static str, title_color: Hsla, subtitle_color: Hsla) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_size(px(16.0))
                .line_height(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title),
        )
        .child(div().text_size(px(12.0)).line_height(px(16.0)).text_color(subtitle_color).child(subtitle))
        .into_any_element()
}

pub fn or_divider(label: &'static str, border: Hsla, text: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap(px(10.0))
        .child(div().flex_1().h(px(1.0)).bg(border))
        .child(div().text_size(px(10.0)).line_height(px(12.0)).text_color(text).child(label))
        .child(div().flex_1().h(px(1.0)).bg(border))
}

pub fn avatar_circle(initials: &'static str, size: f32, bg: Hsla, fg: Hsla) -> impl IntoElement {
    div()
        .size(px(size))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(bg)
        .text_size(px(size * 0.38))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(fg)
        .child(initials)
}

pub fn message_bubble(text: &'static str, align_end: bool, bg: Hsla, fg: Hsla) -> impl IntoElement {
    let bubble = div()
        .max_w(px(220.0))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(10.0))
        .bg(bg)
        .text_size(px(12.0))
        .line_height(px(16.0))
        .text_color(fg)
        .child(text);

    if align_end {
        hstack! { justify=end; bubble }
    } else {
        div().child(bubble)
    }
}
