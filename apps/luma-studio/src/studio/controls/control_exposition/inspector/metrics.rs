use gpui::SharedString;
use gpui_luma::controls::button_family::ButtonFamilyRole;
use gpui_luma::controls::scrollbar::ScrollbarOrientation;
use gpui_luma::theme::{ControlSize, InteractionState};
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use gpui_luma_look_shadcn_inspect::{
    AccordionInspectMetrics, AutocompleteInspectMetrics, BadgeInspectMetrics, ButtonInspectMetrics,
    CheckboxInspectMetrics, ContextMenuInspectMetrics, FloatingMenuInspectMetrics, ListBoxInspectMetrics,
    ListViewInspectMetrics, SidebarInspectMetrics, OverlayWindowInspectMetrics, ProgressInspectMetrics,
    RadioButtonInspectMetrics, ResizablePanelsInspectMetrics, ScrollbarInspectMetrics, ShadcnInspect,
    SliderInspectMetrics, SplitViewInspectMetrics, StepperInspectMetrics, SwitchInspectMetrics,
    TabsNavigationInspectMetrics, TextFieldInspectMetrics, PagerInspectMetrics, ToolbarInspectMetrics,
    TreeViewInspectMetrics,
};

use super::box_model::InspectBoxModelSnapshot;
use super::common::{choice_variant_style, control_size, neutral_box_model_colors, neutral_box_model_label_color};
use super::occupation::{
    button_family_occupation, checkbox_occupation, occupation_metric_properties, radio_occupation, switch_occupation,
};
use super::provenance::{metric_properties, property_rows_from_metric_data};
use super::schema::{InspectLayoutSection, InspectPropertyRow};

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
    let style = choice_variant_style(variant_id);
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

pub fn checkbox_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    size_id: &str,
    indicator_only: bool,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_checkbox_metrics(size);
    checkbox_metrics_layout_section(look, diagram_id, &metrics, indicator_only)
}

pub fn radio_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    size_id: &str,
    indicator_only: bool,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_radio_button_metrics(size);
    radio_metrics_layout_section(look, diagram_id, &metrics, indicator_only)
}

pub fn switch_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let metrics = ShadcnInspect::new(look).inspect_switch_metrics_for_style(choice_variant_style(variant_id), size);
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

pub fn progress_circular_layout_section(
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

pub fn progress_linear_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ProgressInspectMetrics,
) -> InspectLayoutSection {
    let track_height = metrics.track_height.value_px;
    let thumb_size = metrics.thumb_size.value_px;
    let cross_extent = thumb_size.max(track_height);
    let track_inset = (cross_extent - track_height) / 2.0;

    let box_model = InspectBoxModelSnapshot {
        height: cross_extent,
        padding_x: 0.0,
        padding_y: track_inset,
        border_width: track_height,
        gap: 0.0,
        radius: track_height / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("track height", &metrics.track_height), ("thumb size", &metrics.thumb_size)]),
    )
}

pub fn stepper_horizontal_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &StepperInspectMetrics,
) -> InspectLayoutSection {
    let badge_size = metrics.step_badge_size.value_px;
    let track_thickness = metrics.track_thickness.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: badge_size,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: track_thickness,
        gap: 0.0,
        radius: badge_size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("badge size", &metrics.step_badge_size), ("track thickness", &metrics.track_thickness)]),
    )
}

pub fn stepper_vertical_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &StepperInspectMetrics,
) -> InspectLayoutSection {
    let badge_size = metrics.step_badge_size.value_px;
    let track_thickness = metrics.track_thickness.value_px;
    let box_model = InspectBoxModelSnapshot {
        height: badge_size * 2.0 + 24.0,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: track_thickness,
        gap: 24.0,
        radius: badge_size / 2.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[("badge size", &metrics.step_badge_size), ("track thickness", &metrics.track_thickness)]),
    )
}

pub fn slider_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    slider_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_slider_metrics())
}

pub fn scrollbar_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    let style = super::common::scrollbar_style(variant_id);
    let orientation = ScrollbarOrientation::Vertical;
    scrollbar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_scrollbar_metrics(orientation, style),
    )
}

