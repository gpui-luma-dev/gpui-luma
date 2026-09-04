use super::*;

use gpui::{
    ClickEvent, Context, Anchor, Corners, HitboxBehavior, ImageSource, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Render, Window, anchored, canvas, deferred, div, img, point, prelude::*, px, relative, size,
};
use luma::infra::{ElementExt, StyledExt};
use luma::theme::LumaTextStyle;
use luma::{form_field, hstack, vstack};
use luma_look_shadcn::{LumaTypographyExt, ShadcnTextSize};

use super::super::color::{format_hex_color, format_percent};
use super::super::paint::paint_gradient_preview;
use super::mesh::{
    MESH_HANDLE_SIZE, SHELL_RADIUS, controls_panel_corner_radii, fit_aspect_ratio, preview_panel_corner_radii,
    render_info_row,
};

impl Render for GradientBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let stops = self.preview_stops(cx);
        self.sync_preview_strategy(stops.len());
        let rotation_deg = self.rotation_deg;
        let gradient_spec = self.gradient_spec(cx);
        let code_style = self.look.typography_scale(ShadcnTextSize::Xs);
        let info_label_style = self.look.mode_tokens().typography.text.label;
        let info_value_style = self.look.mode_tokens().typography.text.label;
        let top_tabs = self.top_tabs.clone();
        let gradient_stops = self.gradient_stops.clone();
        let add_stop_button = self.add_stop_button.clone();
        let rotation_slider = self.rotation_slider.clone();
        let type_selector = self.type_selector.clone();
        let renderer_selector = self.renderer_selector.clone();
        let color_picker = self.color_picker.clone();
        let color_picker_open = self.color_picker_open;
        let card_bg = self.look.token_color("card").unwrap_or(chrome.panel_background);
        let stop_rows = self.ordered_stop_rows(cx);
        let preview_corner_radii = preview_panel_corner_radii();
        let preview_surface_radii =
            Corners { top_left: px(0.0), top_right: px(0.0), bottom_left: px(0.0), bottom_right: px(0.0) };
        let preview_uses_render_image = self.uses_render_preview(cx);
        let allow_native_two_stop = self.gradient_type == GradientType::Linear
            && stops.len() == 2
            && self.effective_preview_renderer() == PreviewRenderer::Quads;
        let preview_image = if preview_uses_render_image {
            self.cached_preview_image()
        } else {
            None
        };
        let diagnostics_panel = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(render_info_row("Active", self.active_preview_strategy, info_label_style, info_value_style, chrome))
            .child(render_info_row(
                "Requested",
                match self.preview_renderer {
                    PreviewRenderer::Quads => "quads",
                    PreviewRenderer::RenderImageSync => "render (sync)",
                    PreviewRenderer::RenderImageAsync => "render (async)",
                },
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row("Type", self.gradient_type.label(), info_label_style, info_value_style, chrome))
            .child(render_info_row("Stops", format!("{}", stops.len()), info_label_style, info_value_style, chrome))
            .child(render_info_row(
                "Preview",
                format!(
                    "{} x {}",
                    self.preview_size.width.as_f32().round() as i32,
                    self.preview_size.height.as_f32().round() as i32
                ),
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row(
                "Raster",
                self.last_render_ms.map(|ms| format!("{ms:.2} ms")).unwrap_or_else(|| "n/a".to_string()),
                info_label_style,
                info_value_style,
                chrome,
            ));
        let stop_editor = div()
            .w_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(form_field!("Type", chrome; type_selector))
            .child(form_field!("Renderer", chrome; renderer_selector))
            .when(self.gradient_type != GradientType::Radial, |this| {
                this.child(form_field!("Angle", chrome;
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(rotation_slider)
                        .child(
                            div()
                                .typography_style(code_style)
                                .text_color(chrome.muted_text)
                                .child(format!("{rotation_deg:.0}deg"))
                        )
                ))
            })
            .child(
                div()
                    .id("color-viz-gradient-spec")
                    .typography_style(code_style)
                    .text_color(chrome.muted_text)
                    .font_family("Monaco")
                    .whitespace_normal()
                    .child(gradient_spec),
            );

        let controls = vstack! {
            gap=14;
            div()
                .w_full()
                .flex()
                .items_start()
                .justify_between()
                .gap_4()
                .child(div().flex_1().min_w(px(0.0)).pt_1().child(gradient_stops))
                .child(add_stop_button),
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_2()
                .rounded(px(12.0))
                .border_1()
                .border_color(chrome.border)
                .bg(chrome.panel_background)
                .p_3()
                .children(stop_rows.into_iter().map(|(thumb_id, position, color)| {
                    render_stop_row(
                        thumb_id,
                        position,
                        color,
                        self.selected_stop == Some(thumb_id),
                        self.stop_delete_buttons.get(&thumb_id).cloned(),
                        self.look.mode_tokens().typography.text.label,
                        self.look.mode_tokens().typography.text.label,
                        chrome,
                        cx.entity(),
                    )
                })),
            stop_editor,
            diagnostics_panel,
        }
        .id("color-viz-gradient-controls")
        .w(px(432.0))
        .flex_shrink_0()
        .h_full()
        .min_h_0()
        .corner_radii(controls_panel_corner_radii())
        .overflow_y_scroll()
        .p(px(16.0))
        .bg(card_bg);
        let picker_bounds = match self.selected_tab {
            BuilderTab::Gradients => {
                self.selected_stop.and_then(|thumb_id| self.stop_swatch_bounds.get(&thumb_id).copied())
            }
            BuilderTab::Mesh => match self.mesh_color_target {
                MeshColorTarget::Point(point_index) => self.mesh_swatch_bounds.get(&point_index).copied(),
                MeshColorTarget::Background => self.mesh_background_swatch_bounds,
            },
        };
        let content = match self.selected_tab {
            BuilderTab::Gradients => {
                let controls = if color_picker_open {
                    if let Some(bounds) = picker_bounds {
                        controls.child(
                            deferred(
                                anchored()
                                    .snap_to_window_with_margin(px(8.0))
                                    .anchor(Anchor::TopLeft)
                                    .position(point(bounds.left(), bounds.bottom()))
                                    .offset(point(px(0.0), px(4.0)))
                                    .child(
                                        self.look
                                            .card("color-viz-color-picker-popup")
                                            .elevated(true)
                                            .child(color_picker.clone())
                                            .render(window, cx)
                                            .occlude()
                                            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                                this.color_picker_open = false;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .with_priority(1),
                        )
                    } else {
                        controls
                    }
                } else {
                    controls
                };
                let preview_builder = cx.entity();
                let preview_surface = div()
                    .id("color-viz-gradient-preview")
                    .relative()
                    .size_full()
                    .rounded(px(0.0))
                    .overflow_hidden()
                    .on_prepaint(move |bounds, window, cx| {
                        preview_builder.update(cx, |this, cx| {
                            if this.handle_preview_bounds(bounds, cx) {
                                window.refresh();
                            }
                        });
                    })
                    .when_some(preview_image, |this, image| {
                        this.child(img(ImageSource::Render(image)).size_full().absolute().top_0().left_0())
                    })
                    .when(!preview_uses_render_image, |this| {
                        this.child({
                            let stops = stops.clone();
                            let gradient_type = self.gradient_type;
                            canvas(
                                move |_, _, _| {},
                                move |bounds, _, window, _cx| {
                                    paint_gradient_preview(
                                        gradient_type,
                                        window,
                                        bounds,
                                        &stops,
                                        rotation_deg,
                                        preview_surface_radii,
                                        allow_native_two_stop,
                                    );
                                },
                            )
                            .size_full()
                        })
                    });
                let preview = div()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .corner_radii(preview_corner_radii)
                    .p(px(10.0))
                    .child(preview_surface);

                hstack! {
                    gap=0;
                    controls,
                    div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
                    preview,
                }
                .items_stretch()
                .w_full()
                .flex_1()
                .min_h_0()
                .into_any_element()
            }
            BuilderTab::Mesh => {
                let preview_container_builder = cx.entity();
                let preview_frame_builder = cx.entity();
                let frame_size = fit_aspect_ratio(self.mesh_preview_container_size, self.mesh_aspect_ratio_preset);
                let handle_inset = px(MESH_HANDLE_SIZE * 0.5);
                let work_area_size =
                    size(frame_size.width + px(MESH_HANDLE_SIZE), frame_size.height + px(MESH_HANDLE_SIZE));
                let mesh_controls = div().child(render_mesh_controls(
                    self.mesh_points.clone(),
                    self.mesh_background,
                    self.selected_mesh_point,
                    self.mesh_grid_preset,
                    self.mesh_grid_selector.clone(),
                    self.mesh_aspect_ratio_preset,
                    self.mesh_aspect_ratio_selector.clone(),
                    self.mesh_reset_button.clone(),
                    chrome,
                    card_bg,
                    info_label_style,
                    info_value_style,
                    self.preview_size,
                    self.last_render_ms,
                    cx.entity(),
                ));
                let mesh_controls = if color_picker_open {
                    if let Some(bounds) = picker_bounds {
                        mesh_controls.child(
                            deferred(
                                anchored()
                                    .snap_to_window_with_margin(px(8.0))
                                    .anchor(Anchor::TopLeft)
                                    .position(point(bounds.left(), bounds.bottom()))
                                    .offset(point(px(0.0), px(4.0)))
                                    .child(
                                        self.look
                                            .card("color-viz-color-picker-popup")
                                            .elevated(true)
                                            .child(color_picker.clone())
                                            .render(window, cx)
                                            .occlude()
                                            .on_mouse_down_out(cx.listener(|this, _: &MouseDownEvent, _, cx| {
                                                this.color_picker_open = false;
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .with_priority(1),
                        )
                    } else {
                        mesh_controls
                    }
                } else {
                    mesh_controls
                };
                let preview_surface = div()
                    .id("color-viz-mesh-preview")
                    .relative()
                    .w(work_area_size.width)
                    .h(work_area_size.height)
                    .rounded(px(0.0))
                    .child(
                        div()
                            .absolute()
                            .left(handle_inset)
                            .top(handle_inset)
                            .w(frame_size.width)
                            .h(frame_size.height)
                            .overflow_hidden()
                            .when_some(self.cached_preview_image(), |this, image| {
                                this.child(img(ImageSource::Render(image)).size_full().absolute().top_0().left_0())
                            })
                            .child(
                                canvas(|bounds, window, _| window.insert_hitbox(bounds, HitboxBehavior::Normal), {
                                    let builder = cx.entity();
                                    move |bounds, _hitbox, window, cx| {
                                        preview_frame_builder.update(cx, |this, cx| {
                                            if this.mesh_preview_bounds != Some(bounds) {
                                                this.mesh_preview_bounds = Some(bounds);
                                                cx.notify();
                                            }
                                        });

                                        window.on_mouse_event({
                                            let builder = builder.clone();
                                            move |event: &MouseMoveEvent, phase, _window, cx| {
                                                if !phase.bubble() && !phase.capture() {
                                                    return;
                                                }
                                                builder.update(cx, |this, cx| {
                                                    this.handle_mesh_drag_move(event.position, cx);
                                                });
                                            }
                                        });

                                        window.on_mouse_event({
                                            let builder = builder.clone();
                                            move |_event: &MouseUpEvent, _phase, _window, cx| {
                                                builder.update(cx, |this, cx| {
                                                    this.finish_mesh_drag(cx);
                                                });
                                            }
                                        });
                                    }
                                })
                                .absolute()
                                .inset_0(),
                            ),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(handle_inset)
                            .top(handle_inset)
                            .w(frame_size.width)
                            .h(frame_size.height)
                            .children(self.mesh_points.iter().enumerate().map(|(point_index, point)| {
                                render_mesh_handle(
                                    point_index,
                                    *point,
                                    self.selected_mesh_point == Some(point_index),
                                    cx.entity(),
                                )
                            })),
                    );
                let preview = div()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .corner_radii(preview_corner_radii)
                    .p(px(18.0))
                    .on_prepaint(move |bounds, window, cx| {
                        preview_container_builder.update(cx, |this, cx| {
                            if this.handle_mesh_preview_container_bounds(bounds, cx) {
                                window.refresh();
                            }
                        });
                    })
                    .child(preview_surface);

                hstack! {
                    gap=0;
                    mesh_controls,
                    div().w(px(1.0)).flex_shrink_0().bg(chrome.border),
                    preview,
                }
                .items_stretch()
                .w_full()
                .flex_1()
                .min_h_0()
                .into_any_element()
            }
        };

        let shell = div()
            .id("color-viz-gradient-shell")
            .w_full()
            .h_full()
            .min_h_0()
            .flex()
            .flex_col()
            .rounded(px(SHELL_RADIUS))
            .overflow_hidden()
            .border_1()
            .border_color(chrome.border)
            .child(div().w_full().flex_shrink_0().p(px(10.0)).bg(card_bg).child(top_tabs))
            .child(div().h(px(1.0)).w_full().flex_shrink_0().bg(chrome.border))
            .child(content);

        div()
            .id("color-viz-gradient-builder")
            .size_full()
            .min_h_0()
            .flex()
            .items_stretch()
            .justify_start()
            .p(px(32.0))
            .child(shell)
    }
}

#[allow(clippy::too_many_arguments)]
fn render_stop_row(
    thumb_id: ThumbId,
    position: f32,
    color: gpui::Hsla,
    selected: bool,
    delete_button: Option<Entity<Button>>,
    label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    chrome: luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let border = chrome.border;
    let background = if selected {
        chrome.content_background
    } else {
        chrome.panel_background
    };
    let swatch_builder = builder.clone();
    let bounds_builder = builder.clone();

    div()
        .id(format!("color-viz-gradient-stop-row-{}", thumb_id.as_u64()))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(border)
        .bg(background)
        .cursor_pointer()
        .on_click(move |_, _, cx| {
            builder.update(cx, |this, cx| {
                this.gradient_stops.update(cx, |slider, cx| {
                    slider.select_thumb_id(thumb_id, cx);
                });
            });
        })
        .child(
            div()
                .id(format!("color-viz-gradient-stop-swatch-{}", thumb_id.as_u64()))
                .size(px(28.0))
                .rounded(px(6.0))
                .bg(color)
                .border_1()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_stop_swatch_bounds(thumb_id, &bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    swatch_builder.update(cx, |this, cx| {
                        this.open_color_picker_for_stop(thumb_id, cx);
                    });
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("HEX"))
                        .child(
                            div()
                                .typography_style(value_style)
                                .text_color(chrome.body_text)
                                .child(format_hex_color(color)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("POSITION"))
                        .child(
                            div()
                                .typography_style(value_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(position)),
                        ),
                ),
        )
        .when_some(delete_button, |row, button| row.child(button))
}

#[allow(clippy::too_many_arguments)]
fn render_mesh_controls(
    mesh_points: Vec<MeshPoint>,
    mesh_background: gpui::Hsla,
    selected_mesh_point: Option<usize>,
    mesh_grid_preset: MeshGridPreset,
    mesh_grid_selector: Entity<Selector>,
    mesh_aspect_ratio_preset: MeshAspectRatioPreset,
    mesh_aspect_ratio_selector: Entity<Selector>,
    mesh_reset_button: Entity<Button>,
    chrome: luma::theme::LumaChrome,
    card_bg: gpui::Hsla,
    info_label_style: LumaTextStyle,
    info_value_style: LumaTextStyle,
    preview_size: Size<Pixels>,
    last_render_ms: Option<f32>,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let selected_label = selected_mesh_point
        .and_then(|index| mesh_points.get(index))
        .map(|point| format!("P{}{}", point.row, point.col))
        .unwrap_or_else(|| "None".to_string());

    vstack! {
        gap=14;
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(div().typography_style(info_label_style).text_color(chrome.muted_text).child("Mesh"))
                    .child(mesh_reset_button),
            )
            .child(form_field!("Preset", chrome; mesh_grid_selector))
            .child(render_info_row("Grid", mesh_grid_preset.label(), info_label_style, info_value_style, chrome))
            .child(form_field!("Aspect", chrome; mesh_aspect_ratio_selector))
            .child(render_info_row("Frame", mesh_aspect_ratio_preset.label(), info_label_style, info_value_style, chrome))
            .child(render_info_row("Selected", selected_label, info_label_style, info_value_style, chrome))
            .child(render_mesh_background_row(mesh_background, info_label_style, chrome, builder.clone())),
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .p_3()
            .children(mesh_points.into_iter().enumerate().map(|(point_index, point)| {
                render_mesh_point_row(
                    point_index,
                    point,
                    selected_mesh_point == Some(point_index),
                    info_label_style,
                    chrome,
                    builder.clone(),
                )
            })),
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .rounded(px(12.0))
            .border_1()
            .border_color(chrome.border)
            .bg(chrome.panel_background)
            .child(render_info_row(
                "Preview",
                format!(
                    "{} x {}",
                    preview_size.width.as_f32().round() as i32,
                    preview_size.height.as_f32().round() as i32
                ),
                info_label_style,
                info_value_style,
                chrome,
            ))
            .child(render_info_row(
                "Raster",
                last_render_ms.map(|ms| format!("{ms:.2} ms")).unwrap_or_else(|| "n/a".to_string()),
                info_label_style,
                info_value_style,
                chrome,
            )),
    }
    .id("color-viz-mesh-controls")
    .w(px(432.0))
    .flex_shrink_0()
    .h_full()
    .min_h_0()
    .corner_radii(controls_panel_corner_radii())
    .overflow_y_scroll()
    .p(px(16.0))
    .bg(card_bg)
}

fn render_mesh_background_row(
    background: gpui::Hsla,
    label_style: LumaTextStyle,
    chrome: luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let row_builder = builder.clone();

    div()
        .id("color-viz-mesh-background-row")
        .w_full()
        .flex()
        .items_center()
        .justify_between()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(chrome.border)
        .bg(chrome.content_background)
        .cursor_pointer()
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            row_builder.update(cx, |this, cx| {
                this.open_color_picker_for_mesh_background(cx);
            });
        })
        .child(div().typography_style(label_style).text_color(chrome.body_text).child("Background"))
        .child(
            div()
                .id("color-viz-mesh-background-swatch")
                .size(px(28.0))
                .rounded(px(999.0))
                .bg(background)
                .border_2()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_mesh_background_swatch_bounds(&bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    builder.update(cx, |this, cx| {
                        this.open_color_picker_for_mesh_background(cx);
                    });
                }),
        )
}

fn render_mesh_point_row(
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    label_style: LumaTextStyle,
    chrome: luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let swatch_builder = builder.clone();
    let background = if selected {
        chrome.content_background
    } else {
        chrome.panel_background
    };

    div()
        .id(format!("color-viz-mesh-point-row-{point_index}"))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .rounded(px(10.0))
        .border_1()
        .border_color(chrome.border)
        .bg(background)
        .cursor_pointer()
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            builder.update(cx, |this, cx| {
                if event.click_count() >= 2 {
                    this.open_color_picker_for_mesh_point(point_index, cx);
                } else {
                    this.selected_mesh_point = Some(point_index);
                    this.mesh_color_target = MeshColorTarget::Point(point_index);
                    this.color_picker_open = false;
                    cx.notify();
                }
            });
        })
        .child(
            div()
                .id(format!("color-viz-mesh-point-swatch-{point_index}"))
                .size(px(28.0))
                .rounded(px(999.0))
                .bg(point.color)
                .border_2()
                .border_color(chrome.border)
                .cursor_pointer()
                .on_prepaint(move |bounds, _, cx| {
                    bounds_builder.update(cx, |this, _| {
                        this.handle_mesh_swatch_bounds(point_index, &bounds);
                    });
                })
                .on_click(move |event: &ClickEvent, _, cx| {
                    if event.is_keyboard() {
                        return;
                    }
                    cx.stop_propagation();
                    swatch_builder.update(cx, |this, cx| {
                        this.open_color_picker_for_mesh_point(point_index, cx);
                    });
                }),
        )
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .items_center()
                .justify_between()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("POINT"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format!("P{}{}", point.row, point.col)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("X"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(point.u)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().typography_style(label_style).text_color(chrome.muted_text).child("Y"))
                        .child(
                            div()
                                .typography_style(label_style)
                                .text_color(chrome.body_text)
                                .child(format_percent(point.v)),
                        ),
                ),
        )
}

