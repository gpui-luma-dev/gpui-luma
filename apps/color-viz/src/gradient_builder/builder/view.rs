use super::*;

use gpui::{
    ClickEvent, Context, Anchor, Corners, HitboxBehavior, ImageSource, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Render, Window, anchored, canvas, deferred, div, img, point, prelude::*, px, relative, size,
};
use gpui_luma::infra::{ElementExt, StyledExt};
use gpui_luma::theme::LumaTextStyle;
use gpui_luma::{form_field, hstack, vstack};
use gpui_luma_look_shadcn::LumaTypographyExt;
use gpui_luma_look_shadcn as shadcn;

use super::super::color::{format_hex_color, format_percent};
use super::super::paint::paint_gradient_preview;
use super::mesh::{
    MESH_HANDLE_SIZE, controls_panel_corner_radii, fit_aspect_ratio, preview_panel_corner_radii, render_info_row,
};

const INSPECTOR_WIDTH: f32 = 352.0;

fn section_heading(label: &'static str, chrome: gpui_luma::theme::LumaChrome) -> impl IntoElement {
    div()
        .pb(px(6.0))
        .text_size(px(10.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(chrome.muted_text)
        .child(label)
}

fn compact_slider_row(
    label: &'static str,
    slider: Entity<SliderControl>,
    value: String,
    style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
) -> impl IntoElement {
    div()
        .w_full()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(100.0))
                .flex_shrink_0()
                .typography_style(style)
                .text_color(chrome.muted_text)
                .child(label),
        )
        .child(div().flex_1().min_w_0().child(slider))
        .child(
            div()
                .w(px(44.0))
                .flex_shrink_0()
                .text_right()
                .typography_style(style)
                .text_color(chrome.body_text)
                .child(value),
        )
}

impl GradientBuilder {
    fn render_image_adjustments(&self) -> impl IntoElement {
        let chrome = self.look.chrome();
        let style = self.look.mode_tokens().typography.text.label;
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .pt(px(24.0))
            .pb(px(20.0))
            .bg(chrome.panel_background)
            .child(section_heading("COLOR", chrome))
            .child(compact_slider_row(
                "Saturation",
                self.saturation_slider.clone(),
                format!("{:+.0}", self.image_adjustments.saturation),
                style,
                chrome,
            ))
            .child(compact_slider_row(
                "Vibrance",
                self.vibrance_slider.clone(),
                format!("{:+.0}", self.image_adjustments.vibrance),
                style,
                chrome,
            ))
    }
}

