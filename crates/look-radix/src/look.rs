//! Runtime radix look: mode + dual scale space (color + gray).

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{Background, Hsla, linear_color_stop, linear_gradient};
use luma::theme::{MetricTokens, ThemeMode};
use luma_look_core::ResolvedColor;

use crate::scale::{ModeScales, ScaleFamily, ScalePair, ScaleStep, built_in_scales, color_scale_from_seed};
use crate::semantic::SemanticRole;

/// Canonical page wash: color step 3 at the top, settling to gray step 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageBackground {
    /// Chromatic scale step 3 (gradient start / top).
    pub color_step: ScaleStep,
    /// Gray scale step 1 (gradient settle / bottom).
    pub gray_step: ScaleStep,
    /// Fraction down the page where the wash reaches gray (`1/3` matches Radix Colors).
    pub settle_at: f32,
}

impl Default for PageBackground {
    fn default() -> Self {
        Self { color_step: 3, gray_step: 1, settle_at: 1.0 / 3.0 }
    }
}

impl PageBackground {
    pub fn stops(self, look: &RadixLook) -> (Hsla, Hsla) {
        (
            look.resolve_step(ScaleFamily::Color, self.color_step).hsla(),
            look.resolve_step(ScaleFamily::Gray, self.gray_step).hsla(),
        )
    }

    /// Top → down linear wash suitable for `.bg(...)`.
    pub fn paint(self, look: &RadixLook) -> Background {
        let (from, to) = self.stops(look);
        linear_gradient(180.0, linear_color_stop(from, 0.0), linear_color_stop(to, self.settle_at))
    }
}

/// Signup / panel stage behind elevated cards (Radix Colors custom palette).
///
/// HTML equivalent: `background: var(--gray-2)` plus an accent mesh SVG at ~0.6 opacity.
/// Mesh paints use the **color** scale (Radix `--accent-*`) against `--color-background`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SignupStage {
    /// Gray fill behind the card (`--gray-2`).
    pub fill_step: ScaleStep,
    /// Elevated card face (`--color-background` / gray 1).
    pub card_step: ScaleStep,
    /// Overall mesh layer opacity.
    pub mesh_opacity: f32,
}

impl Default for SignupStage {
    fn default() -> Self {
        Self { fill_step: 2, card_step: 1, mesh_opacity: 0.6 }
    }
}

/// Soft mesh tints taken from the Radix signup SVG radial stops (`accent-1/2/3/5/7/9`).
#[derive(Clone, Copy, Debug)]
pub struct SignupMeshColors {
    pub background: Hsla,
    pub accent_1: Hsla,
    pub accent_2: Hsla,
    pub accent_3: Hsla,
    pub accent_5: Hsla,
    pub accent_7: Hsla,
    pub accent_9: Hsla,
}

impl SignupStage {
    pub fn fill(self, look: &RadixLook) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.fill_step).hsla()
    }

    pub fn card(self, look: &RadixLook) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.card_step).hsla()
    }

    pub fn mesh_colors(self, look: &RadixLook) -> SignupMeshColors {
        SignupMeshColors {
            background: look.resolve_role(SemanticRole::Background).hsla(),
            accent_1: look.resolve_step(ScaleFamily::Color, 1).hsla(),
            accent_2: look.resolve_step(ScaleFamily::Color, 2).hsla(),
            accent_3: look.resolve_step(ScaleFamily::Color, 3).hsla(),
            accent_5: look.resolve_step(ScaleFamily::Color, 5).hsla(),
            accent_7: look.resolve_step(ScaleFamily::Color, 7).hsla(),
            accent_9: look.resolve_step(ScaleFamily::Color, 9).hsla(),
        }
    }
}

#[derive(Clone)]
pub struct RadixLook {
    state: Arc<RadixLookState>,
}

struct RadixLookState {
    scales: RwLock<ScalePair>,
    metrics: MetricTokens,
    mode: AtomicU8,
    /// Bumped on mode change so callers can observe cheaply if desired.
    revision: RwLock<u64>,
}

