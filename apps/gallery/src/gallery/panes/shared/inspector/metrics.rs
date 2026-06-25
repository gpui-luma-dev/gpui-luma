use gpui::SharedString;
use gpui_luma_look_shadcn_inspect::ShadcnInspect;
use gpui_luma_look_shadcn_inspect::{
    BadgeInspectMetrics, ButtonInspectMetrics, CardInspectMetrics, CheckboxInspectMetrics, RadioButtonInspectMetrics,
    ResolvedMetric, SliderInspectMetrics, SwitchInspectMetrics, TextFieldInspectMetrics,
    format_inspect_metric_provenance, format_inspect_metric_source, format_metric_px,
};

use super::types::InspectMetricPropertyData;

pub(in crate::gallery) fn metric_properties(
    fields: &[(&'static str, &ResolvedMetric)],
) -> Vec<InspectMetricPropertyData> {
    fields
        .iter()
        .map(|(name, metric)| InspectMetricPropertyData {
            name: SharedString::from(*name),
            value: format_metric_px(metric.value_px).into(),
            source: format_inspect_metric_source(&metric.source).into(),
            provenance: format_inspect_metric_provenance(&metric.source).map(SharedString::from),
        })
        .collect()
}

pub(in crate::gallery) fn checkbox_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_checkbox_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: checkbox_metric_properties(&metrics) }
}

pub(in crate::gallery) fn radio_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_radio_button_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: radio_metric_properties(&metrics) }
}

pub(in crate::gallery) fn switch_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_switch_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: switch_metric_properties(&metrics) }
}

pub(in crate::gallery) fn toggle_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    style: gpui_luma_look_shadcn::ShadcnButtonStyle,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    use gpui_luma::controls::button_family::ButtonFamilyRole;
    use gpui_luma::theme::InteractionState;

    let metrics = ShadcnInspect::new(look).inspect_button_metrics(
        style,
        ButtonFamilyRole::Toggle { selected: true },
        size,
        InteractionState::default(),
    );
    super::types::InspectLayoutSizeData { size, properties: button_metric_properties(&metrics) }
}

pub(in crate::gallery) fn textarea_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_textarea_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: textfield_metric_properties(&metrics) }
}

pub(in crate::gallery) fn scrollbar_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    orientation: gpui_luma::controls::scrollbar::ScrollbarOrientation,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_scrollbar_metrics(orientation);
    super::types::InspectLayoutSizeData {
        size: gpui_luma::theme::ControlSize::Md,
        properties: scrollbar_metric_properties(&metrics),
    }
}

pub(in crate::gallery) fn slider_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_slider_metrics();
    super::types::InspectLayoutSizeData {
        size: gpui_luma::theme::ControlSize::Md,
        properties: slider_metric_properties(&metrics),
    }
}

pub(in crate::gallery) fn progress_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_progress_metrics();
    super::types::InspectLayoutSizeData {
        size: gpui_luma::theme::ControlSize::Md,
        properties: progress_metric_properties(&metrics),
    }
}

pub(in crate::gallery) fn card_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_card_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: card_metric_properties(&metrics) }
}

pub(in crate::gallery) fn badge_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_badge_metrics(gpui_luma_look_shadcn::BadgeVariant::Default, size);
    super::types::InspectLayoutSizeData { size, properties: badge_metric_properties(&metrics) }
}

fn textfield_metric_properties(metrics: &TextFieldInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("min height", &metrics.min_height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("radius", &metrics.radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ])
}

fn scrollbar_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ScrollbarInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("length", &metrics.length),
        ("thickness", &metrics.thickness),
        ("track thickness", &metrics.track_thickness),
        ("thumb thickness", &metrics.thumb_thickness),
        ("min thumb length", &metrics.min_thumb_length),
        ("radius", &metrics.radius),
    ])
}

fn slider_metric_properties(metrics: &SliderInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("width", &metrics.width),
        ("height", &metrics.height),
        ("track height", &metrics.track_height),
        ("thumb size", &metrics.thumb_size),
        ("radius", &metrics.radius),
    ])
}

fn progress_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ProgressInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[("size", &metrics.size), ("stroke width", &metrics.stroke_width)])
}

