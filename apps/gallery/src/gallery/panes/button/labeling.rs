use gpui::{AnyElement, Hsla, div, prelude::*, px, svg};

/// Build an SVG string for a single rotated section label.
///
/// This is useful when you want to generate label assets from arbitrary text
/// while keeping the same visual style used by the button matrix rail.
///
/// The returned SVG rotates the full text node (`-90deg`) inside the SVG.
#[allow(dead_code)]
pub(in crate::gallery) fn rotated_label_svg_string(text: &str) -> String {
    let escaped = escape_svg_text(text);
    let estimated_height = (text.chars().count() as i32 * 10 + 24).max(120);

    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"{estimated_height}\" viewBox=\"0 0 24 {estimated_height}\" fill=\"none\">\n  <g transform=\"translate(18 {y}) rotate(-90)\">\n    <text x=\"0\" y=\"0\" font-family=\"sans-serif\" font-size=\"16\" font-weight=\"500\" fill=\"currentColor\">{escaped}</text>\n  </g>\n</svg>\n",
        y = estimated_height - 10
    )
}

/// Renders a reusable left-side rail label used by matrix-style gallery layouts.
///
/// ## How it works
///
/// - The visible label text is sourced from a small SVG asset per section.
/// - The text is pre-rotated inside the SVG file (`-90deg`) so we don't rely on
///   runtime text transforms on a regular text node.
/// - This function composes:
///   1. the rotated SVG label,
///   2. a vertical divider line,
///   into one narrow "rail" element.
///
/// ## Reuse notes
///
/// - Keep the rail width and minimum height consistent across rows/sections so
///   the matrix body aligns.
/// - Add new labels by creating another SVG and extending
///   `label_asset_path_for_section`.
/// - This helper intentionally stays layout-level (gallery-side) and does not
///   require SDK changes.
pub(in crate::gallery::panes) fn render_vertical_section_rail(
    section_label: &'static str,
    label_color: Hsla,
) -> AnyElement {
    div()
        .w(px(28.0))
        .min_h(px(188.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(render_rotated_section_label(section_label, label_color))
        .child(div().w(px(1.0)).h_full().bg(label_color))
        .into_any_element()
}

fn render_rotated_section_label(section_label: &'static str, label_color: Hsla) -> AnyElement {
    svg()
        .path(label_asset_path_for_section(section_label))
        .w(px(20.0))
        .h(px(104.0))
        .text_color(label_color)
        .into_any_element()
}

fn label_asset_path_for_section(section_label: &'static str) -> &'static str {
    match section_label {
        "Prominent" => "assets/labels/prominent-label.svg",
        "Standard" => "assets/labels/standard-label.svg",
        "Subtle" => "assets/labels/subtle-label.svg",
        "Ghost" => "assets/labels/ghost-label.svg",
        "Selected" => "assets/labels/selected-label.svg",
        "Unselected" => "assets/labels/unselected-label.svg",
        _ => "assets/labels/default-label.svg",
    }
}

#[allow(dead_code)]
fn escape_svg_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
