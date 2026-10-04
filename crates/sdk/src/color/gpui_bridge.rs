//! Conversion boundary for the current GPUI backend (encoded sRGB, straight alpha).
//! P3 source values remain in `ColorValue`; these functions only produce previews.

use std::collections::HashMap;
use anyhow::Result;
use gpui::{App, Global};
use super::{ColorValue, GamutMapping, SourceSpace};

/// Import existing backend colors at integrations that have not yet migrated.
pub fn from_rgba(color: gpui::Rgba) -> ColorValue {
    ColorValue::srgb(color.r, color.g, color.b, color.a)
}

/// GPUI hue fractions are converted by its sRGB conversion at this boundary.
pub fn from_hsla(color: gpui::Hsla) -> ColorValue {
    from_rgba(preview_rgba(color))
}

/// Adapt a Palette HSL editor preview without a round trip through RGB.
/// Preserves authored hue at black/gray and at the 360° endpoint. This is not
/// source storage or gamut mapping; the editor is responsible for its bounds.
pub fn from_palette_hsla(color: palette::Hsla) -> gpui::Hsla {
    gpui::Hsla { h: color.hue.into_raw_degrees() / 360.0, s: color.saturation, l: color.lightness, a: color.alpha }
}

/// Import backend HSL preview components, converting turns to degrees.
/// For canonical source ingress use `from_hsla` instead.
pub fn to_palette_hsla(color: gpui::Hsla) -> palette::Hsla {
    palette::Hsla::new(color.h * 360.0, color.s, color.l, color.a)
}

pub fn to_rgba(color: ColorValue, policy: GamutMapping) -> Result<gpui::Rgba> {
    let c = color.to_srgba_fallback(policy)?;
    Ok(gpui::Rgba { r: c.red, g: c.green, b: c.blue, a: c.alpha })
}

pub fn to_hsla(color: ColorValue, policy: GamutMapping) -> Result<gpui::Hsla> {
    Ok(preview_hsla(to_rgba(color, policy)?))
}

/// Convert an already prepared GPUI preview for raster output or preview interpolation.
/// This performs no source-space conversion or gamut mapping. Alpha remains straight.
pub fn preview_rgba(color: gpui::Hsla) -> gpui::Rgba {
    color.to_rgb()
}

/// Convert an already prepared RGB preview to the backend's HSL representation.
/// GPUI's f32 conversion can put saturation a few ULPs above one, even
/// from bounded RGB. Bound only this final representation, never source data.
pub fn preview_hsla(rgb: gpui::Rgba) -> gpui::Hsla {
    let mut color: gpui::Hsla = rgb.into();
    color.s = color.s.clamp(0.0, 1.0);
    color
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct CacheKey {
    space: SourceSpace,
    components: [u32; 4],
    policy: GamutMapping,
}

/// Bounded cache for this fixed sRGB backend. Source and mapping changes invalidate
/// entries by identity. A future P3/profile-aware backend must include its output
/// configuration in the key rather than reuse this sRGB cache.
#[derive(Default)]
pub struct SrgbRenderCache {
    entries: HashMap<CacheKey, gpui::Rgba>,
}

impl Global for SrgbRenderCache {}

impl SrgbRenderCache {
    pub fn resolve(&mut self, color: ColorValue, policy: GamutMapping) -> Result<gpui::Rgba> {
        color.validate()?;
        let (space, components) = color.components();
        let key = CacheKey { space, components: components.map(f32::to_bits), policy };
        if let Some(value) = self.entries.get(&key) {
            return Ok(*value);
        }
        let value = to_rgba(color, policy)?;
        if self.entries.len() >= 256 {
            self.entries.clear();
        }
        self.entries.insert(key, value);
        Ok(value)
    }

    /// Cached sRGB preview with GPUI's HSL rounding bounded at this boundary.
    pub fn resolve_hsla(&mut self, color: ColorValue, policy: GamutMapping) -> Result<gpui::Hsla> {
        self.resolve(color, policy).map(preview_hsla)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Resolve once per source/policy, sharing the bounded cache across UI elements.
pub fn cached_rgba(color: ColorValue, policy: GamutMapping, cx: &mut App) -> Result<gpui::Rgba> {
    cx.default_global::<SrgbRenderCache>().resolve(color, policy)
}
