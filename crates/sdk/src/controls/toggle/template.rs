

use gpui::{App, Div, Stateful, Window, div, prelude::*, px};

use super::{ToggleKind, ToggleRenderModel};
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;

/// Canonical template contract for `Toggle`.
use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;

pub trait ToggleTemplate: Send + Sync {
    fn render(&self, model: &ToggleRenderModel, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

define_control_template!(
    ThemedToggleTemplate,
    dyn ButtonFamilyTheme,
    ToggleRenderModel,
    ToggleTemplate,
    default_button_family_theme()
);

impl ToggleTemplate for ThemedToggleTemplate {
    fn render(&self, model: &ToggleRenderModel, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(
            toggle_variant(model.kind),
            ButtonFamilyRole::Toggle { selected: model.selected },
            model.size,
            model.state,
        );

        let control = div()
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

        // Apply modifiers from the pipeline
        let control = self.apply_modifiers(control, model);

        let mut root =
            render_button_family_focus_ring(model.id.clone(), control, appearance.focus_ring, appearance.radius);

        if !model.enabled {
            root = root.opacity(DISABLED_OPACITY);
        } else {
            root = root.cursor_pointer();
        }

        root
    }
}

fn toggle_variant(kind: ToggleKind) -> ButtonVariant {
    match kind {
        ToggleKind::Standard => ButtonVariant::Standard,
        ToggleKind::Ghost => ButtonVariant::Ghost,
        ToggleKind::Prominent => ButtonVariant::Prominent,
    }
}

// ---- Compatibility aliases (legacy toggle_button naming) ----

// ---- Compatibility aliases (legacy toggle_button naming) ----

pub trait ToggleButtonTemplate: ToggleTemplate {}
impl<T: ToggleTemplate + ?Sized> ToggleButtonTemplate for T {}

pub type ThemedToggleButtonTemplate = ThemedToggleTemplate;

pub use default_template as default_toggle_button_template;
