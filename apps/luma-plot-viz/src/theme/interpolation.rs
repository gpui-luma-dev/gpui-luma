use gpui::Context;
use gpui_luma::controls::radio_group::{self, RadioGroup, RadioGroupItem};
use gpui_luma_look_radix::Look;
use gpui_luma_plot::Interpolation;

pub(crate) fn spawn<M: 'static>(
    id: &'static str,
    look: &Look,
    interpolation: Interpolation,
    cx: &mut Context<M>,
) -> RadioGroup<RadioGroupItem> {
    radio_group::horizontal(id)
        .single_required()
        .selection_follows_active(true)
        .items([RadioGroupItem::new("linear").label("Straight"), RadioGroupItem::new("monotone").label("Monotone")])
        .selected(match interpolation {
            Interpolation::Linear => "linear",
            Interpolation::Monotone => "monotone",
        })
        .template(gpui_luma_look_radix::segmented_radio_template(look))
        .spawn(cx)
}