fn card_metric_properties(metrics: &CardInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("padding", &metrics.padding),
        ("section gap", &metrics.section_gap),
        ("header gap", &metrics.header_gap),
        ("body gap", &metrics.body_gap),
        ("radius", &metrics.radius),
    ])
}

fn badge_metric_properties(metrics: &BadgeInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("min height", &metrics.min_height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
        ("icon size", &metrics.icon_size),
        ("radius", &metrics.radius),
    ])
}

fn tabs_navigation_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::TabsNavigationInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("list radius", &metrics.list_radius),
        ("list gap", &metrics.list_gap),
        ("list padding", &metrics.list_padding),
        ("item padding x", &metrics.item_padding_x),
        ("item height", &metrics.item_height),
        ("item radius", &metrics.item_radius),
        ("indicator height", &metrics.indicator_height),
    ])
}

pub(in crate::gallery) fn tabs_navigation_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_tabs_navigation_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: tabs_navigation_metric_properties(&metrics) }
}

pub(in crate::gallery) fn listbox_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_listbox_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: listbox_metric_properties(&metrics) }
}

pub(in crate::gallery) fn control_group_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_control_group_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: control_group_metric_properties(&metrics) }
}

fn control_group_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ControlGroupInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("radius", &metrics.radius),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
    ])
}

fn listbox_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ListBoxInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("radius", &metrics.radius),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("row gap", &metrics.row_gap),
        ("row min height", &metrics.row_min_height),
        ("row padding x", &metrics.row_padding_x),
        ("row padding y", &metrics.row_padding_y),
    ])
}

pub(in crate::gallery) fn tree_view_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_tree_view_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: tree_view_metric_properties(&metrics) }
}

pub(in crate::gallery) fn navigation_sidebar_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_navigation_sidebar_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: navigation_sidebar_metric_properties(&metrics) }
}

pub(in crate::gallery) fn autocomplete_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_autocomplete_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: autocomplete_metric_properties(&metrics) }
}

pub(in crate::gallery) fn textfield_and_menu_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let textfield = ShadcnInspect::new(look).inspect_textfield_metrics(size);
    let menu = ShadcnInspect::new(look).inspect_floating_menu_metrics(size);
    super::types::InspectLayoutSizeData {
        size,
        properties: [textfield_metric_properties(&textfield), floating_menu_metric_properties(&menu)].concat(),
    }
}

fn tree_view_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::TreeViewInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("row height", &metrics.row_height),
        ("base padding x", &metrics.base_padding_x),
        ("indentation width", &metrics.indentation_width),
        ("inner gap", &metrics.inner_gap),
        ("radius", &metrics.radius),
        ("icon size", &metrics.icon_size),
        ("chevron size", &metrics.chevron_size),
    ])
}

fn navigation_sidebar_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::NavigationSidebarInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("section height", &metrics.section_height),
        ("item height", &metrics.item_height),
        ("item padding x", &metrics.item_padding_x),
        ("item gap", &metrics.item_gap),
        ("item radius", &metrics.item_radius),
        ("item icon size", &metrics.item_icon_size),
    ])
}

fn autocomplete_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::AutocompleteInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    [textfield_metric_properties(&metrics.textfield), floating_menu_metric_properties(&metrics.menu)].concat()
}

fn split_view_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::SplitViewInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("separator hitbox width", &metrics.separator_hitbox_width),
        ("separator cue width", &metrics.separator_cue_width),
        ("separator cue hovered width", &metrics.separator_cue_hovered_width),
        ("separator cue radius", &metrics.separator_cue_radius),
        ("separator cue inset y", &metrics.separator_cue_inset_y),
    ])
}

pub(in crate::gallery) fn split_view_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
) -> Vec<InspectMetricPropertyData> {
    split_view_metric_properties(&ShadcnInspect::new(look).inspect_split_view_metrics())
}

pub(in crate::gallery) fn resizable_panels_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    handle_size: gpui_luma::controls::resizable_panels::ResizeHandleSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_resizable_panels_metrics(handle_size);
    super::types::InspectLayoutSizeData {
        size: gpui_luma::theme::ControlSize::Md,
        properties: resizable_panels_metric_properties(&metrics),
    }
}

