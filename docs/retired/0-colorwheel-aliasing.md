Created At: 2026-06-21T02:26:00Z
Completed At: 2026-06-21T02:26:00Z
File Path: `file:///Users/scg/Developer/GitHub/gpui-luma/docs/ai/issues/0-colorwheel-aliasing.md`

# Issue #0-colorwheel-aliasing: Smooth Circular Edges on the Hue-Saturation Wheel

## Description
When rendering the Hue-Saturation wheel in the 2D color field picker, the outer perimeter of the circle displays jagged, step-like artifacts ("teeth"). Colored pixels noticeably bleed past the smooth, antialiased black border path.

### The Root Cause
In [paint_raster.rs](file:///Users/scg/Developer/GitHub/gpui-luma/crates/sdk/src/controls/color/color_field/field/paint_raster.rs), `rasterize_hue_saturation_wheel_image` checks if pixels fall inside the circle boundary using a binary (in/out) distance check at the center of each pixel:

```rust
let dx = x as f32 + 0.5 - center_x;
let dy = y as f32 + 0.5 - center_y;
if dx * dx + dy * dy > max_radius_sq {
    continue; // Pixels outside are skipped (alpha = 0), inside are solid (alpha = 255)
}
```

Because there are no intermediate alpha values (fractional opacity/coverage) computed for boundary pixels, the resulting texture has stair-stepped edge pixels. When GPUI overlays the thin antialiased black outline path on top, these jagged colored pixels stick out.

---

## Suggested Solution: 4x Sub-Pixel Supersampling (SSAA)

To resolve this issue, `rasterize_hue_saturation_wheel_image` must be updated to calculate the circle boundary coverage using a 4x sub-pixel sampling grid (using `0.25` and `0.75` coordinate offsets). This matches the supersampling pattern already used for rectangular domains.

### Proposed Code for `crates/sdk/src/controls/color/color_field/field/paint_raster.rs`

```rust
fn rasterize_hue_saturation_wheel_image(
    size: Size<Pixels>,
    model: &dyn ColorFieldModel2D,
    hsv: Hsv,
) -> Option<Arc<Image>> {
    let scale = raster_scale_for_size(size);
    let width = (size.width.as_f32() * scale).round() as u32;
    let height = (size.height.as_f32() * scale).round() as u32;
    if width == 0 || height == 0 {
        return None;
    }

    let mut pixmap = Pixmap::new(width, height)?;
    let pixels = pixmap.pixels_mut();
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let max_radius_sq = (width.min(height) as f32 * 0.5).powi(2);

    for y in 0..height {
        for x in 0..width {
            let mut r_sum = 0.0_f32;
            let mut g_sum = 0.0_f32;
            let mut b_sum = 0.0_f32;
            let mut a_sum = 0.0_f32;
            let mut covered = 0_u8;

            for sy in [0.25_f32, 0.75_f32] {
                for sx in [0.25_f32, 0.75_f32] {
                    let sample_x = x as f32 + sx;
                    let sample_y = y as f32 + sy;
                    let dx = sample_x - center_x;
                    let dy = sample_y - center_y;

                    if dx * dx + dy * dy <= max_radius_sq {
                        let uv = (
                            (sample_x / width as f32).clamp(0.0, 1.0),
                            (sample_y / height as f32).clamp(0.0, 1.0),
                        );
                        let hsla = model.color_at_uv(&hsv, uv);
                        let rgb = hsla.to_rgb();
                        let alpha = hsla.a.clamp(0.0, 1.0);
                        r_sum += rgb.r * alpha;
                        g_sum += rgb.g * alpha;
                        b_sum += rgb.b * alpha;
                        a_sum += alpha;
                        covered += 1;
                    }
                }
            }

            if covered > 0 {
                let inv = 0.25_f32; // Average over the 4 grid positions
                let r_u8 = ((r_sum * inv).clamp(0.0, 1.0) * 255.0).round() as u8;
                let g_u8 = ((g_sum * inv).clamp(0.0, 1.0) * 255.0).round() as u8;
                let b_u8 = ((b_sum * inv).clamp(0.0, 1.0) * 255.0).round() as u8;
                let a_u8 = ((a_sum * inv).clamp(0.0, 1.0) * 255.0).round() as u8;

                if let Some(pixel) = PremultipliedColorU8::from_rgba(r_u8, g_u8, b_u8, a_u8) {
                    pixels[(y * width + x) as usize] = pixel;
                }
            }
        }
    }

    let png_data = pixmap.encode_png().ok()?;
    Some(Arc::new(Image::from_bytes(ImageFormat::Png, png_data)))
}
```

---

## Migration Plan
1. Update `rasterize_hue_saturation_wheel_image` in `crates/sdk/src/controls/color/color_field/field/paint_raster.rs`.
2. Clear/invalidate the color wheel cache on render to force a regenerate of the texture.
3. Verify that the outer boundary displays smooth antialiased transparency and aligns perfectly with the overlay border.
