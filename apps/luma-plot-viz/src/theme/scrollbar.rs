use std::sync::Arc;

use gpui::{AppContext, Entity};
use gpui_luma::controls::scrollbar::{
    DefaultScrollbarTheme, Scrollbar, ScrollbarLook, ScrollbarOrientation, ScrollbarStyle, ScrollbarTheme,
    ThemedScrollbarTemplate,
};
use gpui_luma::theme::{ControlSize, InteractionLayer, InteractionState};
use gpui_luma_look_radix::{Look, SemanticRole};

struct RadixScrollbarTheme(Look);

impl ScrollbarTheme for RadixScrollbarTheme {
    fn resolve(
        &self,
        state: InteractionState,
        orientation: ScrollbarOrientation,
        size: ControlSize,
        style: ScrollbarStyle,
    ) -> ScrollbarLook {
        let mut visual = DefaultScrollbarTheme::default().resolve(state, orientation, size, style);
        visual.track_background = self.0.resolve_role(SemanticRole::Border).hsla();
        let role = match state.layer() {
            InteractionLayer::Pressed => SemanticRole::Primary,
            InteractionLayer::Hovered => SemanticRole::Foreground,
            InteractionLayer::Disabled => SemanticRole::Border,
            InteractionLayer::Default => SemanticRole::MutedForeground,
        };
        visual.thumb_background = self.0.resolve_role(role).hsla();
        visual
    }
}

pub(crate) fn spawn_time_scrollbar(
    look: &Look,
    content: (f64, f64),
    span: f64,
    cx: &mut impl AppContext,
) -> Entity<Scrollbar> {
    Scrollbar::new("plot-viz-time-scrollbar")
        .horizontal()
        .step(0.1)
        .page_step(span * 0.8)
        .viewport(content.0, content.1, content.0, content.0 + span)
        .template(Arc::new(ThemedScrollbarTemplate::new(Arc::new(RadixScrollbarTheme(look.clone())))))
        .spawn(cx)
}
