//! Radix-owned button variants bound to SDK [`ButtonFamilyTheme`].

use std::sync::Arc;

use gpui::{Background, BoxShadow, Hsla, SharedString, black, linear_color_stop, linear_gradient, point, prelude::*, px};
use gpui_luma::controls::button::{ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate};
use gpui_luma::controls::button_family::{
    ButtonFamilyLook, ButtonFamilyPalette, ButtonFamilyRole, ButtonFamilyTheme, compose_button_family_look,
};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState, LumaTextStyle, MetricTokens, StandardBoxScale};
use gpui::FontWeight;

use crate::button_layout::{ButtonSize, Radius, apply_button_box, button_box_with_stylesheet};
use crate::look::Look;
use crate::scale::ScaleFamily;
use crate::semantic::SemanticRole;
use crate::tone::Tone;

/// Look-owned button treatments. Not Shadcn variant names and not SDK enums.
///
/// [`Classic`](Self::Classic) through [`Ghost`](Self::Ghost) are the Radix Themes
/// variant scheme. The remaining variants are app chrome kept for one-off use and
/// are deliberately outside that scheme.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ButtonVariant {
    /// Solid accent fill with a darker edge and a raised drop shadow.
    Classic,
    #[default]
    Solid,
    Soft,
    /// Accent 2 panel with an accent 7 edge and accent 11 text.
    Surface,
    Outline,
    Ghost,
    /// Ghost chrome (transparent at rest, soft hover fill) without a focus ring.
    /// For icon-only chrome that must not draw a border in any state.
    GhostQuiet,
    /// Screen-nav pill. Selected reads as a filled tab rather than a button.
    /// Interaction is binary (off | on) — no focus ring, hover, or pressed paint.
    Page,
}

/// Paint axes that sit beside [`ButtonVariant`]: accent vs gray, and Radix `highContrast`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Paint {
    pub tone: Tone,
    pub high_contrast: bool,
}

impl Paint {
    pub const fn accent() -> Self {
        Self { tone: Tone::Accent, high_contrast: false }
    }

    pub const fn gray() -> Self {
        Self { tone: Tone::Gray, high_contrast: false }
    }

    pub const fn high_contrast(mut self) -> Self {
        self.high_contrast = true;
        self
    }
}

struct ButtonFamilyThemeAdapter {
    look: Look,
    variant: ButtonVariant,
    paint: Paint,
    size: Option<ButtonSize>,
}

impl ButtonFamilyTheme for ButtonFamilyThemeAdapter {
    fn resolve(&self, role: ButtonFamilyRole, _size: ControlSize, state: InteractionState) -> ButtonFamilyPalette {
        resolve_button_palette(&self.look, self.variant, self.paint, role, state)
    }

    fn metrics(&self) -> MetricTokens {
        self.look.metrics()
    }

    fn resolve_look(
        &self,
        role: ButtonFamilyRole,
        size: ControlSize,
        state: InteractionState,
        scale: &StandardBoxScale,
        pill_radius: f32,
    ) -> Option<ButtonFamilyLook> {
        let mut palette = self.resolve(role, size, state);
        let geometry = self.look.common_stylesheet().button.resolve_geometry(
            self.size.map(ButtonSize::as_str).unwrap_or_else(|| crate::look::sdk_size_key(size)),
            gpui_luma::theme::stylesheet::ButtonGeometry {
                height: scale.height,
                padding_x: scale.padding_x,
                padding_y: scale.padding_y,
                gap: scale.gap,
                icon_size: scale.icon_size,
                font_size: palette.typography.size,
                line_height: palette.typography.line_height,
            },
        );
        palette.typography.size = geometry.font_size.value_px;
        palette.typography.line_height = geometry.line_height.value_px;
        let mut look = compose_button_family_look(&palette, role, scale, pill_radius);
        look.height = geometry.height.value_px;
        look.gap = geometry.gap.value_px;
        if role != ButtonFamilyRole::Icon {
            look.padding_x = geometry.padding_x.value_px;
            look.padding_y = geometry.padding_y.value_px;
        } else {
            look.icon_size = geometry.icon_size.value_px;
        }
        if matches!(self.variant, ButtonVariant::Page) {
            look.radius = pill_radius;
        }
        if matches!(self.variant, ButtonVariant::Classic) {
            look.shadow = Some(classic_shadow(&self.look, palette.background, state));
            // Classic keeps an editable radius knob; default matches Radix size-2 / medium (4px).
            look.radius = self.look.classic_params().radius;
        }
        Some(look)
    }
}

