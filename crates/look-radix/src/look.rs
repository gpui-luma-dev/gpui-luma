//! Runtime radix look: mode + dual scale space (color + gray).

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{Background, Global, Hsla, linear_color_stop, linear_gradient};
use luma::theme::{MetricTokens, ThemeMode};
use luma_look_core::ResolvedColor;

use crate::button::ClassicButtonParams;
use crate::palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
use crate::scale::{ModeScales, ScaleFamily, ScalePair, ScaleStep, color_scale_from_seed};
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
    pub fn stops(self, look: &Look) -> (Hsla, Hsla) {
        (
            look.resolve_step(ScaleFamily::Color, self.color_step).hsla(),
            look.resolve_step(ScaleFamily::Gray, self.gray_step).hsla(),
        )
    }

    /// Top → down linear wash suitable for `.bg(...)`.
    pub fn paint(self, look: &Look) -> Background {
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
    pub fn fill(self, look: &Look) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.fill_step).hsla()
    }

    pub fn card(self, look: &Look) -> Hsla {
        look.resolve_step(ScaleFamily::Gray, self.card_step).hsla()
    }

    pub fn mesh_colors(self, look: &Look) -> SignupMeshColors {
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
pub struct Look {
    state: Arc<LookState>,
}

impl Global for Look {}

/// `.look(&…)` if set, else ambient Global, else [`Look::built_in()`]. Never panics.
pub(crate) fn resolve_look(explicit: Option<&Look>, ambient: Option<&Look>) -> Look {
    if let Some(look) = explicit {
        return look.clone();
    }
    if let Some(look) = ambient {
        return look.clone();
    }
    Look::built_in()
}

struct LookState {
    scales: RwLock<ScalePair>,
    palettes: RwLock<ThemePalettes>,
    metrics: MetricTokens,
    mode: AtomicU8,
    classic_shadow: RwLock<ClassicButtonParams>,
    /// Bumped on mode change so callers can observe cheaply if desired.
    revision: RwLock<u64>,
}

impl Look {
    /// Radix's own default theme: indigo accent with its paired slate gray.
    pub fn built_in() -> Self {
        Self::from_palettes(Accent::default(), Gray::default(), crate::button_layout::metric_tokens(), ThemeMode::Light)
    }

    pub fn from_palettes(accent: Accent, gray: Gray, metrics: MetricTokens, mode: ThemeMode) -> Self {
        let look = Self::new(scale_pair(accent, gray), metrics, mode);
        if let Ok(mut palettes) = look.state.palettes.write() {
            *palettes = ThemePalettes::named(accent, gray);
        }
        look
    }

    /// Scales without palette identity; both slots report as custom.
    pub fn new(scales: ScalePair, metrics: MetricTokens, mode: ThemeMode) -> Self {
        Self {
            state: Arc::new(LookState {
                scales: RwLock::new(scales),
                palettes: RwLock::new(ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom }),
                metrics,
                mode: AtomicU8::new(mode_to_u8(mode)),
                classic_shadow: RwLock::new(ClassicButtonParams::default()),
                revision: RwLock::new(0),
            }),
        }
    }

    /// An independent copy of this look's scales, palettes, metrics, mode, and tuning.
    ///
    /// Cloning a [`Look`] shares one mutable state, so every control repaints together.
    /// Forking is how a screen can edit a palette without touching the rest of the app.
    pub fn fork(&self) -> Self {
        let scales = *self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        let forked = Self::new(scales, self.metrics(), self.mode());
        if let Ok(mut palettes) = forked.state.palettes.write() {
            *palettes = self.palettes();
        }
        if let Ok(mut classic) = forked.state.classic_shadow.write() {
            *classic = self.classic_params();
        }
        forked
    }

    /// Palettes currently in use, for readouts and pickers.
    pub fn palettes(&self) -> ThemePalettes {
        *self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Palette name behind one family, so swatches can report `indigo 9` over `color 9`.
    pub fn palette_label(&self, family: ScaleFamily) -> &'static str {
        self.scales().family(family).palette()
    }

    /// Repaints every control from a different named pair.
    pub fn set_palettes(&self, accent: Accent, gray: Gray) {
        if let Ok(mut scales) = self.state.scales.write() {
            *scales = scale_pair(accent, gray);
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            *palettes = ThemePalettes::named(accent, gray);
        }
        self.bump_revision();
    }

    /// Current [`ButtonVariant::Classic`](crate::ButtonVariant::Classic) shadow geometry.
    pub fn classic_params(&self) -> ClassicButtonParams {
        *self.state.classic_shadow.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Retunes the Classic bubble for every button resolved from this look.
    pub fn set_classic_params(&self, params: ClassicButtonParams) {
        if let Ok(mut current) = self.state.classic_shadow.write() {
            *current = params;
        }
        if let Ok(mut revision) = self.state.revision.write() {
            *revision = revision.wrapping_add(1);
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
    ///
    /// The accent slot stops being a named palette; the gray slot keeps its name.
    pub fn set_accent_seed(&self, seed: Hsla) {
        if let Ok(mut scales) = self.state.scales.write() {
            scales.light.color = color_scale_from_seed(seed, ThemeMode::Light);
            scales.dark.color = color_scale_from_seed(seed, ThemeMode::Dark);
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            palettes.accent = PaletteSlot::Custom;
        }
        self.bump_revision();
    }

    fn bump_revision(&self) {
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
        if role == SemanticRole::PrimaryForeground && self.accent_uses_dark_solid_contrast() {
            return self.dark_solid_contrast();
        }
        role.resolve(self.scales(), self.mode())
    }

    pub fn resolve_step(&self, family: ScaleFamily, step: ScaleStep) -> ResolvedColor {
        self.scales().resolved(family, step)
    }

    /// Whether solid accent faces need dark labels (lime / mint / sky / yellow / amber).
    pub fn accent_uses_dark_solid_contrast(&self) -> bool {
        match self.palettes().accent {
            PaletteSlot::Named(name) => {
                Accent::ALL.iter().any(|accent| accent.as_str() == name && accent.uses_dark_solid_contrast())
            }
            // Custom seeds: treat a light solid face like Radix's bright accents.
            PaletteSlot::Custom => self.resolve_step(ScaleFamily::Color, 9).hsla().l >= 0.68,
        }
    }

    /// Dark label for bright solid faces — the darker of gray 1 / gray 12 in the active mode.
    fn dark_solid_contrast(&self) -> ResolvedColor {
        let gray1 = self.resolve_step(ScaleFamily::Gray, 1);
        let gray12 = self.resolve_step(ScaleFamily::Gray, 12);
        if gray1.hsla().l <= gray12.hsla().l {
            gray1
        } else {
            gray12
        }
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
        let look = Look::built_in();
        let light_bg = look.resolve_role(SemanticRole::Background).hsla();
        look.set_mode(ThemeMode::Dark);
        let dark_bg = look.resolve_role(SemanticRole::Background).hsla();
        assert_ne!(light_bg.l, dark_bg.l);
        assert_eq!(look.revision(), 1);
    }

    #[test]
    fn built_in_reports_indigo_on_slate() {
        let look = Look::built_in();

        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
        assert_eq!(look.palette_label(ScaleFamily::Color), "indigo");
        assert_eq!(look.palette_label(ScaleFamily::Gray), "slate");
    }

    #[test]
    fn a_fork_starts_equal_and_then_drifts_alone() {
        let theme = Look::built_in();
        let draft = theme.fork();
        assert_eq!(draft.palettes(), theme.palettes());
        assert_eq!(draft.resolve_role(SemanticRole::Primary).hsla(), theme.resolve_role(SemanticRole::Primary).hsla());

        draft.set_accent_seed(gpui::hsla(0.05, 0.9, 0.5, 1.0));

        assert_ne!(draft.resolve_role(SemanticRole::Primary).hsla(), theme.resolve_role(SemanticRole::Primary).hsla());
        assert_eq!(draft.palettes().accent, PaletteSlot::Custom);
        assert_eq!(theme.palettes().accent, PaletteSlot::Named("indigo"));
        // The gray slot is untouched by an accent seed.
        assert_eq!(draft.palettes().gray, theme.palettes().gray);
    }

    #[test]
    fn set_palettes_restores_named_scales() {
        let draft = Look::built_in().fork();
        draft.set_accent_seed(gpui::hsla(0.05, 0.9, 0.5, 1.0));

        draft.set_palettes(Accent::Indigo, Gray::Auto);

        assert_eq!(draft.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
        assert_eq!(
            draft.resolve_role(SemanticRole::Primary).hsla(),
            Look::built_in().resolve_role(SemanticRole::Primary).hsla()
        );
    }

    #[test]
    fn page_background_mixes_color_and_gray() {
        let look = Look::built_in();
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
        let look = Look::built_in();
        let stage = look.signup_stage();
        assert_eq!(stage.fill(&look), look.resolve_step(ScaleFamily::Gray, 2).hsla());
        assert_eq!(stage.card(&look), look.resolve_step(ScaleFamily::Gray, 1).hsla());
        let mesh = stage.mesh_colors(&look);
        assert_eq!(mesh.accent_3, look.resolve_step(ScaleFamily::Color, 3).hsla());
        assert_ne!(mesh.accent_9.s, stage.fill(&look).s);
    }
}