pub fn floating_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    floating_menu_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_floating_menu_metrics(size),
    )
}

pub fn overlay_window_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    use gpui_luma::controls::overlay_window::OverlayWindowMode;

    let size = control_size(size_id);
    let mode = match variant_id {
        "modal" => OverlayWindowMode::Modal,
        _ => OverlayWindowMode::Modeless,
    };
    overlay_window_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_overlay_window_metrics(size, mode),
    )
}

pub fn popup_menu_trigger_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    let size = control_size(size_id);
    let trigger_style = super::common::popup_menu_trigger_style(variant_id);
    let metrics = ShadcnInspect::new(look).inspect_popup_menu_metrics(trigger_style, size);
    let box_model = InspectBoxModelSnapshot::from_button_metrics(&metrics.trigger);
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("trigger height", &metrics.trigger.height),
            ("trigger padding x", &metrics.trigger.padding_x),
            ("trigger padding y", &metrics.trigger.padding_y),
            ("trigger radius", &metrics.trigger.radius),
            ("trigger border width", &metrics.trigger.border_width),
            ("trigger focus ring width", &metrics.trigger.focus_ring_width),
            ("trigger focus ring offset", &metrics.trigger.focus_ring_offset),
        ]),
    )
}

pub fn context_menu_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    let size = control_size(size_id);
    context_menu_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_context_menu_metrics(size))
}

pub fn listbox_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    listbox_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_listbox_metrics(control_size(size_id)),
    )
}

pub fn list_view_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    variant_id: &str,
    size_id: &str,
) -> InspectLayoutSection {
    if variant_id == "grid-cell" {
        return list_view_grid_cell_layout_section(look, diagram_id, control_size(size_id));
    }
    list_view_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_list_view_metrics(control_size(size_id)),
    )
}

pub fn tree_view_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    tree_view_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_tree_view_metrics(control_size(size_id)),
    )
}

pub fn sidebar_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    sidebar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_sidebar_metrics(control_size(size_id)),
    )
}

pub fn tabs_navigation_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    tabs_navigation_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_tabs_navigation_metrics(control_size(size_id)),
    )
}

pub fn toolbar_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    toolbar_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_toolbar_metrics(control_size(size_id)),
    )
}

pub fn pager_shell_layout_section(look: &ShadcnLook, diagram_id: &str, variant_id: &str) -> InspectLayoutSection {
    let style = super::common::pager_style(variant_id);
    pager_shell_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_pager_metrics(style))
}

pub fn pager_button_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    let style = gpui_luma::controls::pager::PagerStyle::Numeric;
    pager_button_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_pager_metrics(style))
}

pub fn accordion_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    accordion_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_accordion_metrics(control_size(size_id)),
    )
}

pub fn resizable_panels_layout_section(look: &ShadcnLook, diagram_id: &str, size_id: &str) -> InspectLayoutSection {
    resizable_panels_metrics_layout_section(
        look,
        diagram_id,
        &ShadcnInspect::new(look).inspect_resizable_panels_metrics(super::common::resize_handle_size(size_id)),
    )
}

pub fn split_view_layout_section(look: &ShadcnLook, diagram_id: &str) -> InspectLayoutSection {
    split_view_metrics_layout_section(look, diagram_id, &ShadcnInspect::new(look).inspect_split_view_metrics())
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
    indicator_only: bool,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_checkbox_metrics(metrics, indicator_only);
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
    if indicator_only {
        rows.push(InspectPropertyRow::new("template path", "indicator_only (no min_h, no py)", "CheckboxTemplate"));
    } else {
        rows.push(InspectPropertyRow::new(
            "padding y (derived)",
            format!("{:.1}px", box_model.padding_y),
            "=(control_height − indicator_size) / 2 · centers indicator in min_h",
        ));
    }
    rows.extend(property_rows_from_metric_data(&occupation_metric_properties(box_model.height, &occupation)));
    layout_section(look, diagram_id, box_model, Some(occupation), rows)
}

