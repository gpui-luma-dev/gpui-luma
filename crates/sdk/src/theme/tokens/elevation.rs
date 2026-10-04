use gpui::{BoxShadow, Hsla, point, px};
use crate::color::{ColorValue, GamutMapping, gpui_bridge::SrgbRenderCache};
use anyhow::{Context as _, Result};
use serde::{Serialize, Deserialize};
use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LumaElevation<C = ColorValue> {
    pub none: LumaShadow<C>,
    pub control: LumaShadow<C>,
    pub thumb: LumaShadow<C>,
    pub menu: LumaShadow<C>,
    pub popover: LumaShadow<C>,
    pub panel: LumaShadow<C>,
    pub dialog: LumaShadow<C>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LumaShadow<C = ColorValue> {
    pub layers: Vec<LumaShadowLayer<C>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct LumaShadowLayer<C = ColorValue> {
    pub color: C,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

impl Default for LumaElevation {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaElevation<ColorValue> {
    pub fn light() -> Self {
        Self::light_with(|a| ColorValue::srgb(0.0, 0.0, 0.0, a))
    }
    pub fn dark() -> Self {
        Self::dark_with(|a| ColorValue::srgb(0.0, 0.0, 0.0, a))
    }

    /// Prepare current GPUI previews once while retaining original shadow colors.
    pub fn snapshot_srgb(self, policy: GamutMapping) -> Result<SrgbElevation> {
        let mut cache = SrgbRenderCache::default();
        let resolve = |shadow: &LumaShadow, name: &str, cache: &mut SrgbRenderCache| -> Result<LumaShadow<Hsla>> {
            let layers = shadow
                .layers
                .iter()
                .enumerate()
                .map(|(index, layer)| {
                    anyhow::ensure!(
                        [layer.offset_x, layer.offset_y, layer.blur, layer.spread].iter().all(|v| v.is_finite())
                            && layer.blur >= 0.0,
                        "invalid shadow geometry at {name}[{index}]"
                    );
                    Ok(LumaShadowLayer::new(
                        cache.resolve_hsla(layer.color, policy).with_context(|| format!("shadow {name}[{index}]"))?,
                        layer.offset_x,
                        layer.offset_y,
                        layer.blur,
                        layer.spread,
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(LumaShadow::new(layers))
        };
        let preview = LumaElevation {
            none: resolve(&self.none, "none", &mut cache)?,
            control: resolve(&self.control, "control", &mut cache)?,
            thumb: resolve(&self.thumb, "thumb", &mut cache)?,
            menu: resolve(&self.menu, "menu", &mut cache)?,
            popover: resolve(&self.popover, "popover", &mut cache)?,
            panel: resolve(&self.panel, "panel", &mut cache)?,
            dialog: resolve(&self.dialog, "dialog", &mut cache)?,
        };
        Ok(SrgbElevation { source: Arc::new(self), preview: Arc::new(preview), policy })
    }
}

impl<C> LumaElevation<C> {
    fn light_with(color: impl Fn(f32) -> C) -> Self {
        Self {
            none: LumaShadow::default(),
            control: LumaShadow::single(color(0.05), 0.0, 1.0, 2.0, 0.0),
            thumb: LumaShadow::single(color(0.14), 0.0, 1.0, 2.0, 0.0),
            menu: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.10), 0.0, 1.0, 3.0, 0.0),
                LumaShadowLayer::new(color(0.10), 0.0, 1.0, 2.0, -1.0),
            ]),
            popover: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.12), 0.0, 4.0, 8.0, -2.0),
                LumaShadowLayer::new(color(0.06), 0.0, 2.0, 4.0, -1.0),
            ]),
            panel: LumaShadow::single(color(0.06), 0.0, 1.0, 3.0, 0.0),
            dialog: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.16), 0.0, 18.0, 32.0, -8.0),
                LumaShadowLayer::new(color(0.08), 0.0, 6.0, 12.0, -4.0),
            ]),
        }
    }

    fn dark_with(color: impl Fn(f32) -> C) -> Self {
        Self {
            none: LumaShadow::default(),
            control: LumaShadow::single(color(0.18), 0.0, 1.0, 2.0, 0.0),
            thumb: LumaShadow::single(color(0.28), 0.0, 1.0, 2.0, 0.0),
            menu: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.28), 0.0, 6.0, 16.0, -6.0),
                LumaShadowLayer::new(color(0.18), 0.0, 2.0, 6.0, -2.0),
            ]),
            popover: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.34), 0.0, 10.0, 24.0, -8.0),
                LumaShadowLayer::new(color(0.18), 0.0, 2.0, 6.0, -2.0),
            ]),
            panel: LumaShadow::single(color(0.20), 0.0, 1.0, 3.0, 0.0),
            dialog: LumaShadow::new(vec![
                LumaShadowLayer::new(color(0.42), 0.0, 18.0, 32.0, -8.0),
                LumaShadowLayer::new(color(0.24), 0.0, 6.0, 12.0, -4.0),
            ]),
        }
    }
}

