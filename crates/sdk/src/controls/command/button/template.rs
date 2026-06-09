use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::ButtonRenderModel;
use crate::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyTheme, button_family_effective_border, button_family_focus_adorner,
    compose_button_family_appearance, default_button_family_theme,
};
use crate::theme::InteractionState;

const DISABLED_OPACITY: f32 = 0.56;

use crate::controls::template::{Modifier, TemplateWithModifiers};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};
use crate::theme::{LayoutCacheKey, LumaLayoutCacheExt, StandardBoxScale};

fn resolve_theme_appearance<D>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
    scale: &StandardBoxScale,
) -> ButtonFamilyAppearance {
    let palette = theme.resolve(model.role, model.size, model.state);
    compose_button_family_appearance(&palette, model.role, scale, theme.metrics().radius.pill)
}

fn resolve_appearance<D>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
    scale: &StandardBoxScale,
) -> ButtonFamilyAppearance {
    if let Some(resolve) = &model.appearance {
        return resolve(model);
    }

    resolve_theme_appearance(theme, model, scale)
}

fn resolve_focus_probe_appearance<D: Clone>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
    scale: &StandardBoxScale,
) -> Option<ButtonFamilyAppearance> {
    if model.state.disabled {
        return None;
    }

    if let Some(resolve) = &model.appearance {
        let focused_state = InteractionState { focused: true, ..model.state };
        let focused_model = ButtonRenderModel {
            id: model.id.clone(),
            data: model.data.clone(),
            content: model.content.clone(),
            role: model.role,
            size: model.size,
            state: focused_state,
            round: model.round,
            radius_override: std::cell::Cell::new(model.radius_override.get()),
            appearance: model.appearance.clone(),
        };
        return Some(resolve(&focused_model));
    }

    let focused_state = InteractionState { focused: true, ..model.state };
    let palette = theme.resolve(model.role, model.size, focused_state);
    Some(compose_button_family_appearance(&palette, model.role, scale, theme.metrics().radius.pill))
}

pub trait ButtonTemplate<D = ()>: Send + Sync {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct DefaultButtonTemplate<D = ()> {
    pub theme: Arc<dyn ButtonFamilyTheme>,
    pub modifiers: Vec<Modifier<ButtonRenderModel<D>>>,
}

impl<D> DefaultButtonTemplate<D> {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme, modifiers: Vec::new() }
    }

    pub fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(Stateful<Div>, &ButtonRenderModel<D>) -> Stateful<Div> + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl<D: 'static> TemplateWithModifiers<ButtonRenderModel<D>> for DefaultButtonTemplate<D> {
    fn modifiers(&self) -> &[Modifier<ButtonRenderModel<D>>] {
        &self.modifiers
    }
}

pub fn default_button_template<D: Clone + 'static>() -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()))
}