/// Accent tone, normal contrast — the default Radix button paint.
pub fn button_family_theme(look: &Look, variant: ButtonVariant) -> Arc<dyn ButtonFamilyTheme> {
    button_family_theme_with(look, variant, Paint::accent())
}

/// Full look resolver for a Radix size × radius cell (All Sizes matrix / previews).
pub fn button_look_for<D: 'static>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
    radius: Radius,
) -> gpui_luma::controls::button::ButtonLookSource<D> {
    let look = look.clone();
    Arc::new(move |model| {
        let palette = resolve_button_palette(&look, variant, paint, model.role, model.state);
        let mut family =
            apply_button_box(&palette, model.role, button_box_with_stylesheet(&look.common_stylesheet(), size, radius));
        if matches!(variant, ButtonVariant::Classic) {
            family.shadow = Some(classic_shadow(&look, family.background, model.state));
        }
        family
    })
}

pub fn button_family_theme_with(look: &Look, variant: ButtonVariant, paint: Paint) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ButtonFamilyThemeAdapter { look: look.clone(), variant, paint, size: None })
}

pub(crate) fn button_family_theme_for_size(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    size: ButtonSize,
) -> Arc<dyn ButtonFamilyTheme> {
    Arc::new(ButtonFamilyThemeAdapter { look: look.clone(), variant, paint, size: Some(size) })
}

