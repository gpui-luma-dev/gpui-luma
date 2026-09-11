//! Custom Palette tab — seed editor, scale grid, and control preview canvas.

use std::sync::Arc;

use gpui::{App, Context, Entity, Hsla, ImageSource, IntoElement, RenderImage, SharedString, div, img, prelude::*, px};
use luma::controls::button::Button;
use luma::controls::checkbox::Checkbox;
use luma::controls::popup_menu::PopupMenu;
use luma::controls::switch::Switch;
use luma::controls::textfield::TextField;
use luma::controls::toggle::Toggle;
use luma::infra::ElementExt;
use luma::{GridLayout, GridTrack, WideMiddle, WideMiddleLayout, hstack, vstack};
use luma_look_radix::{PaletteSlot, RadixLook, SCALE_LEN, ScaleFamily, SignupStage, ThemePalettes};

use crate::app::RadixStudioApp;
use crate::controls::ColorTextField;

/// Side columns stay compact; center is wider for the signup stage.
const PREVIEW_SIDE_COLUMN_MIN: f32 = 240.0;
const PREVIEW_SIDE_COLUMN_PREFERRED: f32 = 280.0;
/// Middle column hard cap in the three-column row (also signup glass width).
pub const PREVIEW_CENTER_COLUMN_MAX: f32 = 435.0;
const PREVIEW_COLUMN_GAP: f32 = 24.0;
const SIGNUP_MESH_MAX_SIDE: f32 = 960.0;
const SIGNUP_INNER_FORM_CARD_W: f32 = 340.0;
/// Form face alpha so mesh grads show through.
const SIGNUP_PANEL_ALPHA: f32 = 0.55;

/// Gutter between shade cells and between Color / Gray rows (Radix Colors grid).
const SCALE_SWATCH_GAP: f32 = 3.0;

const SCALE_LEGEND_GROUPS: &[(&str, usize)] = &[
    ("Backgrounds", 2),
    ("Interactive components", 3),
    ("Borders and separators", 3),
    ("Solid colors", 2),
    ("Accessible text", 2),
];

pub struct PreviewControls {
    pub search_field: TextField,
    pub search_submit: Entity<Button>,
    pub sign_up_name: TextField,
    pub sign_up_email: TextField,
    pub sign_up_password: TextField,
    pub create_account: Entity<Button>,
    pub continue_github: Entity<Button>,
    pub soft_demo: Entity<Button>,
    pub outline_demo: Entity<Button>,
    pub ghost_demo: Entity<Button>,
    pub preview_switch: Switch,
    pub task_a: Checkbox,
    pub task_b: Checkbox,
    pub task_c: Checkbox,
}

pub struct PageArgs<'a> {
    pub look: &'a Arc<RadixLook>,
    pub light_toggle: Toggle,
    pub dark_toggle: Toggle,
    pub accent_field: Entity<ColorTextField>,
    pub gray_field: Entity<ColorTextField>,
    pub background_field: Entity<ColorTextField>,
    pub copy_menu: Entity<PopupMenu>,
    pub palette_reset: Entity<Button>,
    pub preview_layout: Entity<WideMiddle>,
    pub mesh_image: Option<Arc<RenderImage>>,
    pub app: Entity<RadixStudioApp>,
    pub controls: PreviewControls,
    pub surface: Hsla,
    pub border: Hsla,
    pub muted: Hsla,
    pub fg: Hsla,
    pub accent: Hsla,
    pub is_dark: bool,
}

