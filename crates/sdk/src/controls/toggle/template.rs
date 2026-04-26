use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, prelude::*, px};

use super::{ToggleKind, ToggleRenderModel};
use crate::controls::button_family_template::render_button_family_focus_ring;
use crate::theme::{ButtonFamilyRole, ButtonFamilyTheme, ButtonVariant, default_button_family_theme};

const DISABLED_OPACITY: f32 = 0.56;

/// Canonical template contract for `Toggle`.
pub trait ToggleTemplate: Send + Sync {
    fn render(&self, model: &ToggleRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

/// Default SDK implementation of [`ToggleTemplate`], backed by [`ButtonFamilyTheme`].
pub struct ThemedToggleTemplate {
    theme: Arc<dyn ButtonFamilyTheme>,
}

impl ThemedToggleTemplate {
    pub fn new(theme: Arc<dyn ButtonFamilyTheme>) -> Self {
        Self { theme }
    }
}

/// Shared default template instance for `Toggle`.
pub fn default_toggle_template() -> Arc<dyn ToggleTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToggleTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedToggleTemplate::new(default_button_family_theme()))).clone()
}

impl ToggleTemplate for ThemedToggleTemplate {
    fn render(&self, model: &ToggleRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(
            toggle_variant(model.kind),
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

fn toggle_variant(kind: ToggleKind) -> ButtonVariant {
    match kind {
        ToggleKind::Default => ButtonVariant::Default,
        ToggleKind::Primary => ButtonVariant::Primary,
    }
}

// ---- Compatibility aliases (legacy toggle_button naming) ----

pub trait ToggleButtonTemplate: ToggleTemplate {}

impl<T> ToggleButtonTemplate for T where T: ToggleTemplate + ?Sized {}

pub type ThemedToggleButtonTemplate = ThemedToggleTemplate;

pub fn default_toggle_button_template() -> Arc<dyn ToggleButtonTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ToggleButtonTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedToggleTemplate::new(default_button_family_theme()))).clone()
}
