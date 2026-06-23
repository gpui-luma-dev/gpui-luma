use gpui::*;

use super::color_spec::{self, ColorChannel, ColorSpecification};
use super::types::{Axis, ColorInterpolation, ColorSliderDelegate};
use super::track_context::{ColorSliderTrackContext, value_at_position};
use crate::controls::color::checkerboard_paint::paint_masked_checkerboard;

/// Calculates the start offset and size of a segment with a slight overlap to prevent sub-pixel gaps.
///
/// This is used because rendering multiple adjacent containers (especially with gradients)
/// can often leave a thin gap between them due to anti-aliasing or sub-pixel positioning.
/// By overlapping the segments slightly (2.0px), we ensure a smooth transition.
fn calculate_overlapping_segment(
    index: usize,
    count: usize,
    item_size: Pixels,
    total_size: Pixels,
) -> (Pixels, Pixels) {
    let start_offset = index as f32 * item_size;
    let end_offset = if index == count - 1 {
        total_size
    } else {
        (index + 1) as f32 * item_size + gpui::px(2.0)
    };
    (start_offset, end_offset - start_offset)
}

fn axis_total_size(bounds: Bounds<Pixels>, axis: Axis) -> Pixels {
    if axis == Axis::Vertical {
        bounds.size.height
    } else {
        bounds.size.width
    }
}

fn axis_default_radius(bounds: Bounds<Pixels>, axis: Axis) -> Pixels {
    if axis == Axis::Vertical {
        bounds.size.width / 2.0
    } else {
        bounds.size.height / 2.0
    }
}

fn axis_gradient_angle(axis: Axis) -> f32 {
    if axis == Axis::Vertical { 180.0 } else { 90.0 }
}

fn axis_segment_bounds(
    bounds: Bounds<Pixels>,
    axis: Axis,
    start_offset: Pixels,
    segment_size: Pixels,
) -> Bounds<Pixels> {
    let (origin, size) = if axis == Axis::Vertical {
        (point(bounds.origin.x, bounds.origin.y + start_offset), size(bounds.size.width, segment_size))
    } else {
        (point(bounds.origin.x + start_offset, bounds.origin.y), size(segment_size, bounds.size.height))
    };

    Bounds { origin, size }
}

fn apply_edge_corner_radii(
    corner_radii: &mut Corners<Pixels>,
    axis: Axis,
    radius: Pixels,
    is_first: bool,
    is_last: bool,
) {
    if is_first {
        if axis == Axis::Vertical {
            corner_radii.top_left = radius;
            corner_radii.top_right = radius;
        } else {
            corner_radii.top_left = radius;
            corner_radii.bottom_left = radius;
        }
    }

    if is_last {
        if axis == Axis::Vertical {
            corner_radii.bottom_left = radius;
            corner_radii.bottom_right = radius;
        } else {
            corner_radii.top_right = radius;
            corner_radii.bottom_right = radius;
        }
    }
}

