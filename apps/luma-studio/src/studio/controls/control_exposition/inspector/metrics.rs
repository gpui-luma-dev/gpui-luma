use gpui::SharedString;
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{
    AutocompleteInspectMetrics, BadgeInspectMetrics, ButtonInspectMetrics, CheckboxInspectMetrics,
    ContextMenuInspectMetrics, FloatingMenuInspectMetrics, PopupMenuInspectMetrics, ProgressInspectMetrics,
    RadioButtonInspectMetrics, ScrollbarInspectMetrics, ShadcnInspect, SliderInspectMetrics, SwitchInspectMetrics,
    TextFieldInspectMetrics,
};

use super::box_model::InspectBoxModelSnapshot;
use super::common::{control_size, neutral_box_model_colors, neutral_box_model_label_color, primary_secondary_style};
use super::occupation::{
    button_family_occupation, checkbox_occupation, occupation_metric_properties, radio_occupation, switch_occupation,
};
use super::provenance::{metric_properties, property_rows_from_metric_data};
use super::schema::InspectLayoutSection;

pub fn button_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
) -> InspectLayoutSection {
    let metrics = ShadcnInspect::new(look).inspect_button_metrics(style, role, size, state);
    button_metrics_layout_section(look, diagram_id, style, role, size, state, &metrics)
}

pub fn toggle_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let style = primary_secondary_style(variant_id);
    let size = control_size(size_id);
    button_layout_section(
        look,
        diagram_id,
        style,
        ButtonFamilyRole::Toggle { selected: true },
        size,
        InteractionState::default(),
    )
}

pub fn checkbox_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_checkbox_metrics(size);
    checkbox_metrics_layout_section(look, diagram_id, &metrics)
}

pub fn radio_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_radio_button_metrics(size);
    radio_metrics_layout_section(look, diagram_id, &metrics)
}

pub fn switch_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_switch_metrics(size);
    switch_metrics_layout_section(look, diagram_id, &metrics)
}

pub fn textfield_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_textfield_metrics(size))
}

pub fn textarea_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_textarea_metrics(size))
}

pub fn selector_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_selector_metrics(size);
    selector_metrics_layout_section(look, diagram_id, &metrics)
}

pub fn textfield_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    textfield_and_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_textfield_metrics(size),
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
}

pub fn autocomplete_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    autocomplete_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_autocomplete_metrics(size))
}

pub fn badge_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let variant = super::common::badge_variant(variant_id);
    badge_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_badge_metrics(variant, size))
}

pub fn progress_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    progress_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_progress_metrics())
}

pub fn slider_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    slider_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_slider_metrics())
}

pub fn scrollbar_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    let orientation = super::common::scrollbar_orientation(variant_id);
    scrollbar_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_scrollbar_metrics(orientation))
}

pub fn floating_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    floating_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
}

pub fn popup_menu_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let trigger_style = super::common::popup_menu_trigger_style(variant_id);
    popup_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_popup_menu_metrics(trigger_style, size),
    )
}

pub fn context_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    context_menu_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_context_menu_metrics(size))
}