impl Render for GradientBuilder {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.look.chrome();
        let stops = self.preview_stops(cx);
        self.sync_preview_strategy(stops.len());
        let rotation_deg = self.rotation_deg;
        let gradient_spec = self.gradient_spec(cx);
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
            .py(px(12.0))
            .bg(chrome.panel_background)
            .child(section_heading("RENDER", chrome))
            .child(div().text_size(px(10.0)).text_color(chrome.muted_text).child(gradient_spec))
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
            .gap_2()
            .pb(px(20.0))
            .bg(chrome.panel_background)
            .child(section_heading("CANVAS", chrome))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().flex_1().min_w_0().child(form_field!("Type", chrome; type_selector)))
                    .child(div().flex_1().min_w_0().child(form_field!("Renderer", chrome; renderer_selector))),
            )
            .when(self.gradient_type != GradientType::Radial, |this| {
                this.child(compact_slider_row(
                    "Angle",
                    rotation_slider,
                    format!("{rotation_deg:.0}°"),
                    info_label_style,
                    chrome,
                ))
            });

        let controls = vstack! {
            gap=2;
            stop_editor,
            div().pt(px(24.0)).child(section_heading("FIELDS", chrome)),
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
                .bg(chrome.panel_background)
                .py(px(12.0))
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
            self.render_image_adjustments(),
            diagnostics_panel,
        }
        .id("color-viz-gradient-controls")
        .w(px(INSPECTOR_WIDTH))
        .flex_shrink_0()
        .h_full()
        .min_h_0()
        .corner_radii(controls_panel_corner_radii())
        .overflow_y_scroll()
        .px(px(12.0))
        .pb(px(12.0))
        .bg(card_bg);
        let picker_bounds = match self.selected_tab {
            BuilderTab::Gradients => {
                self.selected_stop.and_then(|thumb_id| self.stop_swatch_bounds.get(&thumb_id).copied())
            }
            BuilderTab::Mesh | BuilderTab::Freeform => match self.mesh_color_target {
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
                                        shadcn::Card::new("color-viz-color-picker-popup")
                                            .look(self.look.as_ref())
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
            BuilderTab::Mesh | BuilderTab::Freeform => {
                let preview_container_builder = cx.entity();
                let preview_frame_builder = cx.entity();
                let frame_size = fit_aspect_ratio(self.mesh_preview_container_size, self.mesh_aspect_ratio_preset);
                let handle_inset = px(MESH_HANDLE_SIZE * 0.5);
                let work_area_size =
                    size(frame_size.width + px(MESH_HANDLE_SIZE), frame_size.height + px(MESH_HANDLE_SIZE));
                let mesh_controls = div().child(render_mesh_controls(
                    self.render_image_adjustments().into_any_element(),
                    self.selected_tab == BuilderTab::Freeform,
                    self.freeform_buttons.clone(),
                    self.spread_slider.clone(),
                    self.spread_percent,
                    self.blend_selector.clone(),
                    self.point_spread_slider.clone(),
                    self.selected_mesh_point.and_then(|index| self.point_spreads.get(index)).copied().unwrap_or(100.0),
                    self.softness_slider.clone(),
                    self.softness_percent,
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
                                        shadcn::Card::new("color-viz-color-picker-popup")
                                            .look(self.look.as_ref())
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
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.deselect_freeform_point(cx);
                        }),
                    )
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(|this, _, _, cx| {
                            cx.stop_propagation();
                            this.toggle_mesh_controls(cx);
                        }),
                    )
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
                    .when(self.mesh_controls_visible, |this| {
                        this.child(
                            div()
                                .absolute()
                                .left(handle_inset)
                                .top(handle_inset)
                                .w(frame_size.width)
                                .h(frame_size.height)
                                .children(self.mesh_points.iter().enumerate().map(|(point_index, point)| {
                                    render_mesh_handle(
                                        self.selected_tab == BuilderTab::Freeform,
                                        point_index,
                                        *point,
                                        self.selected_mesh_point == Some(point_index),
                                        cx.entity(),
                                    )
                                })),
                        )
                    });
                let preview = div()
                    .flex_1()
                    .min_w(px(0.0))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .corner_radii(preview_corner_radii)
                    .p(px(12.0))
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
            .overflow_hidden()
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .py(px(6.0))
                    .bg(card_bg)
                    .child(div().w(px(300.0)).child(top_tabs))
                    .child(div().text_size(px(11.0)).text_color(chrome.muted_text).child("Untitled gradient")),
            )
            .child(div().h(px(1.0)).w_full().flex_shrink_0().bg(chrome.border))
            .child(content)
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .h(px(28.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_t_1()
                    .border_color(chrome.border)
                    .bg(chrome.panel_background)
                    .text_size(px(10.0))
                    .text_color(chrome.muted_text)
                    .child(self.export_status.clone().unwrap_or_else(|| {
                        format!(
                            "{} {}",
                            if self.selected_tab == BuilderTab::Gradients {
                                stops.len()
                            } else {
                                self.mesh_points.len()
                            },
                            if self.selected_tab == BuilderTab::Gradients {
                                "stops"
                            } else {
                                "points"
                            }
                        )
                    }))
                    .child(format!(
                        "{:.0} × {:.0}  ·  sRGB",
                        self.preview_size.width.as_f32(),
                        self.preview_size.height.as_f32()
                    )),
            );

        div()
            .id("color-viz-gradient-builder")
            .size_full()
            .min_h_0()
            .flex()
            .items_stretch()
            .justify_start()
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
    _label_style: LumaTextStyle,
    value_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let background = if selected {
        chrome.border
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
        .p(px(6.0))
        .rounded(px(4.0))
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
                .size(px(22.0))
                .rounded(px(4.0))
                .bg(color)
                .border_2()
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
                .child(div().flex().flex_col().gap_1().child(
                    div().typography_style(value_style).text_color(chrome.body_text).child(format_hex_color(color)),
                ))
                .child(
                    div().flex().flex_col().gap_1().child(
                        div()
                            .typography_style(value_style)
                            .text_color(chrome.body_text)
                            .child(format!("Position {}", format_percent(position))),
                    ),
                ),
        )
        .when_some(delete_button, |row, button| row.child(button))
}