pub fn page(args: PageArgs<'_>, cx: &mut Context<RadixStudioApp>) -> gpui::AnyElement {
    let PageArgs {
        look,
        light_toggle,
        dark_toggle,
        accent_field,
        gray_field,
        background_field,
        copy_menu,
        palette_reset,
        preview_layout,
        mesh_image,
        app,
        controls,
        surface,
        border,
        muted,
        fg,
        accent,
        is_dark,
    } = args;

    vstack! {
        gap=28;
        header_block(fg),
        mode_toggle(light_toggle, dark_toggle, surface, border, is_dark, accent),
        seed_row(SeedRow {
            accent_field,
            gray_field,
            background_field,
            copy_menu,
            palette_reset,
            palettes: look.palettes(),
            accent,
            gray: look.resolve_step(ScaleFamily::Gray, 8).hsla(),
            background: look.resolve_step(ScaleFamily::Gray, 1).hsla(),
            border,
            muted,
            fg,
        }),
        scale_section(look, muted, cx),
        preview_section(look, mesh_image, app, preview_layout, cx, controls, surface, border, muted, fg, accent),
    }
    .into_any_element()
}

pub fn preview_layout_defaults() -> WideMiddle {
    WideMiddle::new()
        .gap(PREVIEW_COLUMN_GAP)
        .side_min_width(PREVIEW_SIDE_COLUMN_MIN)
        .side_preferred_width(PREVIEW_SIDE_COLUMN_PREFERRED)
        .middle_min_width(PREVIEW_CENTER_COLUMN_MAX)
        .middle_max_width(PREVIEW_CENTER_COLUMN_MAX)
}

fn header_block(fg: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div().text_3xl().font_weight(gpui::FontWeight::BOLD).text_color(fg).child("Create a custom palette"),
    }
    .w_full()
}

fn mode_toggle(
    light: Toggle,
    dark: Toggle,
    surface: Hsla,
    border: Hsla,
    is_dark: bool,
    accent: Hsla,
) -> impl IntoElement {
    let _ = (is_dark, accent);
    hstack! {
        gap=4 align=center;
        light,
        dark,
    }
    .mx_auto()
    .p_1()
    .rounded_lg()
    .border_1()
    .border_color(border)
    .bg(surface)
}

struct SeedRow {
    accent_field: Entity<ColorTextField>,
    gray_field: Entity<ColorTextField>,
    background_field: Entity<ColorTextField>,
    copy_menu: Entity<PopupMenu>,
    palette_reset: Entity<Button>,
    palettes: ThemePalettes,
    accent: Hsla,
    gray: Hsla,
    background: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
}

fn seed_row(row: SeedRow) -> impl IntoElement {
    let SeedRow {
        accent_field,
        gray_field,
        background_field,
        copy_menu,
        palette_reset,
        palettes,
        accent,
        gray,
        background,
        border,
        muted,
        fg,
    } = row;

    vstack! {
        gap=10 align=center;
        draft_origin(palettes, muted, fg),
        hstack! {
            gap=16 align=end justify=center;
            seed_field("Accent", accent_field, accent, border, muted),
            seed_field("Gray", gray_field, gray, border, muted),
            seed_field("Background", background_field, background, border, muted),
            div().pt_5().child(copy_menu),
            div().pt_5().child(palette_reset),
        },
    }
    .w_full()
}

/// Says which palettes the draft is on, and whether it has drifted off them.
fn draft_origin(palettes: ThemePalettes, muted: Hsla, fg: Hsla) -> impl IntoElement {
    let named = |slot: PaletteSlot| SharedString::from(slot.label());

    hstack! {
        gap=6 align=center;
        div().text_xs().text_color(muted).child("Editing"),
        div().text_xs().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child(named(palettes.accent)),
        div().text_xs().text_color(muted).child("on"),
        div().text_xs().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child(named(palettes.gray)),
    }
}

fn seed_field(
    label: &'static str,
    field: Entity<ColorTextField>,
    _swatch: Hsla,
    _border: Hsla,
    muted: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=6;
        div().text_xs().text_color(muted).child(label),
        div().w(px(180.0)).child(field),
    }
}