fn button_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    style: ShadcnButtonStyle,
    role: ButtonFamilyRole,
    size: ControlSize,
    state: InteractionState,
    metrics: &ButtonInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_button_metrics(metrics);
    let occupation = button_family_occupation(look, style, role, size, state, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("height", &metrics.height),
        ("padding x", &metrics.padding_x),
        ("padding y", &metrics.padding_y),
        ("gap", &metrics.gap),
        ("radius", &metrics.radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}

fn checkbox_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &CheckboxInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_checkbox_metrics(metrics);
    let occupation = checkbox_occupation(look, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("height", &metrics.height),
        ("gap", &metrics.gap),
        ("indicator size", &metrics.indicator_size),
        ("indicator radius", &metrics.indicator_radius),
        ("glyph size", &metrics.glyph_size),
        ("control radius", &metrics.control_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}

fn radio_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &RadioButtonInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_radio_metrics(metrics);
    let occupation = radio_occupation(look, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("height", &metrics.height),
        ("gap", &metrics.gap),
        ("indicator size", &metrics.indicator_size),
        ("dot size", &metrics.dot_size),
        ("control radius", &metrics.control_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}

fn switch_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SwitchInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_switch_metrics(metrics);
    let occupation = switch_occupation(look, metrics.focus_ring_offset.value_px);
    let mut rows = metric_properties(&[
        ("track width", &metrics.track_width),
        ("track height", &metrics.track_height),
        ("track padding", &metrics.track_padding),
        ("thumb size", &metrics.thumb_size),
        ("gap", &metrics.gap),
        ("track radius", &metrics.track_radius),
        ("border width", &metrics.border_width),
        ("focus ring width", &metrics.focus_ring_width),
        ("focus ring offset", &metrics.focus_ring_offset),
    ]);
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}

fn textfield_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TextFieldInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_textfield_metrics(metrics);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("min height", &metrics.min_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("radius", &metrics.radius),
            ("border width", &metrics.border_width),
            ("focus ring width", &metrics.focus_ring_width),
            ("focus ring offset", &metrics.focus_ring_offset),
        ]),
    )
}

fn selector_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &gpui_luma_look_shadcn_inspect::SelectorInspectMetrics,
) -> InspectLayoutSection {
    let trigger_box = InspectBoxModelSnapshot::from_button_metrics(&metrics.trigger);
    let rows = metric_properties(&[
        ("trigger height", &metrics.trigger.height),
        ("trigger padding x", &metrics.trigger.padding_x),
        ("trigger padding y", &metrics.trigger.padding_y),
        ("trigger radius", &metrics.trigger.radius),
        ("trigger border width", &metrics.trigger.border_width),
        ("trigger focus ring width", &metrics.trigger.focus_ring_width),
        ("trigger focus ring offset", &metrics.trigger.focus_ring_offset),
        ("menu radius", &metrics.items_panel.radius),
        ("menu padding", &metrics.items_panel.padding),
        ("menu min width", &metrics.items_panel.min_width),
        ("menu item height", &metrics.items_panel.item_height),
        ("menu item padding x", &metrics.items_panel.item_padding_x),
        ("menu item gap", &metrics.items_panel.item_gap),
        ("menu item icon size", &metrics.items_panel.item_icon_size),
        ("menu item radius", &metrics.items_panel.item_radius),
        ("menu submenu offset x", &metrics.items_panel.submenu_offset_x),
    ]);
    layout_section(look, diagram_id, trigger_box, None, rows)
}

fn textfield_and_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    textfield: &TextFieldInspectMetrics,
    menu: &FloatingMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_textfield_metrics(textfield);
    let rows = metric_properties(&[
        ("textfield min height", &textfield.min_height),
        ("textfield padding x", &textfield.padding_x),
        ("textfield padding y", &textfield.padding_y),
        ("textfield radius", &textfield.radius),
        ("textfield border width", &textfield.border_width),
        ("textfield focus ring width", &textfield.focus_ring_width),
        ("textfield focus ring offset", &textfield.focus_ring_offset),
        ("menu radius", &menu.radius),
        ("menu padding", &menu.padding),
        ("menu min width", &menu.min_width),
        ("menu item height", &menu.item_height),
        ("menu item padding x", &menu.item_padding_x),
        ("menu item gap", &menu.item_gap),
        ("menu item icon size", &menu.item_icon_size),
        ("menu item radius", &menu.item_radius),
        ("menu submenu offset x", &menu.submenu_offset_x),
    ]);
    layout_section(look, diagram_id, box_model, None, rows)
}

fn autocomplete_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &AutocompleteInspectMetrics,
) -> InspectLayoutSection {
    textfield_and_menu_metrics_layout_section(look, diagram_id, &metrics.textfield, &metrics.menu)
}

fn badge_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &BadgeInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_badge_metrics(metrics);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("min height", &metrics.min_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("gap", &metrics.gap),
            ("icon size", &metrics.icon_size),
            ("radius", &metrics.radius),
        ]),
    )
}