#[allow(clippy::too_many_arguments)]
fn render_mesh_controls(
    image_adjustments: gpui::AnyElement,
    freeform: bool,
    freeform_buttons: Vec<Entity<Button>>,
    spread_slider: Entity<SliderControl>,
    spread_percent: f32,
    blend_selector: Entity<Selector>,
    point_spread_slider: Entity<SliderControl>,
    point_spread: f32,
    softness_slider: Entity<SliderControl>,
    softness_percent: f32,
    mesh_points: Vec<MeshPoint>,
    mesh_background: gpui::Hsla,
    selected_mesh_point: Option<usize>,
    mesh_grid_preset: MeshGridPreset,
    mesh_grid_selector: Entity<Selector>,
    mesh_aspect_ratio_preset: MeshAspectRatioPreset,
    mesh_aspect_ratio_selector: Entity<Selector>,
    mesh_reset_button: Entity<Button>,
    chrome: gpui_luma::theme::LumaChrome,
    card_bg: gpui::Hsla,
    info_label_style: LumaTextStyle,
    _info_value_style: LumaTextStyle,
    _preview_size: Size<Pixels>,
    _last_render_ms: Option<f32>,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let editing = selected_mesh_point
        .map(|index| format!("Editing point {}", index + 1))
        .unwrap_or_else(|| "Select a point".to_string());
    let panel = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .pb(px(20.0))
        .bg(chrome.panel_background)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(section_heading("CANVAS", chrome))
                .child(mesh_reset_button),
        )
        .child(
            div()
                .flex()
                .gap_2()
                .child(div().flex_1().min_w_0().child(if freeform {
                    form_field!("Blend", chrome; blend_selector).into_any_element()
                } else {
                    form_field!("Grid", chrome; mesh_grid_selector).into_any_element()
                }))
                .child(div().flex_1().min_w_0().child(form_field!("Aspect", chrome; mesh_aspect_ratio_selector))),
        )
        .child(render_mesh_background_row(mesh_background, info_label_style, chrome, builder.clone()));
    let fields = div()
        .w_full()
        .flex()
        .flex_col()
        .gap_2()
        .pt(px(24.0))
        .pb(px(12.0))
        .bg(chrome.panel_background)
        .child(section_heading("FIELDS", chrome))
        .when(freeform, |this| {
            this.child(div().flex().items_center().gap_2().children(freeform_buttons))
                .child(compact_slider_row(
                    "Spread · All",
                    spread_slider,
                    format!("{spread_percent:.0}%"),
                    info_label_style,
                    chrome,
                ))
                .child(compact_slider_row(
                    "Softness · All",
                    softness_slider,
                    format!("{softness_percent:.0}%"),
                    info_label_style,
                    chrome,
                ))
                .child(compact_slider_row(
                    "Point spread",
                    point_spread_slider,
                    selected_mesh_point.map(|_| format!("{point_spread:.0}%")).unwrap_or_else(|| "—".to_string()),
                    info_label_style,
                    chrome,
                ))
                .child(div().typography_style(info_label_style).text_color(chrome.muted_text).child(editing))
        });
    let points = div().w_full().flex().flex_col().gap_1().p_2().bg(chrome.panel_background).children(
        mesh_points.into_iter().enumerate().map(|(index, point)| {
            render_mesh_point_row(
                freeform,
                index,
                point,
                selected_mesh_point == Some(index),
                info_label_style,
                chrome,
                builder.clone(),
            )
        }),
    );
    div()
        .id("color-viz-mesh-controls")
        .w(px(INSPECTOR_WIDTH))
        .flex_shrink_0()
        .h_full()
        .min_h_0()
        .flex()
        .flex_col()
        .gap_2()
        .corner_radii(controls_panel_corner_radii())
        .overflow_y_scroll()
        .px(px(12.0))
        .pb(px(12.0))
        .bg(card_bg)
        .child(panel)
        .child(fields)
        .child(points)
        .child(image_adjustments)
        .child(div().text_size(px(10.0)).text_color(chrome.muted_text).child(if freeform {
            "Click canvas to deselect · Right-click to hide points".to_string()
        } else {
            format!("{} · {} · Right-click to hide guides", mesh_grid_preset.label(), mesh_aspect_ratio_preset.label())
        }))
        .into_any_element()
}