fn scale_section(look: &RadixLook, muted: Hsla, cx: &mut Context<RadixStudioApp>) -> impl IntoElement {
    let columns = vec![GridTrack::Star(1.0); SCALE_LEN];
    let mut grid = GridLayout::new().columns(columns).gap_x(SCALE_SWATCH_GAP).gap_y(SCALE_SWATCH_GAP);

    let mut col = 0usize;
    for &(label, span) in SCALE_LEGEND_GROUPS {
        grid = grid.child_with_span(legend_group(label, muted), 0, col, span);
        col += span;
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_step_number(step, muted), 1, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(ScaleFamily::Color, step, look, cx), 2, (step - 1) as usize);
    }

    for step in 1..=SCALE_LEN as u8 {
        grid = grid.child(scale_swatch(ScaleFamily::Gray, step, look, cx), 3, (step - 1) as usize);
    }

    grid.into_element().w_full()
}

fn scale_swatch(family: ScaleFamily, step: u8, look: &RadixLook, cx: &mut Context<RadixStudioApp>) -> impl IntoElement {
    let color = look.resolve_step(family, step).hsla();
    div()
        .id(SharedString::from(format!("swatch-{}-{step}", family.as_str())))
        .w_full()
        .h(px(44.0))
        .bg(color)
        .cursor_pointer()
        .on_click(cx.listener(move |this, _, _, cx| {
            this.open_swatch_info(family, step, cx);
        }))
}

fn legend_group(label: &'static str, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4 align=center;
        div()
            .w_full()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .text_center()
            .truncate()
            .child(label),
        div().w_full().h(px(2.0)).rounded_full().bg(muted.opacity(0.45)),
    }
    .w_full()
}

fn scale_step_number(step: u8, muted: Hsla) -> impl IntoElement {
    hstack! {
        justify=center;
        div()
            .text_xs()
            .font_weight(gpui::FontWeight::MEDIUM)
            .text_color(muted)
            .child(format!("{step}")),
    }
    .w_full()
}

#[allow(clippy::too_many_arguments)]
fn preview_section(
    look: &RadixLook,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    preview_layout: Entity<WideMiddle>,
    cx: &App,
    controls: PreviewControls,
    surface: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    WideMiddleLayout::new(
        preview_layout,
        preview_column_left(
            controls.search_field,
            controls.search_submit,
            controls.preview_switch,
            controls.task_a,
            controls.task_b,
            controls.task_c,
            surface,
            border,
            muted,
            fg,
            accent,
        ),
        preview_column_center(
            look,
            mesh_image,
            app,
            controls.sign_up_name,
            controls.sign_up_email,
            controls.sign_up_password,
            controls.create_account,
            controls.continue_github,
            border,
            muted,
            fg,
        ),
        preview_column_right(controls.soft_demo, controls.outline_demo, controls.ghost_demo, muted, fg, accent),
    )
    .build(cx)
}

#[allow(clippy::too_many_arguments)]
fn preview_column_left(
    search_field: TextField,
    search_submit: Entity<Button>,
    preview_switch: Switch,
    task_a: Checkbox,
    task_b: Checkbox,
    task_c: Checkbox,
    surface: Hsla,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=14;
        hstack! {
            gap=8 align=center;
            div().flex_1().min_w_0().child(search_field),
            search_submit,
        }
        .w_full(),
        alert_card(accent, muted, fg),
        div()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(border)
            .bg(surface)
            .p_3()
            .child(
                vstack! {
                    gap=10;
                    preview_switch,
                    task_a,
                    task_b,
                    task_c,
                }
                .w_full(),
            ),
    }
    .w_full()
}

