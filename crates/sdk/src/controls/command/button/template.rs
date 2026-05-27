use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::ButtonRenderModel;
use crate::controls::button_family::{
    ButtonFamilyAppearance, ButtonFamilyTheme, button_variant, default_button_family_theme,
};
use crate::theme::InteractionState;

const DISABLED_OPACITY: f32 = 0.56;

use crate::controls::template::{Modifier, TemplateWithModifiers};
use crate::theme::adorner::{adorner_oversize_extent, render_optional_adorner_with_focus_radius};

fn resolve_appearance<D>(theme: &Arc<dyn ButtonFamilyTheme>, model: &ButtonRenderModel<D>) -> ButtonFamilyAppearance {
    if let Some(resolve) = &model.appearance {
        return resolve(model);
    }

    theme.resolve(button_variant(model.kind), model.role, model.size, model.state)
}

fn resolve_focus_probe_appearance<D: Clone>(
    theme: &Arc<dyn ButtonFamilyTheme>,
    model: &ButtonRenderModel<D>,
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
            kind: model.kind,
            role: model.role,
            size: model.size,
            state: focused_state,
            round: model.round,
            radius_override: std::cell::Cell::new(model.radius_override.get()),
            appearance: model.appearance.clone(),
        };
        return Some(resolve(&focused_model));
    }

    Some(theme.resolve(
        button_variant(model.kind),
        model.role,
        model.size,
        InteractionState { focused: true, ..model.state },
    ))
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
    fn render(&self, model: &ButtonRenderModel<D>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = resolve_appearance(&self.theme, model);
        let focused_probe_appearance = resolve_focus_probe_appearance(&self.theme, model);

        let mut control = div()
            .id(format!("{}-control", model.id))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_family(appearance.font_family.clone())
            .font_weight(appearance.typography.weight)
            .h(px(appearance.height));

        if model.round {
            control = control.w(px(appearance.height)).p_0().rounded_full();
        } else {
            control = control.px(px(appearance.padding_x)).py(px(appearance.padding_y)).rounded(px(appearance.radius));
        }

        control = control.child((model.content)(model, cx));

        // Generic pipeline call
        control = self.apply_modifiers(control, model);

        let radius = if let Some(r) = model.radius_override.get() {
            r
        } else if model.round {
            appearance.height / 2.0
        } else {
            appearance.radius
        };

        let oversize_extent = adorner_oversize_extent(appearance.adorner)
            .max(focused_probe_appearance.as_ref().map(|probe| adorner_oversize_extent(probe.adorner)).unwrap_or(0.0));

        let mut adorned = div().id(format!("{}-adorned", model.id)).relative().child(control);

        if let Some(adorner) = render_optional_adorner_with_focus_radius(appearance.adorner, radius) {
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
    use crate::controls::button_family::{ButtonFamilyAppearance, ButtonFamilyRole, ButtonKind, ButtonSize};
    use crate::theme::{InteractionState, LumaTextStyle};

    fn lime_appearance() -> ButtonFamilyAppearance {
        ButtonFamilyAppearance {
            background: Hsla { h: 120.0, s: 1.0, l: 0.5, a: 1.0 },
            foreground: Hsla { h: 0.0, s: 0.0, l: 1.0, a: 1.0 },
            border: Hsla { h: 120.0, s: 1.0, l: 0.3, a: 1.0 },
            adorner: None,
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
    fn with_appearance_overrides_kind_resolution() {
        let template: DefaultButtonTemplate<()> = DefaultButtonTemplate::new(default_button_family_theme());
        let model = ButtonRenderModel {
            id: "appearance-test".into(),
            data: (),
            content: Arc::new(|_, _| div().into_any_element()),
            kind: ButtonKind::Prominent,
            role: ButtonFamilyRole::Text,
            size: ButtonSize::Md,
            state: InteractionState::default(),
            round: false,
            radius_override: std::cell::Cell::new(None),
            appearance: Some(Arc::new(|_| lime_appearance())),
        };

        let appearance = resolve_appearance(&template.theme, &model);
        assert_eq!(appearance.background, lime_appearance().background);
    }
}