fn resolve_button_palette(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
    role: ButtonFamilyRole,
    state: InteractionState,
) -> ButtonFamilyPalette {
    let layer = state.layer();
    let selected = matches!(role, ButtonFamilyRole::Toggle { selected: true });
    // Quiet ghosts share every Ghost treatment except the focus ring.
    let quiet = matches!(variant, ButtonVariant::GhostQuiet);
    let page = matches!(variant, ButtonVariant::Page);
    let variant = if quiet { ButtonVariant::Ghost } else { variant };
    let high_contrast = paint.high_contrast;
    let tone = paint.tone;
    let step = |n| tone.step(look, n);

    // Page / selected chrome stay on semantic roles (app chrome, not Radix color prop).
    // Border stays None in every state: the SDK paints a focus ring from `look.border`,
    // so any opaque edge on the selected pill becomes a ring when focused.
    let (mut background, mut foreground, mut border) = if page {
        if selected {
            match look.mode() {
                gpui_luma::theme::ThemeMode::Light => (
                    look.resolve_role(SemanticRole::Foreground).hsla(),
                    look.resolve_role(SemanticRole::Background).hsla(),
                    None,
                ),
                gpui_luma::theme::ThemeMode::Dark => (
                    look.resolve_role(SemanticRole::PrimaryForeground).hsla(),
                    look.resolve_role(SemanticRole::Background).hsla(),
                    None,
                ),
            }
        } else {
            (
                look.resolve_role(SemanticRole::Background).hsla(),
                look.resolve_role(SemanticRole::MutedForeground).hsla(),
                None,
            )
        }
    } else if selected {
        (step(9), tone.contrast(look), Some(step(9)))
    } else {
        match variant {
            ButtonVariant::Classic | ButtonVariant::Solid => {
                if high_contrast {
                    // Radix: accent-12 on gray-1.
                    (step(12), solid_high_contrast_label(look), Some(step(12)))
                } else {
                    (step(9), tone.contrast(look), Some(step(9)))
                }
            }
            ButtonVariant::Soft => {
                let fg = if high_contrast { step(12) } else { step(11) };
                (step(3), fg, Some(step(3)))
            }
            ButtonVariant::Surface => {
                let fg = if high_contrast { step(12) } else { step(11) };
                (step(2), fg, Some(step(7)))
            }
            ButtonVariant::Outline => {
                let fg = if high_contrast {
                    step(12)
                } else {
                    look.resolve_role(SemanticRole::Foreground).hsla()
                };
                let edge = if high_contrast {
                    step(11)
                } else {
                    look.resolve_role(SemanticRole::Border).hsla()
                };
                (transparent(), fg, Some(edge))
            }
            ButtonVariant::Ghost | ButtonVariant::GhostQuiet => {
                let fg = if high_contrast {
                    step(12)
                } else {
                    look.resolve_role(SemanticRole::Foreground).hsla()
                };
                (transparent(), fg, None)
            }
            ButtonVariant::Page => unreachable!(),
        }
    };

    if matches!(variant, ButtonVariant::Ghost | ButtonVariant::Outline | ButtonVariant::Page) && !selected {
        background = transparent();
        // Page and Ghost are borderless chrome; Outline keeps its edge.
        if matches!(variant, ButtonVariant::Ghost | ButtonVariant::Page) {
            border = None;
        }
    }

    match layer {
        InteractionLayer::Disabled if !page => {
            background = look.resolve_role(SemanticRole::Surface).hsla();
            foreground = look.resolve_role(SemanticRole::MutedForeground).hsla();
            border = Some(look.resolve_role(SemanticRole::Border).hsla());
        }
        InteractionLayer::Disabled => {
            foreground = look.resolve_role(SemanticRole::MutedForeground).hsla();
        }
        // Page toggles are binary off|on — ignore hover / press / focus paint.
        InteractionLayer::Pressed | InteractionLayer::Hovered if page => {}
        InteractionLayer::Pressed => {
            background = shift_for_press(look, variant, tone, high_contrast, selected, background);
        }
        InteractionLayer::Hovered => {
            background = shift_for_hover(look, variant, tone, high_contrast, selected, background);
            if matches!(variant, ButtonVariant::Surface) && !selected {
                border = Some(step(8));
            }
        }
        InteractionLayer::Default => {}
    }

    // Classic's edge tracks whatever the face settled on, so hover and press keep a
    // consistent outline instead of snapping back to the resting accent.
    if matches!(variant, ButtonVariant::Classic) && !matches!(layer, InteractionLayer::Disabled) {
        border = Some(shift_lightness(background, look.classic_params().border_delta));
    }

    if state.focused && !state.disabled && !quiet && !page {
        border = Some(look.resolve_role(SemanticRole::Focus).hsla());
    }

    ButtonFamilyPalette {
        background,
        foreground,
        muted_foreground: foreground,
        border,
        typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::MEDIUM },
        font_family: SharedString::from("System UI"),
    }
}

/// Solid high-contrast labels always use gray-1, matching Radix `color: var(--gray-1)`.
fn solid_high_contrast_label(look: &Look) -> Hsla {
    look.resolve_step(ScaleFamily::Gray, 1).hsla()
}

/// Recipes that tint toward accent 3/4 on hover and press instead of the solid steps.
fn is_tinted(variant: ButtonVariant) -> bool {
    matches!(
        variant,
        ButtonVariant::Soft | ButtonVariant::Surface | ButtonVariant::Ghost | ButtonVariant::Outline
    )
}

fn shift_for_hover(
    look: &Look,
    variant: ButtonVariant,
    tone: Tone,
    high_contrast: bool,
    selected: bool,
    base: Hsla,
) -> Hsla {
    if selected || matches!(variant, ButtonVariant::Solid | ButtonVariant::Classic) {
        // HC solids stay on step 12; Radix uses a CSS filter we approximate with a lighten.
        return if high_contrast {
            shift_lightness(tone.step(look, 12), 0.04)
        } else {
            tone.step(look, 10)
        };
    }
    if is_tinted(variant) {
        return tone.step(look, 3);
    }
    let _ = look;
    base
}

