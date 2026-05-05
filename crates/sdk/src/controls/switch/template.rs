use gpui::{App, Div, Stateful, Window, div, px, prelude::*};

use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};

use crate::theme::adorner::{AdornerSpec, render_adorner};
use crate::controls::switch::{SwitchTheme, default_switch_theme};

use crate::controls::template::TemplateWithModifiers;
use crate::define_control_template;

define_control_template!(
    ThemedSwitchTemplate,
    dyn SwitchTheme,
    ButtonRenderModel<bool>,
    ButtonTemplate<bool>,
    default_switch_theme()
);

impl ButtonTemplate<bool> for ThemedSwitchTemplate {
    fn render(&self, model: &ButtonRenderModel<bool>, _window: &mut Window, _cx: &mut App) -> Stateful<Div> {
        let appearance = self.theme.resolve(model.data, model.state);
        let thumb_left = if model.data {
            appearance.width - appearance.thumb_size - appearance.padding
        } else {
            appearance.padding
        };
        let thumb_top = ((appearance.height - appearance.thumb_size) * 0.5 - 1.0).max(0.0);
        let thumb = div()
            .id(format!("{}-thumb", model.id))
            .absolute()
            .left(px(thumb_left))
            .top(px(thumb_top))
            .size(px(appearance.thumb_size))
            .bg(appearance.thumb_background)
            .border_1()
            .border_color(appearance.thumb_border)
            .rounded(px(appearance.radius))
            .shadow(appearance.thumb_shadow.clone());

        let track_visual = div()
            .id(format!("{}-track", model.id))
            .relative()
            .w(px(appearance.width))
            .h(px(appearance.height))
            .bg(appearance.track_background)
            .border_1()
            .border_color(appearance.track_border)
            .rounded(px(appearance.radius))
            .child(thumb);

        let mut track = div().relative().child(track_visual);

        for spec in &appearance.adorners {
            if let Some(mut adorner) = render_adorner(*spec, appearance.radius) {
                let AdornerSpec::FocusRing(focus_ring) = *spec;
                let focus_ring_radius = match focus_ring.placement {
                    crate::theme::AdornerPlacement::Inset => {
                        (appearance.radius - focus_ring.distance.max(0.0) - focus_ring.width.max(0.0)).max(0.0)
                    }
                    crate::theme::AdornerPlacement::Oversize => appearance.radius + focus_ring.distance.max(0.0),
                };
                adorner = adorner.rounded(px(focus_ring_radius));
                track = track.child(adorner);
            }
        }

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
            .child(track)
            .child((model.content)(model, _cx));

        if model.state.disabled {
            root = root.opacity(0.56);
        } else {
            root = root.cursor_pointer();
        }

        // Apply modifiers from the pipeline
        self.apply_modifiers(root, model)
    }
}