impl<D: 'static + Clone> ButtonTemplate<D> for DefaultButtonTemplate<D> {
    fn render(&self, model: &ButtonRenderModel<D>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let scale_factor = window.scale_factor();
        let scale = cx.use_cached_layout(
            self.theme.metrics(),
            LayoutCacheKey { size: model.size, scale_factor_bits: scale_factor.to_bits() },
            |metrics| StandardBoxScale::compute(model.size, metrics, scale_factor),
        );
        let appearance = resolve_appearance(&self.theme, model, &scale);
        let focused_probe_appearance = resolve_focus_probe_appearance(&self.theme, model, &scale);
        let border = button_family_effective_border(appearance.border);

        let mut control = div()
            .id(format!("{}-control", model.id))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_family(appearance.font_family.clone())
            .font_weight(appearance.typography.weight)
            .h(px(appearance.height));

        if border.a > 0.0 {
            control = control.border_1().border_color(border);
        }

        if model.round {
            control = control.w(px(appearance.height)).p_0().rounded_full();
        } else {
            control = control.px(px(appearance.padding_x)).py(px(appearance.padding_y)).rounded(px(appearance.radius));
        }

        control = control.child(div().text_color(appearance.foreground).child((model.content)(model, cx)));

        // Generic pipeline call
        control = self.apply_modifiers(control, model);

        let radius = if let Some(r) = model.radius_override.get() {
            r
        } else if model.round {
            appearance.height / 2.0
        } else {
            appearance.radius
        };

        let metrics = self.theme.metrics();
        let adorner =
            button_family_focus_adorner(model.state.focused, appearance.border, appearance.focus_ring, metrics);
        let focused_adorner = focused_probe_appearance
            .as_ref()
            .and_then(|probe| button_family_focus_adorner(true, probe.border, probe.focus_ring, metrics));

        let oversize_extent = adorner_oversize_extent(adorner).max(adorner_oversize_extent(focused_adorner));

        let mut adorned = div().id(format!("{}-adorned", model.id)).relative().child(control);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(adorner, radius) {
            adorned = adorned.child(adorner);
        }

        let mut root = div().id(model.id.clone()).relative();
        root = if oversize_extent > 0.0 {
            root.p(px(oversize_extent)).child(adorned)
        } else {
            root.child(adorned)
        };

        if model.state.disabled {
            root = root.opacity(DISABLED_OPACITY);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gpui::Hsla;

    use super::*;
    use crate::controls::button_family::{ButtonFamilyAppearance, ButtonFamilyPalette, ButtonFamilyRole, ButtonSize};
    use crate::theme::{InteractionState, LumaTextStyle};

    fn lime_appearance() -> ButtonFamilyAppearance {
        ButtonFamilyAppearance {
            background: Hsla { h: 120.0, s: 1.0, l: 0.5, a: 1.0 },
            foreground: Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 },
            border: Some(Hsla { h: 120.0, s: 1.0, l: 0.3, a: 1.0 }),
            focus_ring: Hsla { h: 200.0, s: 1.0, l: 0.5, a: 1.0 },
            typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: gpui::FontWeight::MEDIUM },
            font_family: "test".into(),
            radius: 8.0,
            padding_x: 12.0,
            padding_y: 6.0,
            gap: 6.0,
            height: 32.0,
        }
    }

    #[test]
    fn with_appearance_overrides_style_resolution() {
        let template: DefaultButtonTemplate<()> = DefaultButtonTemplate::new(default_button_family_theme());
        let model = ButtonRenderModel {
            id: "appearance-test".into(),
            data: (),
            content: Arc::new(|_, _| div().into_any_element()),
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: InteractionState::default(),
            round: false,
            radius_override: std::cell::Cell::new(None),
            appearance: Some(Arc::new(|_| lime_appearance())),
        };

        let scale = StandardBoxScale { height: 32.0, padding_x: 12.0, padding_y: 6.0, gap: 6.0, radius: 8.0 };
        let appearance = resolve_appearance(&template.theme, &model, &scale);
        assert_eq!(appearance.background, lime_appearance().background);
    }

    #[test]
    fn compose_button_family_appearance_uses_box_scale_geometry() {
        let palette = ButtonFamilyPalette {
            background: Hsla::default(),
            foreground: Hsla::default(),
            border: Some(Hsla::default()),
            focus_ring: Hsla::default(),
            typography: LumaTextStyle { size: 14.0, line_height: 20.0, weight: gpui::FontWeight::MEDIUM },
            font_family: "test".into(),
        };
        let scale = StandardBoxScale { height: 36.0, padding_x: 14.0, padding_y: 8.0, gap: 8.0, radius: 6.0 };

        let appearance = compose_button_family_appearance(&palette, ButtonFamilyRole::Text, &scale, 999.0);
        assert_eq!(appearance.height, 36.0);
        assert_eq!(appearance.padding_x, 14.0);
        assert_eq!(appearance.padding_y, 8.0);
        assert_eq!(appearance.gap, 8.0);
        assert_eq!(appearance.radius, 6.0);
    }
}
