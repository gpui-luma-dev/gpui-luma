use std::sync::Arc;

use gpui::{AnyElement, App, Window};

use super::activity::ActivityPowerCurve;
use super::charts::render_power_curve_tab as render_power_curve_chart;
use gpui_luma_look_shadcn::ShadcnLook;

pub fn render_power_curve_tab(
    curve: &ActivityPowerCurve,
    look: &Arc<ShadcnLook>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    render_power_curve_chart(curve, look, window, cx)
}
