use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, DragMoveEvent, FontWeight, IntoElement, KeyDownEvent, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Render, ScrollWheelEvent, SharedString, TextRun, Window, div, font,
    prelude::*, px, svg,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize, default_button_family_theme};
use gpui_luma::controls::command::button::{
    ButtonRenderModel, ButtonTemplate, DefaultButtonTemplate, default_button_template,
};
use gpui_luma::controls::autocomplete::{
    AutocompleteItemsRenderModel, AutocompleteItemsTemplateHandlers, AutocompleteTextBoxRenderModel,
    AutocompleteTextBoxTemplateHandlers, default_autocomplete_items_template, default_autocomplete_textbox_template,
};
use gpui_luma::controls::combobox::{
    ComboBoxItemsRenderModel, ComboBoxItemsTemplate, ComboBoxItemsTemplateHandlers, ComboBoxPanelRenderModel,
    ComboBoxPanelTemplate, ComboBoxRenderModel, ComboBoxTemplateHandlers, SelectionItem as ComboBoxSelectionItem,
    default_combobox_items_template, default_combobox_panel_template, default_combobox_template,
};
use gpui_luma::controls::floating_menu::{
    FloatingMenuClickHandler, FloatingMenuHoverHandler, FloatingMenuLook, FloatingMenuState, FloatingMenuStepDirection,
    render_floating_menu,
};
use gpui_luma::controls::menu_item::MenuItem;
use gpui_luma::controls::popup_menu::{
    ControlFocusState as PopupMenuControlFocusState, PopupMenuPlacement, PopupMenuRenderModel, PopupMenuTemplate,
    PopupMenuTemplateHandlers, PopupMenuTriggerStyle,
};
use gpui_luma::controls::search_selector::{
    SearchSelectorItemsRenderModel, SearchSelectorItemsTemplate, SearchSelectorItemsTemplateHandlers,
    SearchSelectorPanelRenderModel, SearchSelectorPanelTemplate, SearchSelectorRenderModel,
    SearchSelectorTemplateHandlers, SelectionItem as SearchSelectorSelectionItem,
    default_search_selector_items_template, default_search_selector_panel_template, default_search_selector_template,
};
use gpui_luma::controls::selector::{
    ControlFocusState, SelectorItem, SelectorPath, SelectorPlacement, SelectorRenderModel, SelectorTemplateHandlers,
};
use gpui_luma::controls::selector_panel::{
    SelectorItem as SelectorPanelItem, SelectorItemsPanelLook, SelectorItemsRenderModel, SelectorItemsTemplateHandlers,
    SelectorPanelClickHandler, SelectorPanelHoverHandler, default_selector_items_template,
};
use gpui_luma::controls::scrollbar::{
    ScrollbarBoundsHandler, ScrollbarDrag, ScrollbarDragMoveHandler, ScrollbarHoverHandler, ScrollbarMouseDownHandler,
    ScrollbarMouseUpHandler, ScrollbarOrientation, ScrollbarRenderModel, ScrollbarScrollWheelHandler,
    ScrollbarTemplate, ScrollbarTemplateHandlers,
};
use gpui_luma::controls::state::MenuPath;
use gpui_luma::controls::slider::{
    SliderBoundsHandler, SliderDrag, SliderHoverHandler, SliderInputStrategy, SliderMouseDownHandler,
    SliderMouseMoveHandler, SliderMouseUpHandler, SliderRenderModel, SliderTemplate, SliderTemplateHandlers,
    SliderThumbPolicy, SliderThumbRole, SliderThumbValue, ThumbId, TrackPresentation, build_track_segments,
};
use gpui_luma::controls::tabs_navigation::{
    ControlFocusState as TabsControlFocusState, TabsNavigationClickHandler, TabsNavigationHoverHandler,
    TabsNavigationItem, TabsNavigationItemState, TabsNavigationMouseDownHandler, TabsNavigationMouseUpHandler,
    TabsNavigationRenderItem, TabsNavigationRenderModel, TabsNavigationTemplate, TabsNavigationTemplateHandlers,
    TabsNavigationWidthMode,
};
use gpui_luma::controls::textarea::{
    TextAreaClickHandler, TextAreaDrag, TextAreaHoverHandler, TextAreaKeyDownHandler, TextAreaLineMetric,
    TextAreaMouseDownHandler, TextAreaMouseMoveHandler, TextAreaMouseUpHandler, TextAreaRenderModel, TextAreaState,
    TextAreaTemplate, TextAreaTemplateHandlers, TextAreaTheme, ThemedTextAreaTemplate,
};
use gpui_luma::controls::textfield::{
    TextFieldClickHandler, TextFieldHoverHandler, TextFieldKeyDownHandler, TextFieldMouseDownHandler,
    TextFieldMouseMoveHandler, TextFieldMouseUpHandler, TextFieldRenderModel, TextFieldState, TextFieldTemplate,
    TextFieldTemplateHandlers, TextFieldTheme, TextFieldVariant,
};
use gpui_luma::controls::value::ControlRange;
use gpui_luma::theme::{ControlSize, InteractionState, StandardBoxScale};
use gpui_luma::{declare_form, hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;
use self::cards::buttons::{render_lucide_icon, round_icon_glyph};

#[path = "cards/mod.rs"]
mod cards;

const STYLE_GUIDE_DESCRIPTION: &str = concat!(
    "Theme Studio style guide surface. ",
    "Starts with Gallery template previews, then keeps the existing typography references."
);

declare_form! {
    pub struct StyleGuidePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {}
    }
}

