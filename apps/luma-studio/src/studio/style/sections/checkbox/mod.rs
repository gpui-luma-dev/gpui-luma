use std::sync::Arc;

use gpui::{AnyElement, App, Entity, Window};
use luma::controls::tabs_navigation::TabsNavigation;
use luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::choice_matrix::{ChoiceTemplateControl, render_choice_control_template_matrix_section};

pub(crate) fn render_checkbox_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_control_template_matrix_section(
        look,
        ChoiceTemplateControl::Checkbox,
        "Checkbox",
        "Style variants by row. Pressed and Disabled · On columns show checked.",
        preview_tabs,
        window,
        cx,
    )
}
