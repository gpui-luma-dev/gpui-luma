use gpui::Hsla;
use crate::color::{ColorValue, GamutMapping, gpui_bridge::SrgbRenderCache};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::{
    ops::Deref,
    sync::{Arc, OnceLock},
};

/// Semantic source colors; the default component type retains Palette color spaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LumaPalette<C = ColorValue> {
    pub app: AppPalette<C>,
    pub surface: SurfacePalette<C>,
    pub state: StatePalette<C>,
    pub form: FormPalette<C>,
    pub focus: FocusPalette<C>,
    pub border: BorderPalette<C>,
    pub navigation: NavigationPalette<C>,
    pub data: DataPalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub muted_foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfacePalette<C = ColorValue> {
    pub panel: SurfaceWithBorderPalette<C>,
    pub floating: SurfaceWithBorderPalette<C>,
    pub subtle: SurfaceTonePalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceWithBorderPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub border: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceTonePalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatePalette<C = ColorValue> {
    pub hover: StateTonePalette<C>,
    pub pressed: StateBackgroundPalette<C>,
    pub selected: StateTonePalette<C>,
    pub disabled: StateTonePalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateTonePalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StateBackgroundPalette<C = ColorValue> {
    pub background: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormPalette<C = ColorValue> {
    pub input: FormInputPalette<C>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormInputPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub border: C,
    pub invalid_border: C,
    pub placeholder: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FocusPalette<C = ColorValue> {
    pub ring: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BorderPalette<C = ColorValue> {
    pub default: C,
    pub strong: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NavigationPalette<C = ColorValue> {
    pub background: C,
    pub foreground: C,
    pub muted_foreground: C,
    pub hover_background: C,
    pub selected_background: C,
    pub selected_foreground: C,
    pub border: C,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataPalette<C = ColorValue> {
    pub accent_1: C,
    pub accent_2: C,
    pub accent_3: C,
    pub accent_4: C,
    pub accent_5: C,
}

impl Default for LumaPalette<ColorValue> {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaPalette<ColorValue> {
    pub fn light() -> Self {
        Self::light_with(hex_source)
    }
    pub fn dark() -> Self {
        Self::dark_with(hex_source)
    }

    /// Validate and retain the source palette, with an immutable sRGB preview.
    pub fn snapshot_srgb(self, policy: GamutMapping) -> Result<SrgbPalette> {
        let mut cache = SrgbRenderCache::default();
        let preview = self.clone().map_colors(&mut |source| cache.resolve_hsla(source, policy))?;
        Ok(SrgbPalette { source: Arc::new(self), preview: Arc::new(preview), policy })
    }
}

fn hex_source(hex: u32) -> ColorValue {
    ColorValue::srgb(
        ((hex >> 16) & 255) as f32 / 255.0,
        ((hex >> 8) & 255) as f32 / 255.0,
        (hex & 255) as f32 / 255.0,
        1.0,
    )
}

impl<C> LumaPalette<C> {
    fn light_with(color: impl Fn(u32) -> C) -> Self {
        Self {
            app: AppPalette {
                background: color(0xf8fafc),
                foreground: color(0x0f172a),
                muted_foreground: color(0x64748b),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: color(0xffffff),
                    foreground: color(0x0f172a),
                    border: color(0xcbd5e1),
                },
                floating: SurfaceWithBorderPalette {
                    background: color(0xffffff),
                    foreground: color(0x0f172a),
                    border: color(0xcbd5e1),
                },
                subtle: SurfaceTonePalette { background: color(0xf1f5f9), foreground: color(0x334155) },
            },
            state: StatePalette {
                hover: StateTonePalette { background: color(0xe2e8f0), foreground: color(0x0f172a) },
                pressed: StateBackgroundPalette { background: color(0xcbd5e1) },
                selected: StateTonePalette { background: color(0x2563eb), foreground: color(0xffffff) },
                disabled: StateTonePalette { background: color(0xf1f5f9), foreground: color(0x94a3b8) },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: color(0xffffff),
                    foreground: color(0x0f172a),
                    border: color(0x94a3b8),
                    invalid_border: color(0xdb2777),
                    placeholder: color(0x94a3b8),
                },
            },
            focus: FocusPalette { ring: color(0xf59e0b) },
            border: BorderPalette { default: color(0xcbd5e1), strong: color(0x64748b) },
            navigation: NavigationPalette {
                background: color(0xffffff),
                foreground: color(0x0f172a),
                muted_foreground: color(0x64748b),
                hover_background: color(0xf1f5f9),
                selected_background: color(0x2563eb),
                selected_foreground: color(0xffffff),
                border: color(0xcbd5e1),
            },
            data: DataPalette {
                accent_1: color(0x2563eb),
                accent_2: color(0x0f766e),
                accent_3: color(0xc2410c),
                accent_4: color(0x7c3aed),
                accent_5: color(0xbe123c),
            },
        }
    }

    fn dark_with(color: impl Fn(u32) -> C) -> Self {
        Self {
            app: AppPalette {
                background: color(0x0b1120),
                foreground: color(0xf8fafc),
                muted_foreground: color(0x94a3b8),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: color(0x111827),
                    foreground: color(0xf8fafc),
                    border: color(0x334155),
                },
                floating: SurfaceWithBorderPalette {
                    background: color(0x111827),
                    foreground: color(0xf8fafc),
                    border: color(0x334155),
                },
                subtle: SurfaceTonePalette { background: color(0x1e293b), foreground: color(0xcbd5e1) },
            },
            state: StatePalette {
                hover: StateTonePalette { background: color(0x334155), foreground: color(0xf8fafc) },
                pressed: StateBackgroundPalette { background: color(0x475569) },
                selected: StateTonePalette { background: color(0x60a5fa), foreground: color(0x082f49) },
                disabled: StateTonePalette { background: color(0x1e293b), foreground: color(0x64748b) },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: color(0x0f172a),
                    foreground: color(0xf8fafc),
                    border: color(0x475569),
                    invalid_border: color(0xf472b6),
                    placeholder: color(0x64748b),
                },
            },
            focus: FocusPalette { ring: color(0xfbbf24) },
            border: BorderPalette { default: color(0x334155), strong: color(0x94a3b8) },
            navigation: NavigationPalette {
                background: color(0x111827),
                foreground: color(0xf8fafc),
                muted_foreground: color(0x94a3b8),
                hover_background: color(0x1e293b),
                selected_background: color(0x60a5fa),
                selected_foreground: color(0x082f49),
                border: color(0x334155),
            },
            data: DataPalette {
                accent_1: color(0x60a5fa),
                accent_2: color(0x2dd4bf),
                accent_3: color(0xfb923c),
                accent_4: color(0xa78bfa),
                accent_5: color(0xfb7185),
            },
        }
    }
}

impl<C> LumaPalette<C> {
    fn map_colors<D>(self, map: &mut impl FnMut(C) -> Result<D>) -> Result<LumaPalette<D>> {
        Ok(LumaPalette {
            app: AppPalette {
                background: map(self.app.background).with_context(|| "palette.app.background")?,
                foreground: map(self.app.foreground).with_context(|| "palette.app.foreground")?,
                muted_foreground: map(self.app.muted_foreground).with_context(|| "palette.app.muted_foreground")?,
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: map(self.surface.panel.background)
                        .with_context(|| "palette.surface.panel.background")?,
                    foreground: map(self.surface.panel.foreground)
                        .with_context(|| "palette.surface.panel.foreground")?,
                    border: map(self.surface.panel.border).with_context(|| "palette.surface.panel.border")?,
                },
                floating: SurfaceWithBorderPalette {
                    background: map(self.surface.floating.background)
                        .with_context(|| "palette.surface.floating.background")?,
                    foreground: map(self.surface.floating.foreground)
                        .with_context(|| "palette.surface.floating.foreground")?,
                    border: map(self.surface.floating.border).with_context(|| "palette.surface.floating.border")?,
                },
                subtle: SurfaceTonePalette {
                    background: map(self.surface.subtle.background)
                        .with_context(|| "palette.surface.subtle.background")?,
                    foreground: map(self.surface.subtle.foreground)
                        .with_context(|| "palette.surface.subtle.foreground")?,
                },
            },
            state: StatePalette {
                hover: StateTonePalette {
                    background: map(self.state.hover.background).with_context(|| "palette.state.hover.background")?,
                    foreground: map(self.state.hover.foreground).with_context(|| "palette.state.hover.foreground")?,
                },
                pressed: StateBackgroundPalette {
                    background: map(self.state.pressed.background)
                        .with_context(|| "palette.state.pressed.background")?,
                },
                selected: StateTonePalette {
                    background: map(self.state.selected.background)
                        .with_context(|| "palette.state.selected.background")?,
                    foreground: map(self.state.selected.foreground)
                        .with_context(|| "palette.state.selected.foreground")?,
                },
                disabled: StateTonePalette {
                    background: map(self.state.disabled.background)
                        .with_context(|| "palette.state.disabled.background")?,
                    foreground: map(self.state.disabled.foreground)
                        .with_context(|| "palette.state.disabled.foreground")?,
                },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: map(self.form.input.background).with_context(|| "palette.form.input.background")?,
                    foreground: map(self.form.input.foreground).with_context(|| "palette.form.input.foreground")?,
                    border: map(self.form.input.border).with_context(|| "palette.form.input.border")?,
                    invalid_border: map(self.form.input.invalid_border)
                        .with_context(|| "palette.form.input.invalid_border")?,
                    placeholder: map(self.form.input.placeholder).with_context(|| "palette.form.input.placeholder")?,
                },
            },
            focus: FocusPalette { ring: map(self.focus.ring).with_context(|| "palette.focus.ring")? },
            border: BorderPalette {
                default: map(self.border.default).with_context(|| "palette.border.default")?,
                strong: map(self.border.strong).with_context(|| "palette.border.strong")?,
            },
            navigation: NavigationPalette {
                background: map(self.navigation.background).with_context(|| "palette.navigation.background")?,
                foreground: map(self.navigation.foreground).with_context(|| "palette.navigation.foreground")?,
                muted_foreground: map(self.navigation.muted_foreground)
                    .with_context(|| "palette.navigation.muted_foreground")?,
                hover_background: map(self.navigation.hover_background)
                    .with_context(|| "palette.navigation.hover_background")?,
                selected_background: map(self.navigation.selected_background)
                    .with_context(|| "palette.navigation.selected_background")?,
                selected_foreground: map(self.navigation.selected_foreground)
                    .with_context(|| "palette.navigation.selected_foreground")?,
                border: map(self.navigation.border).with_context(|| "palette.navigation.border")?,
            },
            data: DataPalette {
                accent_1: map(self.data.accent_1).with_context(|| "palette.data.accent_1")?,
                accent_2: map(self.data.accent_2).with_context(|| "palette.data.accent_2")?,
                accent_3: map(self.data.accent_3).with_context(|| "palette.data.accent_3")?,
                accent_4: map(self.data.accent_4).with_context(|| "palette.data.accent_4")?,
                accent_5: map(self.data.accent_5).with_context(|| "palette.data.accent_5")?,
            },
        })
    }
}

/// Immutable source palette and precomputed projection for the current sRGB backend.
/// Field access reads the preview; `source()` returns retained ColorValue fields.
/// Rebuild from a modified source to change colors or mapping policy.
#[derive(Clone, Debug)]
pub struct SrgbPalette {
    source: Arc<LumaPalette>,
    preview: Arc<LumaPalette<Hsla>>,
    policy: GamutMapping,
}

impl SrgbPalette {
    pub fn source(&self) -> &LumaPalette {
        &self.source
    }
    pub fn mapping(&self) -> GamutMapping {
        self.policy
    }

    pub(crate) fn light() -> Self {
        static PALETTE: OnceLock<SrgbPalette> = OnceLock::new();
        PALETTE
            .get_or_init(|| Self {
                source: Arc::new(LumaPalette::light()),
                preview: Arc::new(LumaPalette::light_with(|hex| gpui::rgb(hex).into())),
                policy: GamutMapping::CssLocalMinde,
            })
            .clone()
    }
    pub(crate) fn dark() -> Self {
        static PALETTE: OnceLock<SrgbPalette> = OnceLock::new();
        PALETTE
            .get_or_init(|| Self {
                source: Arc::new(LumaPalette::dark()),
                preview: Arc::new(LumaPalette::dark_with(|hex| gpui::rgb(hex).into())),
                policy: GamutMapping::CssLocalMinde,
            })
            .clone()
    }
}

impl Deref for SrgbPalette {
    type Target = LumaPalette<Hsla>;
    fn deref(&self) -> &Self::Target {
        &self.preview
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_palette_persistence_preserves_wide_gamut_and_extended_components() {
        let mut source = LumaPalette::light();
        source.app.background = ColorValue::display_p3(1.0, 0.0, 0.0, 0.37);
        source.state.selected.background = ColorValue::oklch(0.7, 0.4, 725.0, 0.6);
        source.data.accent_5 = ColorValue::linear_srgb(-0.25, 2.0, 0.5, 1.0);
        let stored = toml::to_string(&source).unwrap();
        let restored: LumaPalette = toml::from_str(&stored).unwrap();
        assert_eq!(restored, source);
        let mapped = restored.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(mapped.source(), &source);
        assert_eq!(mapped.app.background.a, 0.37);
        assert!(!mapped.source().app.background.is_in_gamut(crate::color::Gamut::Srgb).unwrap());
    }

    #[test]
    fn rebuilding_source_or_policy_keeps_old_snapshots_independent() {
        let mut source = LumaPalette::light();
        source.app.background = ColorValue::display_p3(1.0, 0.0, 0.0, 0.5);
        let clipped = source.clone().snapshot_srgb(GamutMapping::Clip).unwrap();
        let mapped = source.clone().snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_eq!(clipped.source(), mapped.source());
        assert_ne!(clipped.app.background, mapped.app.background);
        source.app.background = ColorValue::srgb(0.0, 0.0, 0.0, 1.0);
        let changed = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
        assert_ne!(changed.source(), mapped.source());
        assert_eq!(mapped.source().app.background, ColorValue::display_p3(1.0, 0.0, 0.0, 0.5));
    }

    #[test]
    fn snapshot_errors_identify_the_invalid_source_field() {
        let mut source = LumaPalette::dark();
        source.navigation.selected_foreground = ColorValue::srgb(0.0, 0.0, 0.0, f32::NAN);
        let error = source.snapshot_srgb(GamutMapping::CssLocalMinde).unwrap_err();
        assert!(error.to_string().contains("palette.navigation.selected_foreground"));
    }

    #[test]
    fn structural_previews_match_the_bridge_for_every_field() {
        for snapshot in [SrgbPalette::light(), SrgbPalette::dark()] {
            let projected = snapshot.source().clone().snapshot_srgb(GamutMapping::CssLocalMinde).unwrap();
            assert_eq!(*snapshot, *projected);
        }
    }
}