fn shift_for_press(
    look: &Look,
    variant: ButtonVariant,
    tone: Tone,
    high_contrast: bool,
    selected: bool,
    base: Hsla,
) -> Hsla {
    if selected || matches!(variant, ButtonVariant::Solid | ButtonVariant::Classic) {
        return if high_contrast {
            shift_lightness(tone.step(look, 12), -0.03)
        } else {
            tone.step(look, 10)
        };
    }
    if is_tinted(variant) {
        return tone.step(look, 4);
    }
    let _ = look;
    base
}

fn inset_shadow(offset_y: f32, blur: f32, spread: f32, color: Hsla) -> BoxShadow {
    BoxShadow {
        offset: point(px(0.0), px(offset_y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        color,
        inset: true,
    }
}

/// Tunable geometry for the [`Classic`](ButtonVariant::Classic) button.
///
/// Held on [`Look`](crate::Look) so a tuning UI can drive every Classic
/// button live. Defaults are sampled from a reference render: the face is a vertical
/// wash that grows *lighter* toward the bottom, a 1px specular rim sits just inside
/// the top border, and a single border darker than the fill wraps the whole edge.
///
/// `*_delta` and `rim_lighten` are lightness offsets applied to the accent fill,
/// `*_alpha` are `0..1`, and the rest are `px`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClassicButtonParams {
    /// Corner radius for every Classic button, replacing the size-scale radius. The
    /// rim shadows trace this curve, so the two have to be tuned together.
    pub radius: f32,
    /// Lightness offset at the top of the face wash.
    pub fill_top_delta: f32,
    /// Lightness offset at the bottom of the face wash. Positive: bottom is lighter.
    pub fill_bottom_delta: f32,
    /// Lightness offset for the border that wraps the edge. Negative: darker than fill.
    pub border_delta: f32,
    /// Lightness dropped below the fill for the shaded lip inside the bottom border.
    /// Reads as a second, darker edge band under the face.
    pub bevel_darken: f32,
    pub bevel_offset: f32,
    pub bevel_alpha: f32,
    /// Lightness offset for the specular highlight inside the top border.
    pub rim_lighten: f32,
    pub rim_offset: f32,
    pub rim_alpha: f32,
    pub fade_offset: f32,
    pub fade_blur: f32,
    /// Negative spread keeps the fade a top band instead of a full-perimeter halo.
    pub fade_spread: f32,
    pub fade_alpha: f32,
    pub drop_offset: f32,
    pub drop_blur: f32,
    pub drop_alpha: f32,
}

impl Default for ClassicButtonParams {
    fn default() -> Self {
        Self {
            // Radix size-2 + medium radius (`--radius-2` at factor 1).
            radius: 4.0,
            fill_top_delta: 0.0,
            fill_bottom_delta: 0.067,
            border_delta: -0.022,
            bevel_darken: 0.045,
            bevel_offset: 1.0,
            bevel_alpha: 1.0,
            rim_lighten: 0.30,
            rim_offset: 1.0,
            rim_alpha: 0.95,
            fade_offset: 4.0,
            fade_blur: 4.0,
            fade_spread: -2.0,
            fade_alpha: 0.35,
            drop_offset: 1.0,
            drop_blur: 2.0,
            drop_alpha: 0.08,
        }
    }
}

fn shift_lightness(color: Hsla, delta: f32) -> Hsla {
    Hsla { l: (color.l + delta).clamp(0.02, 0.98), ..color }
}

/// Vertical wash stops for the Classic face: the light source is overhead, so the top
/// sits at the accent and the bottom lifts. Pressing flips them so the face reads
/// recessed.
fn classic_wash(params: ClassicButtonParams, base: Hsla, pressed: bool) -> (Hsla, Hsla) {
    let top = shift_lightness(base, params.fill_top_delta);
    let bottom = shift_lightness(base, params.fill_bottom_delta);
    if pressed { (bottom, top) } else { (top, bottom) }
}

fn classic_fill(look: &Look, paint: Paint, role: ButtonFamilyRole, state: InteractionState) -> Background {
    let base = resolve_button_palette(look, ButtonVariant::Classic, paint, role, state).background;
    if state.disabled {
        return base.into();
    }

    let (top, bottom) = classic_wash(look.classic_params(), base, state.pressed);
    linear_gradient(180.0, linear_color_stop(top, 0.0), linear_color_stop(bottom, 1.0))
}

/// Bevel for the Classic face: a crisp specular rim just inside the top border with a
/// short fade below it, and a shaded lip inside the bottom border. Pressing drops the
/// lip and inverts the rim so the light reads as coming from below.
fn classic_shadow(look: &Look, fill: Hsla, state: InteractionState) -> Vec<BoxShadow> {
    let params = look.classic_params();
    let tint = |delta: f32, alpha: f32| Hsla { a: alpha, ..shift_lightness(fill, delta) };

    if state.pressed {
        let shade = -params.rim_lighten * 0.5;
        return vec![
            inset_shadow(params.rim_offset, 0.0, 0.0, tint(shade, params.rim_alpha * 0.6)),
            inset_shadow(params.fade_offset, params.fade_blur, params.fade_spread, tint(shade, params.fade_alpha)),
        ];
    }

    let mut shadows = vec![
        inset_shadow(params.rim_offset, 0.0, 0.0, tint(params.rim_lighten, params.rim_alpha)),
        inset_shadow(
            params.fade_offset,
            params.fade_blur,
            params.fade_spread,
            tint(params.rim_lighten, params.fade_alpha),
        ),
        inset_shadow(-params.bevel_offset, 0.0, 0.0, tint(-params.bevel_darken, params.bevel_alpha)),
    ];
    if params.drop_alpha > 0.0 {
        shadows.push(BoxShadow {
            offset: point(px(0.0), px(params.drop_offset)),
            blur_radius: px(params.drop_blur),
            spread_radius: px(0.0),
            color: Hsla { a: params.drop_alpha, ..black() },
            inset: false,
        });
    }
    shadows
}

/// Template for a Radix variant × paint cell. Classic keeps its gradient wash.
pub fn button_template<D: Clone + 'static>(
    look: &Look,
    variant: ButtonVariant,
    paint: Paint,
) -> Arc<dyn ButtonTemplate<D>> {
    if matches!(variant, ButtonVariant::Classic) {
        classic_button_template_with(look, paint)
    } else {
        Arc::new(DefaultButtonTemplate::new(button_family_theme_with(look, variant, paint)))
    }
}

