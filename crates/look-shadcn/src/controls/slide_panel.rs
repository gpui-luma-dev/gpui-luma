use std::sync::Arc;

use gpui::Hsla;

use gpui_luma::controls::resizable_panels::ResizablePanelsLook;
use gpui_luma::theme::InteractionState;

use crate::ShadcnLook;

pub fn slide_panel_panels_look(look: &Arc<ShadcnLook>) -> ResizablePanelsLook {
    look.resizable_panels_theme().resolve(InteractionState::default())
}

pub fn slide_panel_background(look: &Arc<ShadcnLook>) -> Hsla {
    look.token_color("card").unwrap_or_else(|_| look.chrome().panel_background)
}