impl StyleGuidePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        cx.notify();
    }
}

impl Render for StyleGuidePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        with_look(&self.look, || {
            let chrome = self.look.chrome();

            div()
                .id("theme-studio-typography")
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(chrome.content_background)
                .p(px(28.0))
                .child(
                    div()
                        .w_full()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_size(px(20.0))
                                .line_height(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(chrome.title_text)
                                .child("Style Guide"),
                        )
                        .child(
                            div()
                                .max_w(px(760.0))
                                .text_size(px(13.0))
                                .line_height(px(18.0))
                                .text_color(chrome.muted_text)
                                .child(STYLE_GUIDE_DESCRIPTION),
                        ),
                )
                .child(
                    div().w_full().min_h(px(0.0)).flex_1().child(
                        div()
                            .id("typography-content")
                            .size_full()
                            .flex()
                            .flex_col()
                            .gap(px(18.0))
                            .overflow_y_scroll()
                            .pt(px(18.0))
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_wrap()
                                    .items_start()
                                    .gap(px(20.0))
                                    .when_some(render_sparse_catalog_callout(self.look.as_ref()), |panel, callout| {
                                        panel.child(callout)
                                    })
                                    .children([
                                        cards::buttons::render_button_template_matrix_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::buttons::render_choice_template_matrix_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::buttons::render_toggle_template_matrix_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::menus::render_menu_template_state_section(self.look.clone(), window, cx),
                                        cards::selectors::render_selector_templates_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::menus::render_tabs_navigation_template_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::inputs::render_input_controls_template_section(
                                            self.look.clone(),
                                            window,
                                            cx,
                                        ),
                                        cards::typography::render_typography_section(self.look.as_ref()),
                                    ]),
                            ),
                    ),
                )
        })
    }
}

#[derive(Clone, Copy)]
struct ButtonStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
enum ButtonTemplateVariant {
    TextButton,
    TextButtonLeadingIcon,
    TextButtonTrailingIcon,
    IconButton,
}

