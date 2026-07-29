use gpui::{Hsla, hsla};
use gpui_luma::controls::color::color_arc::visual::default_color_arc_visual;
use gpui_luma::controls::color::color_ring::visual::default_color_ring_visual;
use gpui_luma::controls::color::color_slider::color_thumb::ThumbStyle;
use gpui_luma::controls::color::color_slider::visual::default_color_slider_visual;
use gpui_luma::controls::color::style::ColorControlTheme;
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn::sync_color_control_theme;

use super::schema::{InspectColorRow, InspectPropertyRow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorChromeProfile {
    SliderTrack,
    Ring,
    Arc,
    Field,
    Swatch,
    Thumb,
}

pub const COLOR_SLIDER_CHROME_PROFILES: &[ColorChromeProfile] =
    &[ColorChromeProfile::SliderTrack, ColorChromeProfile::Thumb, ColorChromeProfile::Swatch];

pub const COLOR_FIELD_CHROME_PROFILES: &[ColorChromeProfile] =
    &[ColorChromeProfile::Field, ColorChromeProfile::Thumb, ColorChromeProfile::Swatch];

pub const COLOR_RING_CHROME_PROFILES: &[ColorChromeProfile] = &[ColorChromeProfile::Ring, ColorChromeProfile::Thumb];

pub const COLOR_ARC_CHROME_PROFILES: &[ColorChromeProfile] = &[ColorChromeProfile::Arc, ColorChromeProfile::Thumb];

#[derive(Clone, Debug)]
pub struct ColorChromeSection {
    pub title: &'static str,
    pub note: Option<&'static str>,
    pub color_rows: Vec<InspectColorRow>,
    pub property_rows: Vec<InspectPropertyRow>,
}

pub fn resolve_color_chrome_sections(look: &ShadcnLook, profiles: &[ColorChromeProfile]) -> Vec<ColorChromeSection> {
    sync_color_control_theme(look);
    let theme = active_theme(look);
    let slider_enabled = default_color_slider_visual(true);
    let slider_disabled = default_color_slider_visual(false);
    let ring_enabled = default_color_ring_visual(true);
    let ring_disabled = default_color_ring_visual(false);
    let arc_disabled = default_color_arc_visual(false);

    let mut sections = vec![shared_theme_section(theme, look)];

    for profile in profiles {
        match profile {
            ColorChromeProfile::SliderTrack => {
                sections.push(slider_track_section(slider_enabled, slider_disabled, theme))
            }
            ColorChromeProfile::Ring => {
                sections.push(ring_section(ring_enabled, ring_disabled, theme));
            }
            ColorChromeProfile::Arc => sections.push(arc_section(arc_disabled, theme)),
            ColorChromeProfile::Field => sections.push(field_section(theme)),
            ColorChromeProfile::Swatch => sections.push(swatch_section(theme)),
            ColorChromeProfile::Thumb => sections.push(thumb_section()),
        }
    }

    sections
}

fn active_theme(look: &ShadcnLook) -> ColorControlTheme {
    let chrome = look.chrome();
    ColorControlTheme::new(chrome.border, chrome.panel_background, matches!(look.mode(), ThemeMode::Dark))
}

fn shared_theme_section(theme: ColorControlTheme, look: &ShadcnLook) -> ColorChromeSection {
    ColorChromeSection {
        title: "Color control theme",
        note: Some("Synced from ShadcnLook::chrome() via sync_color_control_theme."),
        color_rows: vec![
            hsla_row("border", theme.border, "ShadcnLook::chrome().border", "chrome.border"),
            hsla_row(
                "background",
                theme.background,
                "ShadcnLook::chrome().panel_background",
                "chrome.panel_background",
            ),
        ],
        property_rows: vec![
            property_row(
                "dark mode",
                if theme.is_dark() { "yes" } else { "no" },
                "ShadcnLook::mode()",
                Some(format!("Theme mode resolves to {:?}", look.mode())),
            ),
            property_row(
                "sync entrypoint",
                "sync_color_control_theme",
                "look-shadcn/context.rs",
                Some("Called inside with_look before color controls render.".to_string()),
            ),
        ],
    }
}

fn slider_track_section(
    enabled: gpui_luma::controls::color::color_slider::visual::ColorSliderVisual,
    disabled: gpui_luma::controls::color::color_slider::visual::ColorSliderVisual,
    theme: ColorControlTheme,
) -> ColorChromeSection {
    let blocked_dark = hsla(0., 0., 0.18, 1.);
    let blocked_light = hsla(0., 0., 0.72, 1.);

    ColorChromeSection {
        title: "Color slider track chrome",
        note: Some("Track fill gradients come from the active color model — not theme tokens."),
        color_rows: vec![
            hsla_row("track border", enabled.border, "ColorControlTheme.border", "active_color_control_theme().border"),
            hsla_row(
                "disabled overlay",
                disabled.disabled_overlay,
                "ColorControlTheme.background @ 45%",
                "theme.background.opacity(0.45) when disabled",
            ),
            hsla_row(
                "blocked overlay (dark)",
                blocked_dark,
                "fixed dark-mode tone",
                "hsla(0, 0, 0.18, 1) when theme.is_dark()",
            ),
            hsla_row(
                "blocked overlay (light)",
                blocked_light,
                "fixed light-mode tone",
                "hsla(0, 0, 0.72, 1) when theme is light",
            ),
        ],
        property_rows: vec![property_row(
            "blocked overlay mode",
            if theme.is_dark() { "dark preset" } else { "light preset" },
            "default_color_slider_visual",
            Some("Blocked segments pick the dark or light overlay literal.".to_string()),
        )],
    }
}

fn ring_section(
    enabled: gpui_luma::controls::color::color_ring::visual::ColorRingVisual,
    disabled: gpui_luma::controls::color::color_ring::visual::ColorRingVisual,
    _theme: ColorControlTheme,
) -> ColorChromeSection {
    ColorChromeSection {
        title: "Color ring chrome",
        note: Some("Ring hue/saturation/lightness fills are domain rendering — not theme."),
        color_rows: vec![
            hsla_row("ring border", enabled.border, "ColorControlTheme.border", "active_color_control_theme().border"),
            hsla_row(
                "disabled overlay",
                disabled.disabled_overlay,
                "ColorControlTheme.background @ 45%",
                "theme.background.opacity(0.45) when disabled",
            ),
            hsla_row(
                "center hole",
                enabled.center_hole,
                "ColorControlTheme.background",
                "active_color_control_theme().background",
            ),
        ],
        property_rows: Vec::new(),
    }
}

fn arc_section(
    disabled: gpui_luma::controls::color::color_arc::visual::ColorArcVisual,
    theme: ColorControlTheme,
) -> ColorChromeSection {
    ColorChromeSection {
        title: "Color arc chrome",
        note: Some("Arc channel fills are domain rendering — not theme."),
        color_rows: vec![
            hsla_row(
                "arc border",
                theme.border,
                "ColorControlTheme.border",
                "active_color_control_theme().border via raster/track path",
            ),
            hsla_row(
                "disabled overlay",
                disabled.disabled_overlay,
                "ColorControlTheme.background @ 45%",
                "theme.background.opacity(0.45) when disabled",
            ),
        ],
        property_rows: Vec::new(),
    }
}

fn field_section(theme: ColorControlTheme) -> ColorChromeSection {
    ColorChromeSection {
        title: "Color field chrome",
        note: Some("Field domain fills (SV plane, wheel, triangle) are model rendering — not theme."),
        color_rows: vec![
            hsla_row(
                "field border",
                theme.border,
                "ColorControlTheme.border",
                "cx.theme().border in ColorField layout",
            ),
            hsla_row(
                "outside / corner occluder",
                theme.background,
                "ColorControlTheme.background",
                "outside_color + paint_corner_occluder",
            ),
        ],
        property_rows: vec![property_row(
            "disabled opacity",
            "0.5",
            "ColorField render",
            Some("cursor_not_allowed().opacity(0.5) when state.disabled".to_string()),
        )],
    }
}

fn swatch_section(theme: ColorControlTheme) -> ColorChromeSection {
    let (checker_a, checker_b) = if theme.is_dark() {
        (hsla(0., 0., 0.1, 1.), hsla(0., 0., 0.13, 1.))
    } else {
        (hsla(0., 0., 1.0, 1.), hsla(0., 0., 0.95, 1.))
    };

    ColorChromeSection {
        title: "Color swatch chrome",
        note: Some("Swatch fill color is the selected sample — only border and checkerboard come from theme."),
        color_rows: vec![
            hsla_row("border", theme.border, "ColorControlTheme.border", "cx.theme().border"),
            hsla_row(
                "checkerboard base",
                checker_a,
                "mode-derived literal",
                "dark: hsla(0,0,0.10) / light: hsla(0,0,1.0)",
            ),
            hsla_row(
                "checkerboard accent",
                checker_b,
                "mode-derived literal",
                "dark: hsla(0,0,0.13) / light: hsla(0,0,0.95)",
            ),
        ],
        property_rows: vec![property_row(
            "checkerboard",
            "opt-in per swatch",
            "ColorSwatch::checkerboard",
            Some("Painted only when color.a < 0.999".to_string()),
        )],
    }
}

fn thumb_section() -> ColorChromeSection {
    let thumb = ThumbStyle::new(gpui::px(16.0));

    ColorChromeSection {
        title: "Color thumb chrome",
        note: Some("ThumbStyle defaults are hard-coded — not derived from Shadcn tokens today."),
        color_rows: vec![
            hsla_row("outer border", thumb.border_outer, "ThumbStyle default", "black()"),
            hsla_row("inner border", thumb.border_inner, "ThumbStyle default", "white()"),
        ],
        property_rows: vec![
            property_row("border width", format_px(thumb.border_width), "ThumbStyle", None),
            property_row("inner inset", format_px(thumb.inner_inset), "ThumbStyle", None),
            property_row("square radius", format_px(thumb.square_radius), "ThumbStyle", None),
            property_row("active shadow", if thumb.active_shadow { "yes" } else { "no" }, "ThumbStyle", None),
            property_row("show outer border", if thumb.show_outer_border { "yes" } else { "no" }, "ThumbStyle", None),
            property_row("show inner border", if thumb.show_inner_border { "yes" } else { "no" }, "ThumbStyle", None),
        ],
    }
}

fn hsla_row(label: &'static str, value: Hsla, source: &'static str, detail: &'static str) -> InspectColorRow {
    InspectColorRow { label, value, source: source.to_owned(), detail: Some(detail.to_owned()) }
}

fn property_row(label: &str, value: impl ToString, source: &str, detail: Option<String>) -> InspectPropertyRow {
    InspectPropertyRow { label: label.to_owned(), value: value.to_string(), source: source.to_owned(), detail }
}

fn format_px(value: gpui::Pixels) -> String {
    format!("{:.1}px", value.as_f32())
}