fn progress_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ProgressInspectMetrics,
) -> InspectLayoutSection {
    let size = metrics.size.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: size,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: metrics.stroke_width.value_px,
        gap: 0.0,
        radius: size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("size", &metrics.size), ("stroke width", &metrics.stroke_width)]),
    )
}

fn slider_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SliderInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.height.value_px,
        padding_x: 0.0,
        padding_y: ((metrics.height.value_px - metrics.track_height.value_px) / 2.0).max(0.0),
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("width", &metrics.width),
            ("height", &metrics.height),
            ("track height", &metrics.track_height),
            ("thumb size", &metrics.thumb_size),
            ("radius", &metrics.radius),
        ]),
    )
}

fn scrollbar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ScrollbarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.thumb_thickness.value_px,
        padding_x: ((metrics.track_thickness.value_px - metrics.thumb_thickness.value_px) / 2.0).max(0.0),
        padding_y: 0.0,
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("length", &metrics.length),
            ("thickness", &metrics.thickness),
            ("track thickness", &metrics.track_thickness),
            ("thumb thickness", &metrics.thumb_thickness),
            ("min thumb length", &metrics.min_thumb_length),
            ("radius", &metrics.radius),
        ]),
    )
}

fn floating_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &FloatingMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: metrics.padding.value_px,
        border_width: 0.0,
        gap: metrics.item_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(look, diagram_id, box_model, None, floating_menu_metric_rows(metrics))
}

fn popup_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &PopupMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_button_metrics(&metrics.trigger);
    let mut rows = metric_properties(&[
        ("trigger height", &metrics.trigger.height),
        ("trigger padding x", &metrics.trigger.padding_x),
        ("trigger padding y", &metrics.trigger.padding_y),
        ("trigger radius", &metrics.trigger.radius),
        ("trigger border width", &metrics.trigger.border_width),
        ("trigger focus ring width", &metrics.trigger.focus_ring_width),
        ("trigger focus ring offset", &metrics.trigger.focus_ring_offset),
    ]);
    rows.extend(floating_menu_metric_rows(&metrics.menu));
    layout_section(look, diagram_id, box_model, None, rows)
}

fn context_menu_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ContextMenuInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.target_padding_y.value_px * 2.0 + 16.0,
        padding_x: metrics.target_padding_x.value_px,
        padding_y: metrics.target_padding_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.target_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    let mut rows = metric_properties(&[
        ("target padding x", &metrics.target_padding_x),
        ("target padding y", &metrics.target_padding_y),
        ("target radius", &metrics.target_radius),
        ("target min width", &metrics.target_min_width),
    ]);
    rows.extend(floating_menu_metric_rows(&metrics.menu));
    layout_section(look, diagram_id, box_model, None, rows)
}

fn floating_menu_metric_rows(metrics: &FloatingMenuInspectMetrics) -> Vec<super::schema::InspectPropertyRow> {
    metric_properties(&[
        ("menu radius", &metrics.radius),
        ("menu padding", &metrics.padding),
        ("menu min width", &metrics.min_width),
        ("menu item height", &metrics.item_height),
        ("menu item padding x", &metrics.item_padding_x),
        ("menu item gap", &metrics.item_gap),
        ("menu item icon size", &metrics.item_icon_size),
        ("menu item radius", &metrics.item_radius),
        ("menu submenu offset x", &metrics.submenu_offset_x),
    ])
}

fn layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    box_model: InspectBoxModelSnapshot,
    occupation: Option<super::box_model::InspectOccupationSnapshot>,
    metrics: Vec<super::schema::InspectPropertyRow>,
) -> InspectLayoutSection {
    InspectLayoutSection {
        box_model_diagram_id: SharedString::from(diagram_id),
        box_model,
        occupation,
        box_model_colors: neutral_box_model_colors(look.mode()),
        box_model_label_color: neutral_box_model_label_color(look.mode()),
        metrics,
    }
}