#[derive(Clone, Copy)]
struct ChoiceTemplateStateSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
struct ToggleStateSample {
    id: &'static str,
    header: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
struct InputInteractionSample {
    id: &'static str,
    label: &'static str,
    state: InteractionState,
}

#[derive(Clone, Copy)]
struct InputTextFieldSample {
    id: &'static str,
    label: &'static str,
    state: TextFieldState,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct InputTextAreaSample {
    id: &'static str,
    label: &'static str,
    state: TextAreaState,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct PopupMenuStateSample {
    id: &'static str,
    label: &'static str,
    trigger_style: PopupMenuTriggerStyle,
    state: InteractionState,
    focus: PopupMenuControlFocusState,
}

#[derive(Clone, Copy)]
struct TabsNavigationStateSample {
    id: &'static str,
    label: &'static str,
    active_index: usize,
    target_index: usize,
    target_state: TabsNavigationItemState,
    enabled: bool,
}

#[derive(Clone, Copy)]
struct SelectorTemplateStateSample {
    id: &'static str,
    label: &'static str,
    textfield_state: TextFieldState,
    textfield_enabled: bool,
    selector_state: InteractionState,
    selector_focus: ControlFocusState,
    selector_enabled: bool,
}

#[derive(Clone, Copy)]
enum ChoiceTemplateControl {
    Radio,
    Checkbox,
    Switch,
    Toggle,
    ToggleIcon,
}

#[derive(Clone, Copy)]
enum ToggleTemplateVariant {
    TextUnselected,
    TextSelected,
    RoundIconUnselected,
    RoundIconSelected,
}

#[derive(Clone, Copy)]
enum SelectorTemplateControl {
    AutocompleteTextBox,
    ComboBox,
    Selector,
    SearchSelector,
}

impl ChoiceTemplateControl {
    fn id(self) -> &'static str {
        match self {
            Self::Radio => "radio",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Toggle => "toggle",
            Self::ToggleIcon => "toggle-icon",
        }
    }

    fn header(self) -> &'static str {
        match self {
            Self::Radio => "Radio",
            Self::Checkbox => "Checkbox",
            Self::Switch => "Switch",
            Self::Toggle => "Toggle",
            Self::ToggleIcon => "Toggle Icon",
        }
    }

    fn content(
        self,
        selected: bool,
    ) -> gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<bool>> {
        match self {
            Self::Radio => Arc::new(move |_, _| div().child("Radio").into_any_element()),
            Self::Checkbox => Arc::new(move |_, _| div().child("Checkbox").into_any_element()),
            Self::Switch => Arc::new(move |_, _| div().into_any_element()),
            Self::Toggle => Arc::new(move |_, _| div().child("Toggle").into_any_element()),
            Self::ToggleIcon => {
                let icon = if selected { LucideIcon::Check } else { LucideIcon::Plus };
                Arc::new(move |_, _| render_lucide_icon(icon))
            }
        }
    }

    fn round(self) -> bool {
        matches!(self, Self::ToggleIcon)
    }
}

impl ToggleTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::TextUnselected => "text-unselected",
            Self::TextSelected => "text-selected",
            Self::RoundIconUnselected => "round-icon-unselected",
            Self::RoundIconSelected => "round-icon-selected",
        }
    }

    fn selected(self) -> bool {
        matches!(self, Self::TextSelected | Self::RoundIconSelected)
    }

    fn round(self) -> bool {
        matches!(self, Self::RoundIconUnselected | Self::RoundIconSelected)
    }

    fn content(self) -> gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<bool>> {
        match self {
            Self::TextUnselected | Self::TextSelected => {
                let label = SharedString::from("Toggle");
                Arc::new(move |_, _| div().child(label.clone()).into_any_element())
            }
            Self::RoundIconUnselected => Arc::new(move |_, _| round_icon_glyph(false)),
            Self::RoundIconSelected => Arc::new(move |_, _| round_icon_glyph(true)),
        }
    }
}

impl SelectorTemplateControl {
    fn header(self) -> &'static str {
        match self {
            Self::AutocompleteTextBox => "AutocompleteTextBox",
            Self::ComboBox => "ComboBox",
            Self::Selector => "Selector",
            Self::SearchSelector => "SearchSelector",
        }
    }
}

impl ButtonTemplateVariant {
    fn id(self) -> &'static str {
        match self {
            Self::TextButton => "text-button",
            Self::TextButtonLeadingIcon => "text-button-leading-icon",
            Self::TextButtonTrailingIcon => "text-button-trailing-icon",
            Self::IconButton => "icon-button",
        }
    }

    fn round(self) -> bool {
        matches!(self, Self::IconButton)
    }

    fn content(self) -> gpui_luma::controls::command::button::ControlPresenter<ButtonRenderModel<()>> {
        let label = SharedString::from("Button");
        match self {
            Self::TextButton => Arc::new(move |_, _| div().child(label.clone()).into_any_element()),
            Self::TextButtonLeadingIcon => Arc::new(move |_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(render_lucide_icon(LucideIcon::Heart))
                    .child(label.clone())
                    .into_any_element()
            }),
            Self::TextButtonTrailingIcon => Arc::new(move |_, _| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .child(label.clone())
                    .child(render_lucide_icon(LucideIcon::ChevronDown))
                    .into_any_element()
            }),
            Self::IconButton => Arc::new(move |_, _| render_lucide_icon(LucideIcon::Heart)),
        }
    }
}