fn render_mesh_handle(
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let border = if selected {
        gpui::hsla(0.0, 0.0, 1.0, 1.0)
    } else {
        gpui::hsla(0.0, 0.0, 1.0, 0.82)
    };
    let drag_builder = builder.clone();
    let release_builder = builder.clone();

    div()
        .id(format!("color-viz-mesh-handle-{point_index}"))
        .absolute()
        .left(relative(point.u))
        .top(relative(point.v))
        .ml(px(-MESH_HANDLE_SIZE * 0.5))
        .mt(px(-MESH_HANDLE_SIZE * 0.5))
        .size(px(MESH_HANDLE_SIZE))
        .rounded(px(999.0))
        .bg(point.color)
        .border_2()
        .border_color(border)
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
            cx.stop_propagation();
            drag_builder.update(cx, |this, cx| {
                this.begin_mesh_drag(point_index, cx);
            });
        })
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            release_builder.update(cx, |this, cx| {
                this.finish_mesh_drag(cx);
            });
        })
        .on_click(move |event: &ClickEvent, _, cx| {
            if event.is_keyboard() {
                return;
            }
            cx.stop_propagation();
            builder.update(cx, |this, cx| {
                this.selected_mesh_point = Some(point_index);
                this.mesh_color_target = MeshColorTarget::Point(point_index);
                this.color_picker_open = false;
                cx.notify();
            });
        })
}