fn render_mesh_background_row(
    background: gpui::Hsla,
    label_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
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
        .py(px(8.0))
        .bg(chrome.panel_background)
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
                .size(px(22.0))
                .rounded(px(4.0))
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
    _freeform: bool,
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    label_style: LumaTextStyle,
    chrome: gpui_luma::theme::LumaChrome,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let bounds_builder = builder.clone();
    let swatch_builder = builder.clone();
    let background = if selected {
        chrome.border
    } else {
        chrome.panel_background
    };

    div()
        .id(format!("color-viz-mesh-point-row-{point_index}"))
        .w_full()
        .flex()
        .items_center()
        .gap_3()
        .p(px(6.0))
        .rounded(px(4.0))
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
                    this.sync_point_spread(cx);
                    this.mesh_color_target = MeshColorTarget::Point(point_index);
                    this.color_picker_open = false;
                    cx.notify();
                }
            });
        })
        .child(
            div()
                .id(format!("color-viz-mesh-point-swatch-{point_index}"))
                .size(px(22.0))
                .rounded(px(4.0))
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
                    div().flex().flex_col().gap_1().child(
                        div()
                            .typography_style(label_style)
                            .text_color(chrome.body_text)
                            .child((point_index + 1).to_string()),
                    ),
                )
                .child(
                    div().flex().flex_col().gap_1().child(
                        div()
                            .typography_style(label_style)
                            .text_color(chrome.muted_text)
                            .child(format!("X {}", format_percent(point.u))),
                    ),
                )
                .child(
                    div().flex().flex_col().gap_1().child(
                        div()
                            .typography_style(label_style)
                            .text_color(chrome.muted_text)
                            .child(format!("Y {}", format_percent(point.v))),
                    ),
                ),
        )
}

fn render_mesh_handle(
    _freeform: bool,
    point_index: usize,
    point: MeshPoint,
    selected: bool,
    builder: Entity<GradientBuilder>,
) -> impl IntoElement {
    let border = if selected {
        gpui::hsla(0.0, 0.0, 1.0, 1.0)
    } else {
        gpui::hsla(0.0, 0.0, 1.0, 0.6)
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
        .bg(gpui::hsla(0.0, 0.0, 0.0, if selected { 0.18 } else { 0.06 }))
        .border_1()
        .when(selected, |this| this.border_2())
        .border_color(border)
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::hsla(0.0, 0.0, 1.0, 1.0))
        .flex_col()
        .gap(px(3.0))
        .text_size(px(10.0))
        .child(div().size(px(6.0)).rounded(px(999.0)).bg(gpui::hsla(0.0, 0.0, 1.0, 1.0)))
        .child((point_index + 1).to_string())
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
                this.sync_point_spread(cx);
                this.mesh_color_target = MeshColorTarget::Point(point_index);
                this.color_picker_open = false;
                cx.notify();
            });
        })
}