impl RadixLook {
    pub fn built_in() -> Self {
        Self::new(built_in_scales(), MetricTokens::default(), ThemeMode::Light)
    }

    pub fn new(scales: ScalePair, metrics: MetricTokens, mode: ThemeMode) -> Self {
        Self {
            state: Arc::new(RadixLookState {
                scales: RwLock::new(scales),
                metrics,
                mode: AtomicU8::new(mode_to_u8(mode)),
                revision: RwLock::new(0),
            }),
        }
    }

    pub fn mode(&self) -> ThemeMode {
        mode_from_u8(self.state.mode.load(Ordering::Relaxed))
    }

    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
        if let Ok(mut rev) = self.state.revision.write() {
            *rev = rev.wrapping_add(1);
        }
    }

    /// Rebuilds both chromatic mode scales from the supplied color-9 anchor.
    pub fn set_accent_seed(&self, seed: Hsla) {
        if let Ok(mut scales) = self.state.scales.write() {
            scales.light.color = color_scale_from_seed(seed, ThemeMode::Light);
            scales.dark.color = color_scale_from_seed(seed, ThemeMode::Dark);
        }
        if let Ok(mut revision) = self.state.revision.write() {
            *revision = revision.wrapping_add(1);
        }
    }

    pub fn revision(&self) -> u64 {
        *self.state.revision.read().unwrap_or_else(|e| e.into_inner())
    }

    pub fn metrics(&self) -> MetricTokens {
        self.state.metrics
    }

    pub fn scales(&self) -> ModeScales {
        self.state
            .scales
            .read()
            .map(|scales| scales.for_mode(self.mode()))
            .unwrap_or_else(|poisoned| poisoned.into_inner().for_mode(self.mode()))
    }

    pub fn resolve_role(&self, role: SemanticRole) -> ResolvedColor {
        role.resolve(self.scales(), self.mode())
    }

    pub fn resolve_step(&self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.scales().resolved(family, step)
    }

    /// Default page background recipe: **color #3 → gray #1**, settling at ⅓.
    pub fn page_background(&self) -> Background {
        PageBackground::default().paint(self)
    }

    /// Signup stage recipe: gray-2 fill + color-scale mesh tokens + gray-1 card.
    pub fn signup_stage(&self) -> SignupStage {
        SignupStage::default()
    }
}

fn mode_to_u8(mode: ThemeMode) -> u8 {
    match mode {
        ThemeMode::Light => 0,
        ThemeMode::Dark => 1,
    }
}

fn mode_from_u8(value: u8) -> ThemeMode {
    if value == 1 { ThemeMode::Dark } else { ThemeMode::Light }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_switch_changes_resolved_background() {
        let look = RadixLook::built_in();
        let light_bg = look.resolve_role(SemanticRole::Background).hsla();
        look.set_mode(ThemeMode::Dark);
        let dark_bg = look.resolve_role(SemanticRole::Background).hsla();
        assert_ne!(light_bg.l, dark_bg.l);
        assert_eq!(look.revision(), 1);
    }

    #[test]
    fn page_background_mixes_color_and_gray() {
        let look = RadixLook::built_in();
        let recipe = PageBackground::default();
        let (from, to) = recipe.stops(&look);
        let color_3 = look.resolve_step(ScaleFamily::Color, 3).hsla();
        let gray_1 = look.resolve_step(ScaleFamily::Gray, 1).hsla();
        assert_eq!(from, color_3);
        assert_eq!(to, gray_1);
        assert_ne!(from.s, to.s);
    }

    #[test]
    fn signup_stage_uses_gray_fill_and_color_mesh() {
        let look = RadixLook::built_in();
        let stage = look.signup_stage();
        assert_eq!(stage.fill(&look), look.resolve_step(ScaleFamily::Gray, 2).hsla());
        assert_eq!(stage.card(&look), look.resolve_step(ScaleFamily::Gray, 1).hsla());
        let mesh = stage.mesh_colors(&look);
        assert_eq!(mesh.accent_3, look.resolve_step(ScaleFamily::Color, 3).hsla());
        assert_ne!(mesh.accent_9.s, stage.fill(&look).s);
    }
}