fn radio_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &RadioButtonInspectMetrics,
    indicator_only: bool,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot::from_radio_metrics(metrics, indicator_only);
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
    if indicator_only {
        rows.push(InspectPropertyRow::new("template path", "indicator_only (no min_h, no py)", "RadioButtonTemplate"));
    } else {
        rows.push(InspectPropertyRow::new(
            "padding y (derived)",
            format!("{:.1}px", box_model.padding_y),
            "=(control_height − indicator_size) / 2 · centers indicator in min_h",
        ));
    }
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
        ("trigger icon size", &metrics.trigger.icon_size),
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
        ("textfield icon size", &textfield.icon_size),
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

fn overlay_window_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &OverlayWindowInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.estimated_height.value_px,
        padding_x: metrics.padding.value_px,
        padding_y: metrics.padding.value_px,
        border_width: 1.0,
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
            ("min width", &metrics.min_width),
            ("max width", &metrics.max_width),
            ("estimated height", &metrics.estimated_height),
            ("radius", &metrics.radius),
            ("padding", &metrics.padding),
        ]),
    )
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

fn listbox_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ListBoxInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.row_min_height.value_px,
        padding_x: metrics.row_padding_x.value_px,
        padding_y: metrics.row_padding_y.value_px,
        border_width: 0.0,
        gap: metrics.row_gap.value_px,
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
            ("radius", &metrics.radius),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("row gap", &metrics.row_gap),
            ("row min height", &metrics.row_min_height),
            ("row padding x", &metrics.row_padding_x),
            ("row padding y", &metrics.row_padding_y),
        ]),
    )
}

fn list_view_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ListViewInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.row_min_height.value_px,
        padding_x: metrics.row_padding_x.value_px,
        padding_y: metrics.row_padding_y.value_px,
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
            ("radius", &metrics.radius),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("row min height", &metrics.row_min_height),
            ("row padding x", &metrics.row_padding_x),
            ("row padding y", &metrics.row_padding_y),
        ]),
    )
}

fn compact_table_row_height(size: ControlSize) -> f32 {
    match size {
        ControlSize::Sm => 28.0,
        ControlSize::Md => 32.0,
        ControlSize::Lg => 36.0,
    }
}

fn list_view_grid_cell_layout_section(look: &ShadcnLook, diagram_id: &str, size: ControlSize) -> InspectLayoutSection {
    const GRID_CONTROL_COLUMN_WIDTH: f32 = 48.0;
    let row_metrics = ShadcnInspect::new(look).inspect_list_view_metrics(size);
    let checkbox_metrics = ShadcnInspect::new(look).inspect_checkbox_metrics(ControlSize::Sm);
    let compact_row_height = compact_table_row_height(size);
    let inner_row_height = compact_row_height - (row_metrics.row_padding_y.value_px * 2.0);

    let box_model = InspectBoxModelSnapshot {
        height: compact_row_height,
        padding_x: 0.0,
        padding_y: row_metrics.row_padding_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: row_metrics.radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };

    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        vec![
            InspectPropertyRow::new(
                "grid cell width",
                format!("{GRID_CONTROL_COLUMN_WIDTH:.0}px"),
                "ListViewColumn::fixed_control",
            ),
            InspectPropertyRow::new(
                "compact row height",
                format!("{compact_row_height:.0}px"),
                "visible_row_height override (data-table rows)",
            ),
            InspectPropertyRow::new(
                "row padding y",
                format!("{:.1}px", row_metrics.row_padding_y.value_px),
                "applied inside fixed row height",
            ),
            InspectPropertyRow::new(
                "inner content height",
                format!("{inner_row_height:.1}px"),
                "compact row height − (2 × row padding y)",
            ),
            InspectPropertyRow::new("embedded checkbox size", "sm · indicator_only", "payments card pattern"),
            InspectPropertyRow::new(
                "checkbox indicator size",
                format!("{:.1}px", checkbox_metrics.indicator_size.value_px),
                "CheckboxScale at Sm",
            ),
        ],
    )
}

fn tree_view_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TreeViewInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.row_height.value_px,
        padding_x: metrics.base_padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.inner_gap.value_px,
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
            ("row height", &metrics.row_height),
            ("base padding x", &metrics.base_padding_x),
            ("indentation width", &metrics.indentation_width),
            ("inner gap", &metrics.inner_gap),
            ("radius", &metrics.radius),
            ("icon size", &metrics.icon_size),
            ("chevron size", &metrics.chevron_size),
        ]),
    )
}

