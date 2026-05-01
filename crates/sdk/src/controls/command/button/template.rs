use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::ButtonRenderModel;
use crate::controls::button_family::ButtonKind;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;

use crate::controls::template::{Modifier, TemplateWithModifiers};

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

pub fn default_button_template<D: 'static>() -> Arc<dyn ButtonTemplate<D>> {
    Arc::new(DefaultButtonTemplate::new(default_button_family_theme()))
}

impl<D: 'static> ButtonTemplate<D> for DefaultButtonTemplate<D> {
    fn render(&self, model: &ButtonRenderModel<D>, _window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(button_variant(model.kind), model.role, model.size, model.state);

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

        let mut root = render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, radius);

        if model.state.disabled {
            root = root.opacity(DISABLED_OPACITY);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Standard => ButtonVariant::Standard,
        ButtonKind::Ghost => ButtonVariant::Ghost,
        ButtonKind::Prominent => ButtonVariant::Prominent,
    }
}