fn resizable_panels_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ResizablePanelsInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("lane", &metrics.lane_px),
        ("hit target", &metrics.hit_target_px),
        ("grip cross axis", &metrics.grip_cross_axis_px),
        ("grip main axis", &metrics.grip_main_axis_px),
    ])
}

pub(in crate::gallery) fn accordion_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_accordion_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: accordion_metric_properties(&metrics) }
}

fn accordion_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::AccordionInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("trigger height", &metrics.trigger_height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("content padding y", &metrics.content_padding_y),
        ("radius", &metrics.radius),
        ("inner gap", &metrics.inner_gap),
        ("icon size", &metrics.icon_size),
        ("chevron size", &metrics.chevron_size),
    ])
}

pub(in crate::gallery) fn list_view_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_list_view_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: list_view_metric_properties(&metrics) }
}

fn list_view_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ListViewInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("radius", &metrics.radius),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("row min height", &metrics.row_min_height),
        ("row padding x", &metrics.row_padding_x),
        ("row padding y", &metrics.row_padding_y),
    ])
}

pub(in crate::gallery) fn floating_menu_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_floating_menu_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: floating_menu_metric_properties(&metrics) }
}

pub(in crate::gallery) fn context_menu_target_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_context_menu_metrics(size);
    super::types::InspectLayoutSizeData { size, properties: context_menu_target_metric_properties(&metrics) }
}

pub(in crate::gallery) fn popup_menu_outline_trigger_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    popup_menu_trigger_layout_data(look, gpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Outline, size)
}

pub(in crate::gallery) fn popup_menu_ghost_trigger_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    popup_menu_trigger_layout_data(look, gpui_luma::controls::popup_menu::PopupMenuTriggerStyle::Ghost, size)
}

pub(in crate::gallery) fn popup_menu_trigger_layout_data(
    look: &gpui_luma_look_shadcn::ShadcnLook,
    trigger_style: gpui_luma::controls::popup_menu::PopupMenuTriggerStyle,
    size: gpui_luma::theme::ControlSize,
) -> super::types::InspectLayoutSizeData {
    let metrics = ShadcnInspect::new(look).inspect_popup_menu_metrics(trigger_style, size);
    super::types::InspectLayoutSizeData { size, properties: button_metric_properties(&metrics.trigger) }
}

fn floating_menu_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::FloatingMenuInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("radius", &metrics.radius),
        ("padding", &metrics.padding),
        ("min width", &metrics.min_width),
        ("item height", &metrics.item_height),
        ("item padding x", &metrics.item_padding_x),
        ("item gap", &metrics.item_gap),
        ("item icon size", &metrics.item_icon_size),
        ("item radius", &metrics.item_radius),
        ("submenu offset x", &metrics.submenu_offset_x),
    ])
}

fn context_menu_target_metric_properties(
    metrics: &gpui_luma_look_shadcn_inspect::ContextMenuInspectMetrics,
) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("target padding x", &metrics.target_padding_x),
        ("target padding y", &metrics.target_padding_y),
        ("target radius", &metrics.target_radius),
        ("target min width", &metrics.target_min_width),
    ])
}

fn button_metric_properties(metrics: &ButtonInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("height", &metrics.height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
        ("radius", &metrics.radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ])
}

fn checkbox_metric_properties(metrics: &CheckboxInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("height", &metrics.height),
        ("gap", &metrics.gap),
        ("indicator size", &metrics.indicator_size),
        ("indicator radius", &metrics.indicator_radius),
        ("glyph size", &metrics.glyph_size),
        ("control radius", &metrics.control_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ])
}

fn radio_metric_properties(metrics: &RadioButtonInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("height", &metrics.height),
        ("gap", &metrics.gap),
        ("indicator size", &metrics.indicator_size),
        ("dot size", &metrics.dot_size),
        ("control radius", &metrics.control_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ])
}

fn switch_metric_properties(metrics: &SwitchInspectMetrics) -> Vec<InspectMetricPropertyData> {
    metric_properties(&[
        ("track width", &metrics.track_width),
        ("track height", &metrics.track_height),
        ("track padding", &metrics.track_padding),
        ("thumb size", &metrics.thumb_size),
        ("gap", &metrics.gap),
        ("track radius", &metrics.track_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ])
}