fn sidebar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SidebarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.item_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("section height", &metrics.section_height),
            ("item height", &metrics.item_height),
            ("item padding x", &metrics.item_padding_x),
            ("item gap", &metrics.item_gap),
            ("item radius", &metrics.item_radius),
            ("item icon size", &metrics.item_icon_size),
        ]),
    )
}

fn tabs_navigation_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &TabsNavigationInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.item_height.value_px,
        padding_x: metrics.item_padding_x.value_px,
        padding_y: metrics.list_padding.value_px,
        border_width: 0.0,
        gap: metrics.list_gap.value_px,
        radius: metrics.item_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("list radius", &metrics.list_radius),
            ("list gap", &metrics.list_gap),
            ("list padding", &metrics.list_padding),
            ("item padding x", &metrics.item_padding_x),
            ("item height", &metrics.item_height),
            ("item radius", &metrics.item_radius),
            ("indicator height", &metrics.indicator_height),
        ]),
    )
}

fn toolbar_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ToolbarInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.separator_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.gap.value_px,
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
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("gap", &metrics.gap),
            ("radius", &metrics.radius),
            ("separator height", &metrics.separator_height),
        ]),
    )
}

fn pager_shell_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &PagerInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.control_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.group_gap.value_px,
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
            ("control height", &metrics.control_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("item gap", &metrics.gap),
            ("group gap", &metrics.group_gap),
            ("radius", &metrics.radius),
        ]),
    )
}

fn pager_button_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &PagerInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.button_size.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: 0.0,
        border_width: 0.0,
        gap: metrics.gap.value_px,
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
            ("button size", &metrics.button_size),
            ("button min width", &metrics.button_min_width),
            ("padding x", &metrics.padding_x),
            ("gap", &metrics.gap),
            ("radius", &metrics.radius),
        ]),
    )
}

fn accordion_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &AccordionInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.trigger_height.value_px,
        padding_x: metrics.padding_x.value_px,
        padding_y: metrics.padding_y.value_px,
        border_width: 0.0,
        gap: metrics.inner_gap.value_px,
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
            ("trigger height", &metrics.trigger_height),
            ("padding x", &metrics.padding_x),
            ("padding y", &metrics.padding_y),
            ("content padding y", &metrics.content_padding_y),
            ("radius", &metrics.radius),
            ("inner gap", &metrics.inner_gap),
            ("icon size", &metrics.icon_size),
            ("chevron size", &metrics.chevron_size),
        ]),
    )
}

fn resizable_panels_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &ResizablePanelsInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.hit_target_px.value_px,
        padding_x: 0.0,
        padding_y: 0.0,
        border_width: 0.0,
        gap: 0.0,
        radius: 0.0,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("lane", &metrics.lane_px),
            ("hit target", &metrics.hit_target_px),
            ("grip cross axis", &metrics.grip_cross_axis_px),
            ("grip main axis", &metrics.grip_main_axis_px),
        ]),
    )
}

fn split_view_metrics_layout_section(
    look: &ShadcnLook,
    diagram_id: &str,
    metrics: &SplitViewInspectMetrics,
) -> InspectLayoutSection {
    let box_model = InspectBoxModelSnapshot {
        height: metrics.separator_hitbox_width.value_px,
        padding_x: 0.0,
        padding_y: metrics.separator_cue_inset_y.value_px,
        border_width: 0.0,
        gap: 0.0,
        radius: metrics.separator_cue_radius.value_px,
        focus_ring_width: 0.0,
        focus_ring_offset: 0.0,
    };
    layout_section(
        look,
        diagram_id,
        box_model,
        None,
        metric_properties(&[
            ("separator hitbox width", &metrics.separator_hitbox_width),
            ("separator cue width", &metrics.separator_cue_width),
            ("separator cue hovered width", &metrics.separator_cue_hovered_width),
            ("separator cue radius", &metrics.separator_cue_radius),
            ("separator cue inset y", &metrics.separator_cue_inset_y),
        ]),
    )
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
