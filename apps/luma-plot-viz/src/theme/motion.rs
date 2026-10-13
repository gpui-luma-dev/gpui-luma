use gpui::Context;
use gpui_luma_look_radix::Look;
use gpui_luma::prelude::HasPresenter;

/// Replay chrome uses the same SDK button factory and runtime look as the app.
pub(crate) fn spawn_replay<M: 'static>(
    id: &'static str,
    look: &Look,
    cx: &mut Context<M>,
) -> gpui::Entity<gpui_luma::controls::button::Button<()>> {
    gpui_luma_look_radix::Button::new(id).look(look).outline().label("Replay").spawn(cx)
}

pub(crate) fn spawn_plot_action<M: 'static>(
    id: &'static str,
    label: &'static str,
    look: &Look,
    cx: &mut gpui::Context<M>,
) -> gpui::Entity<gpui_luma::controls::button::Button<()>> {
    gpui_luma_look_radix::Button::new(id).look(look).outline().label(label).spawn(cx)
}
