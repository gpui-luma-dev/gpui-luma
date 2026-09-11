//! Style Guide · Buttons
//!
//! In-section tabs:
//! - **Template Preview** — variants × interaction states (`table` + `states`)
//! - **Colors** — accents × variants (default + high-contrast pair per cell)
//! - **All Sizes** — per-variant size × radius grids

use std::sync::Arc;

use gpui::{AnyElement, App, Entity, Hsla, IntoElement, SharedString, Window, div, prelude::*, px};
use luma::controls::button::{ButtonRenderModel, ButtonTemplate, ControlPresenter};
use luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use luma::controls::tabs::Tabs;
use luma::infra::icon::lucide_icon;
use luma::{hstack, vstack};
use luma_look_radix::{
    RadixAccent, RadixButtonPaint, RadixButtonSize, RadixButtonVariant, RadixGray, RadixLook, RadixLookControlExt,
    RadixRadius, ScaleFamily, button_look_for,
};
use lucide_svg_static::Icon as LucideIcon;

use super::matrix_grid::{
    COL_GAP, centered, column_header, corner_label, empty_corner, equal_data_columns, fixed_grid, preview_tabbed,
    row_label,
};
use super::states::{self, StateSample};
use super::table::{TableRow, TableStyle};

const STATE_COLUMN_WIDTH: f32 = 152.0;
const ROW_LABEL_WIDTH: f32 = 88.0;
const COLOR_COLUMN_WIDTH: f32 = 176.0;
const SIZE_COLUMN_WIDTH: f32 = 120.0;
const SIZE_ROW_GAP: f32 = 20.0;
const SIZE_TABLE_GAP: f32 = 40.0;

pub struct VariantDef {
    pub id: &'static str,
    pub label: &'static str,
    description: &'static str,
    pub variant: RadixButtonVariant,
}

/// The Radix Themes variant scheme shown in the Style Guide (Classic deferred until elevation lands).
pub const RADIX_VARIANTS: [VariantDef; 5] = [
    VariantDef {
        id: "solid",
        label: "Solid",
        description: "High emphasis actions",
        variant: RadixButtonVariant::Solid,
    },
    VariantDef { id: "soft", label: "Soft", description: "Lower emphasis actions", variant: RadixButtonVariant::Soft },
    VariantDef {
        id: "surface",
        label: "Surface",
        description: "Tinted panel with an edge",
        variant: RadixButtonVariant::Surface,
    },
    VariantDef {
        id: "outline",
        label: "Outline",
        description: "Subtle secondary controls",
        variant: RadixButtonVariant::Outline,
    },
    VariantDef {
        id: "ghost",
        label: "Ghost",
        description: "Quiet utility actions",
        variant: RadixButtonVariant::Ghost,
    },
];

pub fn tabbed(
    look: &Arc<RadixLook>,
    preview_tabs: Entity<Tabs>,
    fg: Hsla,
    muted: Hsla,
    border: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let active = preview_tabs.read(cx).active_id().cloned().unwrap_or_else(|| SharedString::from("template-preview"));

    let body = match active.as_ref() {
        "colors" => colors_matrix(look, fg, muted, window, cx),
        "all-sizes" => sizes_matrix(look, fg, muted, window, cx),
        _ => matrix(look, &RADIX_VARIANTS, fg, muted, window, cx),
    };

    preview_tabbed(preview_tabs, border, body)
}

/// Size 1–4 × radius none…full for every Radix button variant.
pub fn sizes_matrix(look: &Arc<RadixLook>, fg: Hsla, muted: Hsla, window: &mut Window, cx: &mut App) -> AnyElement {
    vstack! { gap=SIZE_TABLE_GAP; }
        .children(RADIX_VARIANTS.iter().map(|def| sizes_variant_table(look, def, fg, muted, window, cx)))
        .into_any_element()
}

fn sizes_variant_table(
    look: &Arc<RadixLook>,
    def: &VariantDef,
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let template = look.button_template(def.variant, RadixButtonPaint::accent());
    let mut grid = fixed_grid(
        equal_data_columns(ROW_LABEL_WIDTH, SIZE_COLUMN_WIDTH, RadixRadius::ALL.len()),
        COL_GAP,
        SIZE_ROW_GAP,
    );

    grid = grid.child(corner_label(def.label, muted), 0, 0);
    for (col, radius) in RadixRadius::ALL.iter().enumerate() {
        grid = grid.child(column_header(radius.label(), muted), 0, col + 1);
    }

    for (row, size) in RadixButtonSize::ALL.iter().enumerate() {
        let row = row + 1;
        grid = grid.child(row_label(size.label(), fg), row, 0);
        for (col, radius) in RadixRadius::ALL.iter().enumerate() {
            grid = grid.child(sizes_cell(look, &template, def, *size, *radius, window, cx), row, col + 1);
        }
    }

    grid.into_any_element()
}