impl<C> Default for LumaShadow<C> {
    fn default() -> Self {
        Self { layers: Vec::new() }
    }
}

impl<C> LumaShadow<C> {
    pub fn new(layers: Vec<LumaShadowLayer<C>>) -> Self {
        Self { layers }
    }

    pub fn single(color: C, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self::new(vec![LumaShadowLayer::new(color, offset_x, offset_y, blur, spread)])
    }
}

impl LumaShadow<Hsla> {
    pub fn to_box_shadows(&self) -> Vec<BoxShadow> {
        self.layers.iter().map(LumaShadowLayer::to_box_shadow).collect()
    }
}

impl<C> LumaShadowLayer<C> {
    pub fn new(color: C, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self { color, offset_x, offset_y, blur, spread }
    }
}

impl LumaShadowLayer<Hsla> {
    pub fn to_box_shadow(&self) -> BoxShadow {
        BoxShadow {
            color: self.color,
            offset: point(px(self.offset_x), px(self.offset_y)),
            blur_radius: px(self.blur),
            spread_radius: px(self.spread),
            inset: false,
        }
    }
}

/// Immutable source shadows and their current sRGB rendering snapshot.
#[derive(Clone, Debug)]
pub struct SrgbElevation {
    source: Arc<LumaElevation>,
    preview: Arc<LumaElevation<Hsla>>,
    policy: GamutMapping,
}
impl SrgbElevation {
    pub fn source(&self) -> &LumaElevation {
        &self.source
    }
    pub fn mapping(&self) -> GamutMapping {
        self.policy
    }
    pub fn light() -> Self {
        Self {
            source: Arc::new(LumaElevation::light()),
            preview: Arc::new(LumaElevation::light_with(|a| gpui::hsla(0.0, 0.0, 0.0, a))),
            policy: GamutMapping::CssLocalMinde,
        }
    }
    pub fn dark() -> Self {
        Self {
            source: Arc::new(LumaElevation::dark()),
            preview: Arc::new(LumaElevation::dark_with(|a| gpui::hsla(0.0, 0.0, 0.0, a))),
            policy: GamutMapping::CssLocalMinde,
        }
    }
}
impl Deref for SrgbElevation {
    type Target = LumaElevation<Hsla>;
    fn deref(&self) -> &Self::Target {
        &self.preview
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_retains_extended_p3_shadow_source() {
        let mut source = LumaElevation::light();
        let color = ColorValue::display_p3(1.15, -0.1, 0.2, 0.234567);
        source.menu.layers[0].color = color;
        let snapshot = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(snapshot.source().menu.layers[0].color, color);
        assert_eq!(snapshot.menu.layers[0].color.a, color.alpha());
        let stored = toml::to_string(snapshot.source()).unwrap();
        let restored: LumaElevation = toml::from_str(&stored).unwrap();
        assert_eq!(restored, *snapshot.source());
    }
    #[test]
    fn invalid_shadow_color_is_rejected_with_role() {
        let mut source = LumaElevation::light();
        source.thumb.layers[0].color = ColorValue::srgb(f32::NAN, 0.0, 0.0, 1.0);
        assert!(source.snapshot_srgb(GamutMapping::Clip).unwrap_err().to_string().contains("thumb[0]"));
    }
}