/// Classic needs a gradient face, which [`ButtonFamilyLook`] cannot carry, so the look
/// ships its own template. Callers that build templates by hand (the style guide matrix)
/// must use this instead of a bare [`DefaultButtonTemplate`] or they lose the wash.
pub fn classic_button_template<D: Clone + 'static>(look: &Look) -> Arc<dyn ButtonTemplate<D>> {
    classic_button_template_with(look, Paint::accent())
}

/// Classic template with an explicit paint axis (tone / high contrast).
pub fn classic_button_template_with<D: Clone + 'static>(look: &Look, paint: Paint) -> Arc<dyn ButtonTemplate<D>> {
    let theme = button_family_theme_with(look, ButtonVariant::Classic, paint);
    let look = look.clone();
    Arc::new(DefaultButtonTemplate::new(theme).with_modifier(move |root, model: &ButtonRenderModel<D>| {
        root.bg(classic_fill(&look, paint, model.role, model.state))
    }))
}

fn transparent() -> Hsla {
    gpui::hsla(0.0, 0.0, 0.0, 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_luma::theme::InteractionState;

    #[test]
    fn solid_default_uses_primary_steps() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Solid);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        let expected_bg = look.resolve_role(SemanticRole::Primary).hsla();
        assert_eq!(palette.background.h, expected_bg.h);
        assert_eq!(palette.background.l, expected_bg.l);
    }

    #[test]
    fn ghost_default_is_transparent() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Ghost);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
        assert_eq!(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn surface_uses_accent_panel_and_edge() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Surface);
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, look.resolve_step(ScaleFamily::Color, 2).hsla());
        assert_eq!(palette.border, Some(look.resolve_step(ScaleFamily::Color, 7).hsla()));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Color, 11).hsla());
    }

    #[test]
    fn solid_high_contrast_uses_step_twelve_on_gray_one() {
        let look = Look::built_in();
        let theme = button_family_theme_with(&look, ButtonVariant::Solid, Paint::gray().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, Tone::Gray.step(&look, 12));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn classic_high_contrast_uses_step_twelve() {
        let look = Look::built_in();
        let theme = button_family_theme_with(&look, ButtonVariant::Classic, Paint::accent().high_contrast());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background, Tone::Accent.step(&look, 12));
        assert_eq!(palette.foreground, look.resolve_step(ScaleFamily::Gray, 1).hsla());
    }

    #[test]
    fn soft_high_contrast_darkens_the_label_only() {
        let look = Look::built_in();
        let normal = button_family_theme_with(&look, ButtonVariant::Soft, Paint::gray());
        let hc = button_family_theme_with(&look, ButtonVariant::Soft, Paint::gray().high_contrast());
        let state = InteractionState::default();
        let normal = normal.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);
        let hc = hc.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);

        assert_eq!(hc.background, normal.background);
        assert_eq!(hc.foreground, Tone::Gray.step(&look, 12));
        assert_ne!(hc.foreground, normal.foreground);
    }

    #[test]
    fn classic_high_contrast_shifts_fill_to_step_twelve() {
        let look = Look::built_in();
        let normal = button_family_theme_with(&look, ButtonVariant::Classic, Paint::gray());
        let hc = button_family_theme_with(&look, ButtonVariant::Classic, Paint::gray().high_contrast());
        let state = InteractionState::default();

        assert_ne!(
            normal.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background,
            hc.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background
        );
    }

    #[test]
    fn classic_lime_uses_dark_contrast_label() {
        let look = Look::from_palettes(
            crate::palette::Accent::Lime,
            crate::palette::Gray::Auto,
            Default::default(),
            gpui_luma::theme::ThemeMode::Light,
        );
        let theme = button_family_theme_with(&look, ButtonVariant::Classic, Paint::accent());
        let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());

        assert!(palette.foreground.l < palette.background.l);
        assert_eq!(palette.foreground, look.resolve_role(SemanticRole::PrimaryForeground).hsla());
    }

    #[test]
    fn classic_shares_the_solid_fill_behind_a_darker_border() {
        let look = Look::built_in();
        let classic = button_family_theme(&look, ButtonVariant::Classic);
        let solid = button_family_theme(&look, ButtonVariant::Solid);
        let state = InteractionState::default();
        let palette = classic.resolve(ButtonFamilyRole::Text, ControlSize::Md, state);

        assert_eq!(palette.background, solid.resolve(ButtonFamilyRole::Text, ControlSize::Md, state).background);
        let border = palette.border.expect("classic keeps a border that delineates the edge");
        assert!(border.l < palette.background.l, "border is darker than the fill");
        assert_eq!(border.h, palette.background.h, "border keeps the accent hue");
    }

    fn classic_shadows(look: &Look, state: InteractionState) -> Vec<BoxShadow> {
        let theme = button_family_theme(&look, ButtonVariant::Classic);
        let scale = StandardBoxScale::compute(ControlSize::Md, &look.metrics(), 2.0);
        theme
            .resolve_look(ButtonFamilyRole::Text, ControlSize::Md, state, &scale, 999.0)
            .expect("classic look")
            .shadow
            .expect("classic shadows")
    }

    #[test]
    fn classic_bevels_a_lit_top_and_a_shaded_bottom() {
        let look = Look::built_in();
        let fill = look.resolve_role(SemanticRole::Primary).hsla();
        let shadows = classic_shadows(&look, InteractionState::default());
        let edge = |downward: bool| {
            shadows
                .iter()
                .filter(|shadow| shadow.inset && (shadow.offset.y > px(0.0)) == downward)
                .collect::<Vec<_>>()
        };
        let (top, bottom) = (edge(true), edge(false));

        assert!(top.iter().all(|shadow| shadow.color.l > fill.l), "the top rim is lighter than the fill");
        assert!(!bottom.is_empty(), "the bottom keeps a shaded lip");
        assert!(bottom.iter().all(|shadow| shadow.color.l < fill.l), "the bottom lip is darker than the fill");
        assert!(shadows.iter().all(|shadow| !shadow.inset || shadow.color.h == fill.h), "edges keep the accent hue");
        assert!(top.iter().any(|shadow| shadow.blur_radius == px(0.0)), "has a crisp specular line");
        // Negative spread keeps the fade a top band instead of a full-perimeter halo.
        assert!(
            top.iter().any(|shadow| shadow.blur_radius > px(0.0) && shadow.spread_radius < px(0.0)),
            "has an inward-pulled fade"
        );
    }

    #[test]
    fn classic_face_lightens_toward_the_bottom() {
        let look = Look::built_in();
        let fill = look.resolve_role(SemanticRole::Primary).hsla();
        let params = look.classic_params();
        let (top, bottom) = classic_wash(params, fill, false);
        let (pressed_top, pressed_bottom) = classic_wash(params, fill, true);

        assert!(bottom.l > top.l, "the wash lifts toward the bottom, away from the overhead light");
        assert!(pressed_bottom.l < pressed_top.l, "pressing flips the wash so the face reads recessed");
    }

    #[test]
    fn classic_shadow_follows_the_look_params() {
        let look = Look::built_in();
        let blur = || classic_shadows(&look, InteractionState::default()).iter().any(|s| s.blur_radius == px(19.0));

        assert!(!blur());
        look.set_classic_params(ClassicButtonParams { fade_blur: 19.0, ..Default::default() });
        assert!(blur(), "editor retunes live buttons");
    }

    #[test]
    fn page_toggle_stays_borderless_when_unselected() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Page);
        let palette =
            theme.resolve(ButtonFamilyRole::Toggle { selected: false }, ControlSize::Md, InteractionState::default());

        assert_eq!(palette.background.a, 0.0);
        assert!(palette.border.is_none());
    }

    #[test]
    fn page_toggle_is_binary_off_on() {
        let look = Look::built_in();
        let theme = button_family_theme(&look, ButtonVariant::Page);
        let role = ButtonFamilyRole::Toggle { selected: true };
        let resting = theme.resolve(role, ControlSize::Md, InteractionState::default());
        let focused = theme.resolve(role, ControlSize::Md, InteractionState { focused: true, ..Default::default() });
        let pressed = theme.resolve(
            role,
            ControlSize::Md,
            InteractionState { hovered: true, pressed: true, ..Default::default() },
        );
        let hovered = theme.resolve(role, ControlSize::Md, InteractionState { hovered: true, ..Default::default() });

        assert!(resting.border.is_none(), "opaque page borders become SDK focus rings");
        assert_eq!(focused.border, resting.border);
        assert_eq!(focused.background, resting.background);
        assert_eq!(pressed.background, resting.background);
        assert_eq!(hovered.background, resting.background);
    }

    #[test]
    fn ghost_quiet_keeps_ghost_hover_without_focus_border() {
        let look = Look::built_in();
        let ghost = button_family_theme(&look, ButtonVariant::Ghost);
        let quiet = button_family_theme(&look, ButtonVariant::GhostQuiet);
        let hovered = InteractionState { hovered: true, ..InteractionState::default() };
        let focused = InteractionState { focused: true, ..InteractionState::default() };

        assert_eq!(
            quiet.resolve(ButtonFamilyRole::Text, ControlSize::Md, hovered).background,
            ghost.resolve(ButtonFamilyRole::Text, ControlSize::Md, hovered).background
        );
        assert!(ghost.resolve(ButtonFamilyRole::Text, ControlSize::Md, focused).border.is_some());
        assert!(quiet.resolve(ButtonFamilyRole::Text, ControlSize::Md, focused).border.is_none());
    }
}
