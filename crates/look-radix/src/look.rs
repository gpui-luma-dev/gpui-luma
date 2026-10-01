//! Runtime radix look: mode + dual scale space (color + gray).

use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
    RwLock,
};

use gpui::{Background, Global, Hsla, linear_color_stop, linear_gradient};
use luma::theme::{MetricTokens, ThemeMode};
use luma::theme::provenance::ResolvedColor;

use crate::button::ClassicButtonParams;
use crate::palette::{PaletteSlot, Accent, Gray, ThemePalettes, scale_pair};
use crate::scale::{ModeScales, ScaleFamily, ScalePair, ScaleStep};
use crate::semantic::SemanticRole;
use crate::{CustomColors, generate_colors};

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

/// Shared palette and control styling state; cloning shares state, `fork` copies it.
///
/// # Redrawing after changes
/// Setters update state and revision only: they do **not** send GPUI notifications.
/// Notify the owning view and any independently rendered control entities that use
/// this look. Controls resolve new colors on their next render. Rebuild passive
/// elements (Avatar, Badge, Card) in the owning view's render method.
///
/// ```no_run
/// use gpui::{Context, Entity};
/// use luma::controls::button::Button;
/// use luma::theme::ThemeMode;
/// use luma_look_radix::Look;
///
/// fn change_mode<M: 'static>(look: &Look, button: &Entity<Button>, cx: &mut Context<M>) {
///     look.set_mode(ThemeMode::Dark);
///     button.update(cx, |_, cx| cx.notify()); // repeat for affected child entities
///     cx.notify(); // rebuild the owner's passive elements and layout
/// }
/// ```
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
    palettes: RwLock<[ThemePalettes; 2]>,
    custom_contrast: RwLock<[Option<Hsla>; 2]>,
    metrics: MetricTokens,
    mode: AtomicU8,
    classic_shadow: RwLock<ClassicButtonParams>,
    tabs_style: RwLock<crate::tabs::TabsStyle>,
    /// Bumped by look setters; polling does not schedule GPUI redraws.
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
            *palettes = [ThemePalettes::named(accent, gray); 2];
        }
        look
    }

    /// Scales without palette identity; both slots report as custom.
    pub fn new(scales: ScalePair, metrics: MetricTokens, mode: ThemeMode) -> Self {
        Self {
            state: Arc::new(LookState {
                scales: RwLock::new(scales),
                palettes: RwLock::new([ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom }; 2]),
                custom_contrast: RwLock::new([None; 2]),
                metrics,
                mode: AtomicU8::new(mode_to_u8(mode)),
                classic_shadow: RwLock::new(ClassicButtonParams::default()),
                tabs_style: RwLock::new(crate::tabs::TabsStyle::default()),
                revision: RwLock::new(0),
            }),
        }
    }

    /// An independent copy of this look's scales, palettes, metrics, mode, and tuning.
    ///
    /// Cloning a [`Look`] shares mutable state; callers still notify affected views.
    /// Forking is how a screen can edit a palette without touching the rest of the app.
    pub fn fork(&self) -> Self {
        let scales = *self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        let forked = Self::new(scales, self.metrics(), self.mode());
        if let Ok(mut palettes) = forked.state.palettes.write() {
            *palettes = *self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if let Ok(mut classic) = forked.state.classic_shadow.write() {
            *classic = self.classic_params();
        }
        if let Ok(mut tabs) = forked.state.tabs_style.write() {
            *tabs = self.tabs_style();
        }
        if let Ok(mut contrast) = forked.state.custom_contrast.write() {
            *contrast = *self.state.custom_contrast.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        forked
    }

    /// Shared tab geometry, baseline, and indicator settings for this look.
    pub fn tabs_style(&self) -> crate::tabs::TabsStyle {
        *self.state.tabs_style.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Update tab defaults for subsequent renders. Notify affected views; see [`Look`].
    pub fn set_tabs_style(&self, style: crate::tabs::TabsStyle) {
        if let Ok(mut current) = self.state.tabs_style.write() {
            *current = style;
        }
        self.bump_revision();
    }

    /// Palettes currently in use, for readouts and pickers.
    pub fn palettes(&self) -> ThemePalettes {
        self.state.palettes.read().unwrap_or_else(|poisoned| poisoned.into_inner())[mode_to_u8(self.mode()) as usize]
    }

    /// Palette name behind one family, so swatches can report `indigo 9` over `color 9`.
    pub fn palette_label(&self, family: ScaleFamily) -> &'static str {
        self.scales().family(family).palette()
    }

    /// Select a named palette pair for subsequent renders.
    /// Notify affected views after changing it; see [`Look`].
    pub fn set_palettes(&self, accent: Accent, gray: Gray) {
        if let Ok(mut scales) = self.state.scales.write() {
            *scales = scale_pair(accent, gray);
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            *palettes = [ThemePalettes::named(accent, gray); 2];
        }
        if let Ok(mut contrast) = self.state.custom_contrast.write() {
            *contrast = [None; 2];
        }
        self.bump_revision();
    }

    /// Current [`ButtonVariant::Classic`](crate::ButtonVariant::Classic) shadow geometry.
    pub fn classic_params(&self) -> ClassicButtonParams {
        *self.state.classic_shadow.read().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Retunes the Classic bubble for every button resolved from this look.
    /// Notify affected views after changing it; see [`Look`].
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

    /// Change the active mode. Notify affected views afterward; see [`Look`].
    pub fn set_mode(&self, mode: ThemeMode) {
        self.state.mode.store(mode_to_u8(mode), Ordering::Relaxed);
        if let Ok(mut rev) = self.state.revision.write() {
            *rev = rev.wrapping_add(1);
        }
    }

    /// Rebuilds both accent scales using Radix's custom-color generator.
    /// Uses each mode's current gray step 8 and background; leaves gray unchanged.
    /// Notify affected views after changing it; see [`Look`].
    pub fn set_accent_seed(&self, seed: Hsla) {
        let pair = *self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner());
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let scales = pair.for_mode(mode);
            let generated = generate_colors(
                CustomColors { accent: seed, gray: scales.gray.step(8), background: scales.gray.step(1) },
                mode,
            );
            if let Ok(mut scales) = self.state.scales.write() {
                match mode {
                    ThemeMode::Light => scales.light.color = generated.accent,
                    ThemeMode::Dark => scales.dark.color = generated.accent,
                }
            }
            if let Ok(mut contrast) = self.state.custom_contrast.write() {
                contrast[mode_to_u8(mode) as usize] = Some(generated.accent_contrast);
            }
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            for palette in palettes.iter_mut() {
                palette.accent = PaletteSlot::Custom;
            }
        }
        self.bump_revision();
    }

    /// Regenerates the active mode's accent and gray scales from all three seed colors.
    /// The caller owns editor state and must notify affected views; see [`Look`].
    /// The background affects scale generation; page composition stays with the caller.
    pub fn set_custom_colors(&self, inputs: CustomColors) {
        let mode = self.mode();
        let generated = generate_colors(inputs, mode);
        if let Ok(mut pair) = self.state.scales.write() {
            let scales = match mode {
                ThemeMode::Light => &mut pair.light,
                ThemeMode::Dark => &mut pair.dark,
            };
            scales.color = generated.accent;
            scales.gray = generated.gray;
        }
        if let Ok(mut contrast) = self.state.custom_contrast.write() {
            contrast[mode_to_u8(mode) as usize] = Some(generated.accent_contrast);
        }
        if let Ok(mut palettes) = self.state.palettes.write() {
            palettes[mode_to_u8(mode) as usize] =
                ThemePalettes { accent: PaletteSlot::Custom, gray: PaletteSlot::Custom };
        }
        self.bump_revision();
    }

    fn custom_contrast(&self) -> Option<Hsla> {
        self.state.custom_contrast.read().unwrap_or_else(|poisoned| poisoned.into_inner())
            [mode_to_u8(self.mode()) as usize]
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
        if role == SemanticRole::PrimaryForeground {
            if let Some(contrast) = self.custom_contrast() {
                return ResolvedColor::authored(contrast, "custom-accent-contrast");
            }
            if self.accent_uses_dark_solid_contrast() {
                return self.dark_solid_contrast();
            }
        }
        role.resolve(self.scales(), self.mode())
    }

    /// Control thumbs retain their light palette in both theme modes.
    pub(crate) fn light_gray_step(&self, step: ScaleStep) -> Hsla {
        self.state.scales.read().unwrap_or_else(|poisoned| poisoned.into_inner()).light.gray.step(step)
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
            PaletteSlot::Custom => self
                .custom_contrast()
                .map(|color| color.l < 0.5)
                .unwrap_or_else(|| self.resolve_step(ScaleFamily::Color, 9).hsla().l >= 0.68),
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
    fn custom_palette_is_mode_local_and_fork_copies_contrast() {
        let look = Look::from_palettes(Accent::Yellow, Gray::Auto, Default::default(), ThemeMode::Light);
        let original = look.resolve_step(ScaleFamily::Color, 9).hsla();
        let light_contrast = look.resolve_role(SemanticRole::PrimaryForeground).hsla();
        look.set_mode(ThemeMode::Dark);
        look.set_custom_colors(CustomColors {
            accent: gpui::rgb(0xffff00).into(),
            gray: gpui::rgb(0x888888).into(),
            background: gpui::black(),
        });
        let dark_contrast = look.resolve_role(SemanticRole::PrimaryForeground).hsla();
        assert!(dark_contrast.l < 0.5);
        assert_eq!(look.fork().resolve_role(SemanticRole::PrimaryForeground).hsla(), dark_contrast);
        look.set_mode(ThemeMode::Light);
        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Yellow, Gray::Auto));
        assert_eq!(look.resolve_step(ScaleFamily::Color, 9).hsla(), original);
        assert_eq!(look.resolve_role(SemanticRole::PrimaryForeground).hsla(), light_contrast);
        look.set_palettes(Accent::Indigo, Gray::Auto);
        look.set_mode(ThemeMode::Dark);
        assert!(look.custom_contrast().is_none());
        assert_eq!(look.palettes(), ThemePalettes::named(Accent::Indigo, Gray::Auto));
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
}
