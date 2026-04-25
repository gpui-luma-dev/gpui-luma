use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::ToggleButtonRenderModel;
use crate::controls::button_family::ButtonKind;
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;

pub trait ToggleButtonTemplate: Send + Sync {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedToggleButtonTemplate {
    theme: Arc<dyn ButtonFamilyTheme>,
}

impl ThemedToggleButtonTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_toggle_button_template() -> Arc<dyn ToggleButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToggleButtonTemplate>> = OnceLock::new();

    TEMPLATE
        .get_or_init(|| Arc::new(ThemedToggleButtonTemplate::new(default_button_family_theme())))
        .clone()
}

impl ToggleButtonTemplate for ThemedToggleButtonTemplate {
    fn render(&self, model: &ToggleButtonRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(
            button_variant(model.kind),
            ButtonFamilyRole::Toggle { selected: model.selected },
            model.size,
            model.state,
        );
        let control = div()
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

fn button_variant(kind: ButtonKind) -> ButtonVariant {
    match kind {
        ButtonKind::Default => ButtonVariant::Default,
        ButtonKind::Primary => ButtonVariant::Primary,
        ButtonKind::Destructive => ButtonVariant::Destructive,
    }
}
