use std::sync::Arc;

use gpui::{AnyElement, App, Entity, Window};
use gpui_luma::controls::tabs_navigation::TabsNavigation;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::style::shared::choice_matrix::{ChoiceTemplateControl, render_choice_control_template_matrix_section};

pub(crate) fn render_switch_template_matrix_section(
    look: Arc<ShadcnLook>,
    preview_tabs: Entity<TabsNavigation>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_choice_control_template_matrix_section(
        look,
        ChoiceTemplateControl::Switch,
        "Switch",
        "Style variants by row. Sizes tab: Sm/Md/Lg × radius (No radius → Full), primary style.",
        preview_tabs,
        window,
        cx,
    )
}