fn sizes_cell(
    look: &Arc<RadixLook>,
    template: &Arc<dyn ButtonTemplate<()>>,
    def: &VariantDef,
    size: RadixButtonSize,
    radius: RadixRadius,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let id = format!("sizes-{}-{}-{}", def.id, size.as_str(), radius.as_str());
    let model = ButtonRenderModel {
        id: SharedString::from(id),
        content: next_content(),
        role: ButtonFamilyRole::Text,
        size: size.control_size(),
        look: Some(button_look_for(Arc::clone(look), def.variant, RadixButtonPaint::accent(), size, radius)),
        ..Default::default()
    };

    centered(template.render(&model, window, cx))
}

/// Accents × variants: each cell pairs default + high-contrast on that accent.
pub fn colors_matrix(
    base_look: &Arc<RadixLook>,
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let mut grid =
        fixed_grid(equal_data_columns(ROW_LABEL_WIDTH, COLOR_COLUMN_WIDTH, RADIX_VARIANTS.len()), COL_GAP, COL_GAP);

    grid = grid.child(empty_corner(), 0, 0);
    for (col, def) in RADIX_VARIANTS.iter().enumerate() {
        grid = grid.child(column_header(def.label, muted), 0, col + 1);
    }

    for (row, accent) in RadixAccent::ALL.iter().enumerate() {
        let row = row + 1;
        let row_look = Arc::new(base_look.fork());
        row_look.set_palettes(*accent, RadixGray::Auto);
        let accent_id = accent.as_str();
        let label = super::palettes::title_case(accent_id);

        grid = grid.child(row_label(label, fg), row, 0);
        for (col, def) in RADIX_VARIANTS.iter().enumerate() {
            grid = grid.child(colors_variant_cell(&row_look, accent_id, def, window, cx), row, col + 1);
        }
    }

    grid.into_any_element()
}

fn colors_variant_cell(
    look: &Arc<RadixLook>,
    accent_id: &str,
    def: &VariantDef,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let default = look.button_template(def.variant, RadixButtonPaint::accent());
    let high_contrast = look.button_template(def.variant, RadixButtonPaint::accent().high_contrast());

    centered(
        hstack! { gap=8 align=center; }
            .child(preview_next(&default, &format!("colors-{accent_id}-{}-default", def.id), window, cx))
            .child(preview_next(&high_contrast, &format!("colors-{accent_id}-{}-hc", def.id), window, cx)),
    )
}

fn preview_next(template: &Arc<dyn ButtonTemplate<()>>, id: &str, window: &mut Window, cx: &mut App) -> AnyElement {
    let model = ButtonRenderModel {
        id: SharedString::from(id),
        content: next_content(),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        ..Default::default()
    };
    template.render(&model, window, cx).into_any_element()
}

fn next_content() -> ControlPresenter<ButtonRenderModel<()>> {
    Arc::new(|model, _| {
        let fg = model.resolved_look.as_ref().map(|look| look.foreground).unwrap_or_else(gpui::white);
        let icon = model.resolved_look.as_ref().map(|look| look.icon_size).unwrap_or(14.0);
        let gap = model.resolved_look.as_ref().map(|look| look.gap).unwrap_or(8.0);
        div()
            .flex()
            .items_center()
            .gap(px(gap))
            .child("Next")
            .child(lucide_icon(LucideIcon::ArrowRight, fg, icon))
            .into_any_element()
    })
}

pub fn matrix(
    look: &Arc<RadixLook>,
    variants: &[VariantDef],
    fg: Hsla,
    muted: Hsla,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let samples = states::samples();
    let style = TableStyle { state_column_width: STATE_COLUMN_WIDTH, ..TableStyle::new(fg, muted) };
    let accent = super::palettes::title_case(look.palette_label(ScaleFamily::Color));

    let headers = samples.iter().map(|sample| states::header_cell(sample, muted)).collect();
    let rows = variants
        .iter()
        .map(|def| {
            // Forced states resolve through the look's own palette.
            let template = look.button_template(def.variant, RadixButtonPaint::accent());
            TableRow {
                label: SharedString::from(def.label),
                description: SharedString::from(description(def, &accent)),
                cells: samples.iter().map(|sample| state_cell(&template, def.id, sample, window, cx)).collect(),
            }
        })
        .collect();

    super::table::render(&style, "VARIANTS", headers, rows)
}

fn description(def: &VariantDef, accent: &str) -> String {
    match def.variant {
        RadixButtonVariant::Solid => format!("{accent} 9 face"),
        RadixButtonVariant::Soft => format!("{accent} 3 tint"),
        RadixButtonVariant::Surface => format!("{accent} panel with an edge"),
        _ => def.description.to_string(),
    }
}

fn state_cell(
    template: &Arc<dyn ButtonTemplate<()>>,
    variant_id: &str,
    sample: &StateSample,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let model = ButtonRenderModel {
        id: SharedString::from(format!("style-guide-button-{variant_id}-{}", sample.id)),
        content: Arc::new(|_, _| div().child("Button").into_any_element()),
        role: ButtonFamilyRole::Text,
        size: ButtonSize::Md,
        state: sample.state,
        ..Default::default()
    };

    centered(template.render(&model, window, cx))
}
