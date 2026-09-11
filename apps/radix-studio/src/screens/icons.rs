//! Icons tab screen.

use gpui::{Bounds, Hsla, IntoElement, PathBuilder, Pixels, Point, Window, canvas, div, point, prelude::*, px};
use luma::vstack;

use crate::assets::{ICON_GROUPS, icon_named, react_icon};

const MINT_LINE: Hsla = Hsla { h: 168.0 / 360.0, s: 0.80, l: 0.60, a: 0.24 };
const PINK_LINE: Hsla = Hsla { h: 325.0 / 360.0, s: 0.90, l: 0.65, a: 0.28 };

pub fn page(fg: Hsla, muted: Hsla, surface: Hsla) -> gpui::AnyElement {
    let mut groups = vstack! { gap=28; };

    for (title, names) in ICON_GROUPS {
        let mut grid = div().w_full().flex().flex_row().flex_wrap().gap(px(20.0)).justify_start();

        for name in *names {
            if let Some(icon) = icon_named(name) {
                grid = grid.child(
                    div()
                        .size(px(15.0))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(react_icon(icon, fg, 15.0)),
                );
            }
        }

        groups = groups.child(
            vstack! {
                gap=16;
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(*title),
                grid,
            }
            .w_full(),
        );
    }

    vstack! {
        gap=16 align=center;
        div().text_2xl().font_weight(gpui::FontWeight::SEMIBOLD).text_color(fg).child("Icons"),
        div()
            .text_sm()
            .text_color(muted)
            .child("A crisp set of 15×15 icons."),
        div()
            .w_full()
            .rounded_xl()
            .bg(surface)
            .px_8()
            .py_8()
            .child(groups),
    }
    .w_full()
    .px_6()
    .py_12()
    .into_any_element()
}

pub fn hero_decoration(fg: Hsla) -> impl IntoElement {
    div().absolute().inset_0().overflow_hidden().child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                paint_hero_construction(bounds, fg, window);
            },
        )
        .absolute()
        .size_full(),
    )
}

fn paint_hero_construction(bounds: Bounds<Pixels>, fg: Hsla, window: &mut Window) {
    let s = 80.0_f32;
    let x0 = f32::from(bounds.left()) + (f32::from(bounds.size.width) - s) * 0.5 - 200.0;
    let y0 = f32::from(bounds.top()) + 230.0;

    let span = (f32::from(bounds.size.width) + f32::from(bounds.size.height)) * 2.0;

    let mut paint_ray = |px_off: f32, py_off: f32, deg: f32, color: Hsla| {
        let angle = deg.to_radians();
        let dx = angle.cos() * span;
        let dy = angle.sin() * span;
        let cx = x0 + px_off * s;
        let cy = y0 + py_off * s;

        let mut b = PathBuilder::stroke(px(1.0));
        b.move_to(point(px(cx - dx), px(cy - dy)));
        b.line_to(point(px(cx + dx), px(cy + dy)));
        if let Ok(path) = b.build() {
            window.paint_path(path, color);
        }
    };

    paint_ray(0.0, 0.0, 0.0, MINT_LINE);
    paint_ray(0.0, 1.0, 0.0, MINT_LINE);
    paint_ray(0.0, 0.0, 90.0, MINT_LINE);
    paint_ray(1.0, 0.0, 90.0, MINT_LINE);

    paint_ray(0.0, 0.467, 0.0, PINK_LINE);
    paint_ray(0.0, 0.533, 0.0, PINK_LINE);
    paint_ray(-0.288, 0.0, 45.0, PINK_LINE);
    paint_ray(-0.288, 0.090, 45.0, PINK_LINE);
    paint_ray(0.622, 0.0, -45.0, PINK_LINE);
    paint_ray(0.622, 0.090, -45.0, PINK_LINE);
    paint_ray(0.242, 0.0, 45.0, PINK_LINE);
    paint_ray(0.242, 1.0, -45.0, PINK_LINE);

    paint_hero_arrow(Point::new(px(x0), px(y0)), s, fg, window);
}

fn paint_hero_arrow(origin: Point<Pixels>, s: f32, fg: Hsla, window: &mut Window) {
    let scale = s / 15.0;
    let mut b = PathBuilder::fill();
    let p = |x: f32, y: f32| point(origin.x + px(x * scale), origin.y + px(y * scale));

    b.move_to(p(6.854, 3.146));
    b.line_to(p(3.707, 7.0));
    b.line_to(p(12.5, 7.0));
    b.line_to(p(13.0, 7.5));
    b.line_to(p(12.5, 8.0));
    b.line_to(p(3.707, 8.0));
    b.line_to(p(6.854, 11.854));
    b.line_to(p(6.146, 11.854));
    b.line_to(p(2.146, 7.854));
    b.line_to(p(2.0, 7.5));
    b.line_to(p(2.146, 7.146));
    b.line_to(p(6.146, 3.146));
    b.close();

    if let Ok(path) = b.build() {
        window.paint_path(path, fg);
    }
}