fn effective_channel_range(ctx: &ColorSliderTrackContext, channel: ColorChannel) -> (f32, f32) {
    let slider_range = (ctx.range.start, ctx.range.end);
    let is_default_slider_range =
        (slider_range.0 - 0.0).abs() <= f32::EPSILON && (slider_range.1 - 1.0).abs() <= f32::EPSILON;
    let is_unit_channel = (channel.min - 0.0).abs() <= f32::EPSILON && (channel.max - 1.0).abs() <= f32::EPSILON;

    if is_default_slider_range && !is_unit_channel {
        (channel.min, channel.max)
    } else {
        (ctx.range.start.min(ctx.range.end), ctx.range.start.max(ctx.range.end))
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HueDelegate;

fn paint_hue_spectrum(context: &ColorSliderTrackContext, bounds: Bounds<Pixels>, window: &mut Window) {
    let reversed = context.reversed;
    let axis = context.axis;
    let style_corner_radii = context.corner_radii.clone();
    let rem_size = window.rem_size();
    let total_size = axis_total_size(bounds, axis);
    let band_size = total_size / 6.0;
    let default_radius = axis_default_radius(bounds, axis);
    let corner_radius = style_corner_radii.top_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius);

    let mut bands = [
        (hsla(0.0, 1.0, 0.5, 1.0), hsla(60.0 / 360.0, 1.0, 0.5, 1.0)),
        (hsla(60.0 / 360.0, 1.0, 0.5, 1.0), hsla(120.0 / 360.0, 1.0, 0.5, 1.0)),
        (hsla(120.0 / 360.0, 1.0, 0.5, 1.0), hsla(180.0 / 360.0, 1.0, 0.5, 1.0)),
        (hsla(180.0 / 360.0, 1.0, 0.5, 1.0), hsla(240.0 / 360.0, 1.0, 0.5, 1.0)),
        (hsla(240.0 / 360.0, 1.0, 0.5, 1.0), hsla(300.0 / 360.0, 1.0, 0.5, 1.0)),
        (hsla(300.0 / 360.0, 1.0, 0.5, 1.0), hsla(1.0, 1.0, 0.5, 1.0)),
    ];

    if reversed {
        bands.reverse();
        for band in bands.iter_mut() {
            let (start, end) = *band;
            *band = (end, start);
        }
    }

    for (i, (start_color, end_color)) in bands.iter().enumerate() {
        let (start_offset, current_band_size) = calculate_overlapping_segment(i, 6, band_size, total_size);
        let band_bounds = axis_segment_bounds(bounds, axis, start_offset, current_band_size);
        let mut corner_radii = Corners::default();
        apply_edge_corner_radii(&mut corner_radii, axis, corner_radius, i == 0, i == bands.len() - 1);
        let angle = axis_gradient_angle(axis);

        window.paint_quad(PaintQuad {
            bounds: band_bounds,
            corner_radii,
            background: linear_gradient(
                angle,
                linear_color_stop(*start_color, 0.0),
                linear_color_stop(*end_color, 1.0),
            ),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: BorderStyle::default(),
        });
    }
}

impl ColorSliderDelegate for HueDelegate {
    fn paint_domain_track(&self, context: &ColorSliderTrackContext, bounds: Bounds<Pixels>, window: &mut Window) {
        paint_hue_spectrum(context, bounds, window);
    }

    fn get_color_for_context(&self, context: &ColorSliderTrackContext, position: f32) -> Hsla {
        let hue = value_at_position(context, position);
        gpui::hsla(hue / 360.0, 1.0, 0.5, 1.0)
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub struct GradientDelegate {
    pub colors: Vec<Hsla>,
}

fn gradient_colors(delegate: &GradientDelegate, context: &ColorSliderTrackContext) -> Vec<Hsla> {
    if context.reversed {
        let mut colors = delegate.colors.clone();
        colors.reverse();
        colors
    } else {
        delegate.colors.clone()
    }
}

fn paint_gradient_spectrum(
    delegate: &GradientDelegate,
    context: &ColorSliderTrackContext,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let axis = context.axis;
    let interpolation = context.interpolation;
    let colors = gradient_colors(delegate, context);

    if colors.len() <= 1 {
        if let Some(color) = colors.first() {
            window.paint_quad(PaintQuad {
                bounds,
                corner_radii: Corners::all(axis_default_radius(bounds, axis)),
                background: (*color).into(),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
        }
        return;
    }

    let rem_size = window.rem_size();
    let total_size = axis_total_size(bounds, axis);
    let segment_count = colors.len() - 1;
    let segment_size = total_size / segment_count as f32;
    let default_radius = axis_default_radius(bounds, axis);
    let corner_radius = context.corner_radii.top_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius);

    for i in 0..segment_count {
        let start_color = colors[i];
        let end_color = colors[i + 1];
        let (start_offset, current_segment_size) =
            calculate_overlapping_segment(i, segment_count, segment_size, total_size);
        let band_bounds = axis_segment_bounds(bounds, axis, start_offset, current_segment_size);
        let mut corner_radii = Corners::default();
        apply_edge_corner_radii(&mut corner_radii, axis, corner_radius, i == 0, i == segment_count - 1);
        let angle = axis_gradient_angle(axis);

        if interpolation == ColorInterpolation::Rgb {
            window.paint_quad(PaintQuad {
                bounds: band_bounds,
                corner_radii,
                background: linear_gradient(
                    angle,
                    linear_color_stop(start_color, 0.0),
                    linear_color_stop(end_color, 1.0),
                ),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
        } else {
            let sub_steps = 10;
            let sub_segment_size = segment_size / sub_steps as f32;

            for j in 0..sub_steps {
                let t_start = j as f32 / sub_steps as f32;
                let t_end = (j + 1) as f32 / sub_steps as f32;
                let sub_start_color = match interpolation {
                    ColorInterpolation::Hsl => color_spec::interpolate_hsl(start_color, end_color, t_start),
                    ColorInterpolation::Lab => color_spec::interpolate_lab(start_color, end_color, t_start),
                    ColorInterpolation::Rgb => unreachable!(),
                };
                let sub_end_color = match interpolation {
                    ColorInterpolation::Hsl => color_spec::interpolate_hsl(start_color, end_color, t_end),
                    ColorInterpolation::Lab => color_spec::interpolate_lab(start_color, end_color, t_end),
                    ColorInterpolation::Rgb => unreachable!(),
                };
                let (sub_start_offset, current_sub_size) =
                    calculate_overlapping_segment(j, sub_steps, sub_segment_size, segment_size);
                let sub_bounds = axis_segment_bounds(band_bounds, axis, sub_start_offset, current_sub_size);
                let mut sub_radii = Corners::default();
                apply_edge_corner_radii(
                    &mut sub_radii,
                    axis,
                    corner_radius,
                    i == 0 && j == 0,
                    i == segment_count - 1 && j == sub_steps - 1,
                );

                window.paint_quad(PaintQuad {
                    bounds: sub_bounds,
                    corner_radii: sub_radii,
                    background: linear_gradient(
                        angle,
                        linear_color_stop(sub_start_color, 0.0),
                        linear_color_stop(sub_end_color, 1.0),
                    ),
                    border_widths: Edges::default(),
                    border_color: transparent_black(),
                    border_style: BorderStyle::default(),
                });
            }
        }
    }
}

impl ColorSliderDelegate for GradientDelegate {
    fn paint_domain_track(&self, context: &ColorSliderTrackContext, bounds: Bounds<Pixels>, window: &mut Window) {
        paint_gradient_spectrum(self, context, bounds, window);
    }

    fn get_color_for_context(&self, context: &ColorSliderTrackContext, position: f32) -> Hsla {
        let pct = if context.reversed { 1.0 - position } else { position };

        if self.colors.is_empty() {
            return hsla(0.0, 0.0, 0.0, 1.0);
        }
        if self.colors.len() == 1 {
            return self.colors[0];
        }

        let scaled_pct = pct * (self.colors.len() - 1) as f32;
        let idx = scaled_pct.floor() as usize;
        let next_idx = (idx + 1).min(self.colors.len() - 1);
        let t = scaled_pct - idx as f32;

        let start = self.colors[idx];
        let end = self.colors[next_idx];

        match context.interpolation {
            ColorInterpolation::Rgb => color_spec::interpolate_rgb(start, end, t),
            ColorInterpolation::Hsl => color_spec::interpolate_hsl(start, end, t),
            ColorInterpolation::Lab => color_spec::interpolate_lab(start, end, t),
        }
    }
}

pub struct AlphaDelegate<S: ColorSpecification> {
    pub spec: S,
}

fn paint_alpha_spectrum<S: ColorSpecification>(
    delegate: &AlphaDelegate<S>,
    context: &ColorSliderTrackContext,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let axis = context.axis;
    let (start, end) = if context.reversed { (1.0, 0.0) } else { (0.0, 1.0) };
    let is_dark = context.theme_is_dark;
    let rem_size = window.rem_size();
    let default_radius = axis_default_radius(bounds, axis);
    let radius = context.corner_radii.top_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius);

    let mut opaque_color = delegate.spec.to_hsla();
    opaque_color.a = 1.0;
    let mut transparent_color = opaque_color;
    transparent_color.a = 0.0;

    let (c1, c2) = if is_dark {
        (hsla(0., 0., 0.1, 1.), hsla(0., 0., 0.13, 1.))
    } else {
        (hsla(0., 0., 1.0, 1.), hsla(0., 0., 0.95, 1.))
    };

    window.paint_quad(PaintQuad {
        bounds,
        corner_radii: Corners::all(radius),
        background: c1.into(),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::default(),
    });

    paint_masked_checkerboard(window, bounds, radius, c2, true, px(color_spec::constants::CHECKERBOARD_SIZE));

    let angle = axis_gradient_angle(axis);
    window.paint_quad(PaintQuad {
        bounds,
        corner_radii: Corners::all(radius),
        background: linear_gradient(
            angle,
            linear_color_stop(transparent_color, start),
            linear_color_stop(opaque_color, end),
        ),
        border_widths: Edges::default(),
        border_color: transparent_black(),
        border_style: BorderStyle::default(),
    });
}

impl<S: ColorSpecification> ColorSliderDelegate for AlphaDelegate<S> {
    fn paint_domain_track(&self, context: &ColorSliderTrackContext, bounds: Bounds<Pixels>, window: &mut Window) {
        paint_alpha_spectrum(self, context, bounds, window);
    }

    fn get_color_for_context(&self, context: &ColorSliderTrackContext, position: f32) -> Hsla {
        let alpha = value_at_position(context, position);
        let mut color = self.spec.to_hsla();
        color.a = alpha;
        color
    }
}

pub struct ChannelDelegate<S: ColorSpecification> {
    spec: S,
    channel_name: SharedString,
}

impl<S: ColorSpecification> ChannelDelegate<S> {
    pub fn new(spec: S, channel_name: SharedString) -> Result<Self, String> {
        // Validate channel exists
        if !spec.channels().iter().any(|c| c.name == channel_name.as_ref()) {
            return Err(format!("Channel '{}' not found in {:?}", channel_name, spec.name()));
        }
        Ok(Self { spec, channel_name })
    }

    fn resolve_channel(&self) -> Option<ColorChannel> {
        self.spec.channels().iter().find(|c| c.name == self.channel_name.as_ref()).copied()
    }
}

fn paint_channel_spectrum<S: ColorSpecification>(
    delegate: &ChannelDelegate<S>,
    context: &ColorSliderTrackContext,
    bounds: Bounds<Pixels>,
    window: &mut Window,
) {
    let Some(channel) = delegate.resolve_channel() else {
        return;
    };

    let axis = context.axis;
    let reversed = context.reversed;
    let interpolation = context.interpolation;
    let (range_min, range_max) = effective_channel_range(context, channel);

    if interpolation == ColorInterpolation::Rgb {
        let (start, end) = if reversed { (1.0, 0.0) } else { (0.0, 1.0) };
        let mut start_spec = delegate.spec;
        start_spec.set_value(delegate.channel_name.as_ref(), range_min);
        let mut end_spec = delegate.spec;
        end_spec.set_value(delegate.channel_name.as_ref(), range_max);
        let angle = axis_gradient_angle(axis);
        window.paint_quad(PaintQuad {
            bounds,
            corner_radii: Corners::all(
                context
                    .corner_radii
                    .top_left
                    .map(|v| v.to_pixels(window.rem_size()))
                    .unwrap_or(axis_default_radius(bounds, axis)),
            ),
            background: linear_gradient(
                angle,
                linear_color_stop(start_spec.to_hsla(), start),
                linear_color_stop(end_spec.to_hsla(), end),
            ),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: BorderStyle::default(),
        });
        return;
    }

    let rem_size = window.rem_size();
    let total_size = axis_total_size(bounds, axis);
    let segment_count = 10;
    let segment_size = total_size / segment_count as f32;
    let default_radius = axis_default_radius(bounds, axis);
    let corner_radius = context.corner_radii.top_left.map(|v| v.to_pixels(rem_size)).unwrap_or(default_radius);
    let spec = delegate.spec;
    let channel_name = delegate.channel_name.clone();

    for i in 0..segment_count {
        let t_start = i as f32 / segment_count as f32;
        let t_end = (i + 1) as f32 / segment_count as f32;
        let spec_t_start = if reversed { 1.0 - t_start } else { t_start };
        let spec_t_end = if reversed { 1.0 - t_end } else { t_end };
        let val_start = range_min + (range_max - range_min) * spec_t_start;
        let val_end = range_min + (range_max - range_min) * spec_t_end;
        let mut start_spec = spec;
        start_spec.set_value(channel_name.as_ref(), val_start);
        let start_color = start_spec.to_hsla();
        let mut end_spec = spec;
        end_spec.set_value(channel_name.as_ref(), val_end);
        let end_color = end_spec.to_hsla();
        let (start_offset, current_segment_size) =
            calculate_overlapping_segment(i, segment_count, segment_size, total_size);
        let band_bounds = axis_segment_bounds(bounds, axis, start_offset, current_segment_size);
        let mut corner_radii = Corners::default();
        apply_edge_corner_radii(&mut corner_radii, axis, corner_radius, i == 0, i == segment_count - 1);
        let angle = axis_gradient_angle(axis);

        window.paint_quad(PaintQuad {
            bounds: band_bounds,
            corner_radii,
            background: linear_gradient(angle, linear_color_stop(start_color, 0.0), linear_color_stop(end_color, 1.0)),
            border_widths: Edges::default(),
            border_color: transparent_black(),
            border_style: BorderStyle::default(),
        });
    }
}

impl<S: ColorSpecification> ColorSliderDelegate for ChannelDelegate<S> {
    fn paint_domain_track(&self, context: &ColorSliderTrackContext, bounds: Bounds<Pixels>, window: &mut Window) {
        paint_channel_spectrum(self, context, bounds, window);
    }

    fn get_color_for_context(&self, context: &ColorSliderTrackContext, position: f32) -> Hsla {
        let Some(channel) = self.resolve_channel() else {
            return self.spec.to_hsla();
        };
        let (range_min, range_max) = effective_channel_range(context, channel);
        let value = if context.reversed {
            range_max - (range_max - range_min) * position
        } else {
            range_min + (range_max - range_min) * position
        };
        let mut spec = self.spec;
        spec.set_value(self.channel_name.as_ref(), value);
        spec.to_hsla()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::color::color_slider::color_spec::Hsl;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-6, "expected {a} ~= {b}, delta={}", (a - b).abs());
    }

    #[::core::prelude::v1::test]
    fn overlapping_segment_adds_overlap_except_last_segment() {
        let item = px(10.0);
        let total = px(30.0);

        let (start0, size0) = calculate_overlapping_segment(0, 3, item, total);
        let (start1, size1) = calculate_overlapping_segment(1, 3, item, total);
        let (start2, size2) = calculate_overlapping_segment(2, 3, item, total);

        let start0: f32 = start0.into();
        let size0: f32 = size0.into();
        let start1: f32 = start1.into();
        let size1: f32 = size1.into();
        let start2: f32 = start2.into();
        let size2: f32 = size2.into();

        approx_eq(start0, 0.0);
        approx_eq(size0, 12.0);
        approx_eq(start1, 10.0);
        approx_eq(size1, 12.0);
        approx_eq(start2, 20.0);
        approx_eq(size2, 10.0);
    }

    #[::core::prelude::v1::test]
    fn overlapping_segment_uses_total_size_for_single_segment() {
        let (start, size) = calculate_overlapping_segment(0, 1, px(40.0), px(33.0));
        let start: f32 = start.into();
        let size: f32 = size.into();
        approx_eq(start, 0.0);
        approx_eq(size, 33.0);
    }

    #[::core::prelude::v1::test]
    fn channel_delegate_new_validates_channel_name() {
        let spec = Hsl { h: 0.0, s: 0.5, l: 0.5, a: 1.0 };

        let ok = ChannelDelegate::new(spec, Hsl::HUE.into());
        assert!(ok.is_ok());

        let err = ChannelDelegate::new(spec, "unknown".into());
        assert!(err.is_err());
        assert!(err.err().is_some_and(|message| message.contains("Channel 'unknown' not found")));
    }
}
