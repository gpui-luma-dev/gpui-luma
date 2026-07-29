use gpui_luma::theme::ControlSize;
use gpui_luma_look_shadcn::ShadcnLook;
use gpui_luma_look_shadcn_inspect::{
    AutocompleteChromeInspectPalette, FloatingMenuInspectPalette, ShadcnInspect, TextFieldInspectPalette,
};

use super::provenance::color_row;
use super::schema::InspectColorRow;

pub fn textfield_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    textfield_color_rows_prefixed("field", palette)
}

pub fn textfield_color_rows_prefixed(prefix: &'static str, palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    match prefix {
        "textfield" => textfield_input_color_rows(palette),
        "field" => textfield_field_color_rows(palette),
        _ => textfield_field_color_rows(palette),
    }
}

fn textfield_field_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = vec![
        color_row("background", &palette.background),
        color_row("foreground", &palette.foreground),
        color_row("border", &palette.border),
        color_row("placeholder", &palette.placeholder),
        color_row("icon", &palette.icon),
        color_row("selection background", &palette.selection_background),
        color_row("selection foreground", &palette.selection_foreground),
        color_row("caret", &palette.caret),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("focus ring", focus_ring));
    }
    rows
}

fn textfield_input_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = vec![
        color_row("textfield background", &palette.background),
        color_row("textfield foreground", &palette.foreground),
        color_row("textfield border", &palette.border),
        color_row("textfield placeholder", &palette.placeholder),
        color_row("textfield icon", &palette.icon),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("textfield focus ring", focus_ring));
    }
    rows
}

pub fn textfield_trigger_color_rows(palette: &TextFieldInspectPalette) -> Vec<InspectColorRow> {
    let mut rows = vec![
        color_row("trigger background", &palette.background),
        color_row("trigger foreground", &palette.foreground),
        color_row("trigger border", &palette.border),
        color_row("trigger placeholder", &palette.placeholder),
        color_row("trigger icon", &palette.icon),
    ];
    if let Some(focus_ring) = &palette.focus_ring {
        rows.push(color_row("trigger focus ring", focus_ring));
    }
    rows
}

pub fn floating_menu_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_floating_menu_color_palette(ControlSize::Md);
    floating_menu_palette_rows(&palette)
}

pub fn floating_menu_palette_rows(palette: &FloatingMenuInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("menu background", &palette.background),
        color_row("menu foreground", &palette.foreground),
        color_row("menu border", &palette.border),
        color_row("menu item hover background", &palette.item_hover_background),
        color_row("menu item hover foreground", &palette.item_hover_foreground),
        color_row("menu item disabled foreground", &palette.item_disabled_foreground),
    ]
}

pub fn autocomplete_chrome_color_rows(palette: &AutocompleteChromeInspectPalette) -> Vec<InspectColorRow> {
    vec![
        color_row("chrome status", &palette.status_color),
        color_row("chrome muted text", &palette.muted_text_color),
        color_row("chrome clear icon", &palette.clear_icon_color),
        color_row("chrome clear icon hover", &palette.clear_icon_hover_color),
    ]
}

pub fn trigger_color_rows(
    background: &gpui_luma_look_shadcn::ResolvedColor,
    foreground: &gpui_luma_look_shadcn::ResolvedColor,
    border: &gpui_luma_look_shadcn::ResolvedColor,
    focus_ring: Option<&gpui_luma_look_shadcn::ResolvedColor>,
) -> Vec<InspectColorRow> {
    let mut rows = vec![
        color_row("trigger background", background),
        color_row("trigger foreground", foreground),
        color_row("trigger border", border),
    ];
    if let Some(focus_ring) = focus_ring {
        rows.push(color_row("trigger focus ring", focus_ring));
    }
    rows
}
