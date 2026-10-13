use gpui::Context;
use gpui_luma::controls::{
    radio_group::{self, RadioGroup, RadioGroupItem},
    tooltip::{Tooltip, TooltipPlacement},
};
use gpui_luma_look_radix::Look;
use gpui_luma_plot::{MarkerVisibility, PlotPoint, PlotSlice};

pub(crate) fn spawn<M: 'static>(id: &'static str, look: &Look, cx: &mut Context<M>) -> RadioGroup<RadioGroupItem> {
    radio_group::horizontal(id)
        .single_required()
        .selection_follows_active(true)
        .items([
            RadioGroupItem::new("hover").label("Hover point"),
            RadioGroupItem::new("slice").label("Hover slice"),
            RadioGroupItem::new("all").label("All points"),
        ])
        .selected("slice")
        .template(gpui_luma_look_radix::segmented_radio_template(look))
        .spawn(cx)
}

pub(crate) fn visibility(id: &str) -> MarkerVisibility {
    match id {
        "all" => MarkerVisibility::Always,
        "slice" => MarkerVisibility::HoverSlice,
        _ => MarkerVisibility::Hover,
    }
}

pub(crate) fn tooltip(look: &Look, point: &PlotPoint, coordinate_label: String) -> Tooltip {
    Tooltip::new(format!("{} · {}", coordinate_label, point.label))
        .shortcut(point.sample.y.to_string())
        .theme(gpui_luma_look_radix::tooltip_theme(look))
}

pub(crate) fn slice_tooltip(look: &Look, slice: &PlotSlice, coordinate_label: String) -> Tooltip {
    if let Some(point) = slice.points.first().filter(|_| slice.points.len() == 1) {
        return tooltip(look, point, coordinate_label).delay(std::time::Duration::ZERO);
    }
    let values = slice
        .points
        .iter()
        .map(|point| format!("{}: {}", point.label, point.sample.y))
        .collect::<Vec<_>>()
        .join("\n");
    Tooltip::new(format!("{}\n{}", coordinate_label, values))
        .delay(std::time::Duration::ZERO)
        .theme(gpui_luma_look_radix::tooltip_theme(look))
}

pub(crate) fn spawn_motion<M: 'static>(look: &Look, cx: &mut Context<M>) -> RadioGroup<RadioGroupItem> {
    options("point-tooltip-motion", look, [("snap", "Immediate"), ("smooth", "Smooth")], "smooth", cx)
}

pub(crate) fn spawn_placement<M: 'static>(look: &Look, cx: &mut Context<M>) -> RadioGroup<RadioGroupItem> {
    options("point-tooltip-placement", look, [("above", "Above"), ("below", "Below")], "above", cx)
}

pub(crate) fn placement(id: &str) -> TooltipPlacement {
    if id == "below" {
        TooltipPlacement::Below
    } else {
        TooltipPlacement::Above
    }
}

pub(crate) fn motion(id: &str) -> std::time::Duration {
    if id == "smooth" {
        std::time::Duration::from_millis(250)
    } else {
        std::time::Duration::ZERO
    }
}

fn options<M: 'static>(
    id: &'static str,
    look: &Look,
    choices: [(&'static str, &'static str); 2],
    selected: &'static str,
    cx: &mut Context<M>,
) -> RadioGroup<RadioGroupItem> {
    radio_group::horizontal(id)
        .single_required()
        .selection_follows_active(true)
        .items(choices.map(|(id, label)| RadioGroupItem::new(id).label(label)))
        .selected(selected)
        .template(gpui_luma_look_radix::segmented_radio_template(look))
        .spawn(cx)
}
