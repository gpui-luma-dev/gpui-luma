

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::ModButtonRenderModel;
use crate::controls::button_family::ButtonKind;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;

use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;


pub trait ModButtonTemplate: Send + Sync {
    fn render(&self, model: &ModButtonRenderModel, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

define_control_template!(
    ButtonTemplate,
    dyn ButtonFamilyTheme,
    ModButtonRenderModel,
    ModButtonTemplate,
    default_button_family_theme()
);

impl ModButtonTemplate for ButtonTemplate {
    fn render(&self, model: &ModButtonRenderModel, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance =
            self.theme.resolve(button_variant(model.kind), ButtonFamilyRole::Text, model.size, model.state);

        let mut control = div()
            .id(format!("{}-control", model.id))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(appearance.gap))
            .px(px(appearance.padding_x))
            .py(px(appearance.padding_y))
            .h(px(appearance.height))
            .bg(appearance.background)
            .text_color(appearance.foreground)
            .border_1()
            .border_color(appearance.border)
            .rounded(px(appearance.radius))
            .text_size(px(appearance.typography.size))
            .line_height(px(appearance.typography.line_height))
            .font_weight(appearance.typography.weight)
            .child(model.label.clone());

        // Generic pipeline call
        control = self.apply_modifiers(control, model);

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

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