fn render_sparse_catalog_callout(look: &ShadcnLook) -> Option<AnyElement> {
    if look.has_css_catalog() {
        return None;
    }

    let chrome = look.chrome();
    let title = "Native default theme — sparse CSS catalog";
    let body = "This view uses the active look typography and token mappings. The native default theme has no CSS catalog, so cross-reference data is limited. SDK controls still resolve colors from the embedded palette. Pick a tweakcn theme in the sidebar for full catalog-backed typography context.";

    Some(render_callout(
        title,
        body,
        chrome.border,
        chrome.panel_background,
        chrome.title_text,
        chrome.body_text,
    ))
}

fn render_callout(
    title: &str,
    body: &str,
    border: gpui::Hsla,
    background: gpui::Hsla,
    title_color: gpui::Hsla,
    body_color: gpui::Hsla,
) -> AnyElement {
    div()
        .w_full()
        .max_w(px(860.0))
        .flex()
        .flex_col()
        .gap(px(6.0))
        .border_1()
        .border_color(border)
        .rounded(px(10.0))
        .bg(background)
        .p(px(14.0))
        .child(
            div()
                .typography_sm()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(title_color)
                .child(title.to_string()),
        )
        .child(div().typography_xs().text_color(body_color).child(body.to_string()))
        .into_any_element()
}

fn section_shell_with_width(
    width: f32,
    title: &'static str,
    description: &'static str,
    title_color: gpui::Hsla,
    muted_text: gpui::Hsla,
    border: gpui::Hsla,
    background: gpui::Hsla,
    content: AnyElement,
) -> AnyElement {
    div()
        .w(px(width))
        .max_w_full()
        .flex()
        .flex_col()
        .gap(px(14.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(border)
        .bg(background)
        .p(px(18.0))
        .child(
            vstack! {
                gap=4;
                div().text_h4().text_color(title_color).child(title),
                div().typography_sm().text_color(muted_text).child(description),
            }
            .into_any_element(),
        )
        .child(content)
        .into_any_element()
}

fn render_vertical_section_rail(label: &'static str, color: gpui::Hsla) -> AnyElement {
    div()
        .w(px(28.0))
        .min_h(px(188.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(svg().path(section_label_asset_path(label)).w(px(20.0)).h(px(104.0)).text_color(color))
        .child(div().w(px(1.0)).h_full().bg(color))
        .into_any_element()
}

fn render_vertical_state_rail(label: &'static str, state_id: &str, label_color: gpui::Hsla) -> AnyElement {
    div()
        .id(format!("theme-studio-choice-template-state-rail-{label}"))
        .w(px(28.0))
        .min_h(px(38.0))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(svg().path(state_label_asset_path(state_id)).w(px(20.0)).h(px(38.0)).text_color(label_color))
        .child(div().w(px(1.0)).h_full().bg(label_color))
        .into_any_element()
}

fn section_label_asset_path(section_label: &'static str) -> &'static str {
    match section_label {
        "Primary" | "Prominent" => "assets/labels/primary-label.svg",
        "Secondary" | "Standard" => "assets/labels/secondary-label.svg",
        "Outline" | "Subtle" => "assets/labels/outline-label.svg",
        "Ghost" => "assets/labels/ghost-label.svg",
        "Selected" => "assets/labels/selected-label.svg",
        "Unselected" => "assets/labels/unselected-label.svg",
        _ => "assets/labels/default-label.svg",
    }
}

fn state_label_asset_path(state_id: &str) -> &'static str {
    match state_id {
        "default" => "assets/labels/default-label.svg",
        "hover" => "assets/labels/hover-label.svg",
        "focused" => "assets/labels/focused-label.svg",
        "pressed" => "assets/labels/pressed-label.svg",
        "disabled" => "assets/labels/disabled-label.svg",
        _ => "assets/labels/default-label.svg",
    }
}