#[allow(clippy::too_many_arguments)]
fn preview_column_center(
    look: &RadixLook,
    mesh_image: Option<Arc<RenderImage>>,
    app: Entity<RadixStudioApp>,
    name: TextField,
    email: TextField,
    password: TextField,
    create: Entity<Button>,
    github: Entity<Button>,
    border: Hsla,
    muted: Hsla,
    fg: Hsla,
) -> impl IntoElement {
    let stage = SignupStage::default();
    let stage_fill = stage.fill(look);
    let panel_fill = {
        let mut c = stage.card(look);
        c.a = SIGNUP_PANEL_ALPHA;
        c
    };
    let mesh_opacity = stage.mesh_opacity;

    let inner_form_card = div()
        .id("inner_form_card")
        .flex_none()
        .w(px(SIGNUP_INNER_FORM_CARD_W))
        .min_w(px(SIGNUP_INNER_FORM_CARD_W))
        .max_w(px(SIGNUP_INNER_FORM_CARD_W))
        .overflow_hidden()
        .rounded_xl()
        .border_1()
        .border_color(border)
        .bg(panel_fill)
        .p_6()
        .child(
            vstack! {
                gap=16;
                labeled_field("Full name", name, muted),
                labeled_field("Email", email, muted),
                labeled_field("Password", password, muted),
                create,
                hstack! {
                    gap=12 align=center;
                    div().flex_1().h(px(1.0)).bg(border),
                    div().text_xs().text_color(muted).child("OR"),
                    div().flex_1().h(px(1.0)).bg(border),
                }
                .w_full(),
                github,
            }
            .w_full(),
        );

    let content = vstack! {
        gap=16 align=center justify=center;
        div()
            .flex_none()
            .text_xl()
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(fg)
            .child("Sign up"),
        div()
            .w_full()
            .flex()
            .justify_center()
            .child(inner_form_card),
    }
    .relative()
    .w_full()
    .px_4()
    .pt_6()
    .pb(px(45.0));

    div()
        .id("signup_outer_card")
        .w_full()
        .overflow_hidden()
        .relative()
        .bg(stage_fill)
        .on_prepaint(move |bounds, window, cx| {
            let width_px = bounds.size.width.as_f32().max(1.0);
            let height_px = bounds.size.height.as_f32().max(1.0);
            let scale = (SIGNUP_MESH_MAX_SIDE / width_px.max(height_px)).min(1.0);
            let width = (width_px * scale).round().max(1.0) as u32;
            let height = (height_px * scale).round().max(1.0) as u32;
            app.update(cx, |this, cx| {
                if this.ensure_signup_mesh_cache(width, height) {
                    window.refresh();
                    cx.notify();
                }
            });
        })
        .when_some(mesh_image, |this, image| {
            this.child(
                img(ImageSource::Render(image))
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(gpui::ObjectFit::Cover)
                    .opacity(mesh_opacity),
            )
        })
        .child(content)
}

fn preview_column_right(
    soft: Entity<Button>,
    outline: Entity<Button>,
    ghost: Entity<Button>,
    muted: Hsla,
    fg: Hsla,
    accent: Hsla,
) -> impl IntoElement {
    vstack! {
        gap=14;
        typography_block(muted, fg, accent),
        hstack! {
            gap=8;
            soft,
            outline,
            ghost,
        }
        .w_full(),
    }
    .w_full()
}

fn labeled_field(label: &'static str, field: TextField, muted: Hsla) -> impl IntoElement {
    vstack! {
        gap=4;
        div().text_xs().text_color(muted).child(label),
        field,
    }
    .w_full()
}

fn alert_card(accent: Hsla, muted: Hsla, fg: Hsla) -> impl IntoElement {
    div()
        .w_full()
        .rounded_lg()
        .p_3()
        .bg(gpui::hsla(accent.h, accent.s * 0.35, (accent.l * 0.35).clamp(0.12, 0.35), 1.0))
        .border_1()
        .border_color(accent)
        .child(hstack! {
            gap=10 align=start;
            div().text_sm().text_color(accent).child("i"),
            vstack! {
                gap=2;
                div().text_sm().text_color(fg).child("Please upgrade to the new version."),
                div().text_xs().text_color(muted).child("Accent soft surface preview"),
            },
        })
}

fn typography_block(muted: Hsla, fg: Hsla, accent: Hsla) -> impl IntoElement {
    vstack! {
        gap=8;
        div().text_sm().text_color(fg).child(
            "Susan Kare is an artist and graphic designer known for creating many of the interface elements for the Apple Macintosh in the 1980s."
        ),
        div().text_sm().text_color(muted).child(
            "She also designed icons and typefaces for NeXT, IBM, and Microsoft."
        ),
        div().text_sm().text_color(accent).child("Learn more →"),
    }
    .w_full()
}
