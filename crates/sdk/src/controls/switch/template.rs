use std::sync::{Arc, OnceLock};

use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use super::SwitchRenderModel;
use crate::controls::state::focus_debug_border;
use crate::theme::{SwitchTheme, default_switch_theme};

pub trait SwitchTemplate: Send + Sync {
    fn render(&self, model: &SwitchRenderModel<'_>, window: &mut Window, cx: &mut App) -> Stateful<Div>;
}

pub struct ThemedSwitchTemplate {
    theme: Arc<dyn SwitchTheme>,
}

impl ThemedSwitchTemplate {
    pub fn new(theme: Arc<dyn SwitchTheme>) -> Self {
        Self { theme }
    }
}

pub fn default_switch_template() -> Arc<dyn SwitchTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn SwitchTemplate>> = OnceLock::new();

    TEMPLATE.get_or_init(|| Arc::new(ThemedSwitchTemplate::new(default_switch_theme()))).clone()
}

impl SwitchTemplate for ThemedSwitchTemplate {
    fn render(&self, model: &SwitchRenderModel<'_>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.on, model.state);
        let thumb_left = if model.on {
            appearance.width - appearance.thumb_size - appearance.padding
        } else {
            appearance.padding
        };
        let thumb_top = ((appearance.height - appearance.thumb_size) * 0.5 - 1.0).max(0.0);

        let track = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(appearance.width))
            .h(px(appearance.height))
            .bg(appearance.track_background)
            .border_1()
            .border_color(appearance.track_border)
            .rounded(px(appearance.radius))
            .child(
                div()
                    .id(format!("{}-thumb", model.id))
                    .absolute()
                    .left(px(thumb_left))
                    .top(px(thumb_top))
                    .size(px(appearance.thumb_size))
                    .bg(appearance.thumb_background)
                    .border_1()
                    .border_color(appearance.thumb_border)
                    .rounded(px(appearance.radius))
                    .shadow(appearance.thumb_shadow.clone()),
            );

        let mut root = div()
            .id(model.id.clone())
            .relative()
            .flex()
            .items_center()
            .gap(px(appearance.gap))
            .text_color(appearance.label_color)
            .text_size(px(appearance.label_typography.size))
            .line_height(px(appearance.label_typography.line_height))
            .font_weight(appearance.label_typography.weight)
            .rounded(px(appearance.radius))
            .child(track);

        if let Some(label) = model.label {
            root = root.child(label.clone());
        }

        if model.enabled {
            root = root.cursor_pointer();
        } else {
            root = root.opacity(0.56);
        }

        if model.state.focused {
            root = root
                .child(render_focus_ring(appearance.focus_ring.unwrap_or_else(focus_debug_border), appearance.radius));
        }

        root
    }
}

fn render_focus_ring(color: gpui::Hsla, radius: f32) -> Div {
    div().absolute().size_full().border_1().border_color(color).rounded(px(radius))
}
