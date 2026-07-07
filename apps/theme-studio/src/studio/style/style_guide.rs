use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Context, DragMoveEvent, Entity, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Render, ScrollHandle, ScrollWheelEvent,
    SharedString, TextRun, Window, div, font, point, prelude::*, px, svg,
};
use gpui_luma::controls::button_family::{ButtonFamilyRole, ButtonSize};
use gpui_luma::controls::color::style::ElementExt;
use gpui_luma::controls::command::button::{ButtonRenderModel, ButtonTemplate, default_button_template};
use gpui_luma::controls::progress::{ProgressRenderModel, ProgressTemplate};
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
use gpui_luma::controls::navigation_sidebar::{NavNode, NavigationSidebar};
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
    ControlFocusState as TabsControlFocusState, TabsNavigation, TabsNavigationClickHandler, TabsNavigationEvent,
    TabsNavigationHoverHandler, TabsNavigationItem, TabsNavigationItemState, TabsNavigationMouseDownHandler,
    TabsNavigationMouseUpHandler, TabsNavigationRenderItem, TabsNavigationRenderModel, TabsNavigationTemplate,
    TabsNavigationTemplateHandlers, TabsNavigationWidthMode,
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
use gpui_luma::{GridLayout, GridTrack, declare_form, hstack, vstack};
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::{ShadcnButtonStyle, ShadcnLook};
use lucide_icons::Icon as LucideIcon;
use self::cards::buttons::{render_lucide_icon, round_icon_glyph};

#[path = "variant_state_table.rs"]
mod variant_state_table;

#[path = "cards/mod.rs"]
mod cards;

const TRACKER_HEIGHT: f32 = 280.0;
const TRACKER_INSET: f32 = 22.0;
const TRACKER_THUMB_HEIGHT: f32 = 40.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StyleGuideSection {
    Sidebar,
    Feedback,
    Buttons,
    IconButtons,
    Choice,
    Toggle,
    Menus,
    Selectors,
    Tabs,
    Inputs,
    Typography,
}

impl StyleGuideSection {
    const ALL: [Self; 11] = [
        Self::Typography,
        Self::Buttons,
        Self::IconButtons,
        Self::Choice,
        Self::Toggle,
        Self::Menus,
        Self::Selectors,
        Self::Sidebar,
        Self::Tabs,
        Self::Inputs,
        Self::Feedback,
    ];
}

declare_form! {
    pub struct StyleGuidePanel {
        controls: {},
        args: {
            look: Arc<ShadcnLook>,
        },
        fields: {
            scroll_handle: ScrollHandle = ScrollHandle::new(),
            last_scroll_offset: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            last_max_scroll: Rc<Cell<f32>> = Rc::new(Cell::new(0.0)),
            sidebar_preview: Option<Entity<NavigationSidebar>> = None,
            buttons_preview_tabs: Option<Entity<TabsNavigation>> = None,
            icon_buttons_preview_tabs: Option<Entity<TabsNavigation>> = None,
        }
    }
}

impl StyleGuidePanel {
    pub fn sync_snapshot(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.sync_sidebar_preview(cx);
        self.sync_buttons_preview_tabs(cx);
        self.sync_icon_buttons_preview_tabs(cx);
        cx.notify();
    }

    fn set_vertical_offset(&self, value: f32, cx: &mut Context<Self>) {
        self.scroll_handle.set_offset(point(px(0.0), px(-value.max(0.0))));
        cx.notify();
    }

    fn scroll_vertical_by(&self, delta: Pixels, cx: &mut Context<Self>) -> bool {
        let current = (-self.scroll_handle.offset().y.as_f32()).max(0.0);
        let max = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        let target = (current + delta.as_f32()).clamp(0.0, max);
        if (target - current).abs() <= 0.5 {
            return false;
        }

        self.set_vertical_offset(target, cx);
        true
    }

    fn sidebar_preview(&mut self, cx: &mut Context<Self>) -> Entity<NavigationSidebar> {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            return sidebar;
        }

        let sidebar = self
            .look
            .navigation_sidebar("theme-studio-style-guide-sidebar-preview")
            .title("Properties")
            .subtitle("Task workspace")
            .collapsible(true)
            .selected_id(INITIAL_SIDEBAR_SELECTION_ID)
            .items(style_guide_sidebar_nodes())
            .footer_nodes(style_guide_sidebar_footer_nodes())
            .spawn(cx);

        self.sidebar_preview = Some(sidebar.clone());
        sidebar
    }

    fn sync_sidebar_preview(&mut self, cx: &mut Context<Self>) {
        if let Some(sidebar) = self.sidebar_preview.clone() {
            let look = self.look.clone();
            sidebar.update(cx, move |sidebar, cx| {
                sidebar.set_template(look.navigation_sidebar_template(), cx);
                sidebar.set_scrollbar_template(look.scrollbar_template(), cx);
                sidebar.set_items(style_guide_sidebar_nodes(), cx);
                sidebar.set_footer_nodes(style_guide_sidebar_footer_nodes(), cx);
                sidebar.set_selected_id(INITIAL_SIDEBAR_SELECTION_ID, cx);
            });
        }
    }

    fn sync_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-buttons-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.buttons_preview_tabs = Some(tabs.clone());
        tabs
    }

    fn sync_icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            let look = self.look.clone();
            tabs.update(cx, move |tabs, cx| {
                tabs.set_template(look.tabs_navigation_template(), cx);
            });
        }
    }

    fn icon_buttons_preview_tabs(&mut self, cx: &mut Context<Self>) -> Entity<TabsNavigation> {
        if let Some(tabs) = self.icon_buttons_preview_tabs.clone() {
            return tabs;
        }

        let tabs = self
            .look
            .tabs_navigation("theme-studio-icon-buttons-preview-tabs")
            .items([
                TabsNavigationItem::new("template-preview").label("Template Preview"),
                TabsNavigationItem::new("sizes").label("Sizes"),
            ])
            .active("template-preview")
            .width_mode(TabsNavigationWidthMode::Intrinsic)
            .template(self.look.tabs_navigation_template())
            .spawn(cx);

        cx.subscribe(&tabs, |_, _, _: &TabsNavigationEvent, cx| {
            cx.notify();
        })
        .detach();

        self.icon_buttons_preview_tabs = Some(tabs.clone());
        tabs
    }
}

impl Render for StyleGuidePanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _ = self.sidebar_preview(cx);
        let _ = self.buttons_preview_tabs(cx);
        let _ = self.icon_buttons_preview_tabs(cx);
        with_look(&self.look, || {
            let chrome = self.look.chrome();
            let scroll_progress = self.scroll_progress();
            let scroll_handle = self.scroll_handle.clone();
            let last_scroll_offset = self.last_scroll_offset.clone();
            let last_max_scroll = self.last_max_scroll.clone();

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
                        .min_h(px(0.0))
                        .flex_1()
                        .flex()
                        .items_stretch()
                        .gap(px(16.0))
                        .child(
                            div()
                                .id("style-guide-content")
                                .flex_1()
                                .min_h(px(0.0))
                                .overflow_y_scroll()
                                .scrollbar_width(px(0.0))
                                .track_scroll(&self.scroll_handle)
                                .on_prepaint(move |_, window: &mut Window, cx: &mut App| {
                                    let offset = (-scroll_handle.offset().y.as_f32()).max(0.0);
                                    let max_scroll = scroll_handle.max_offset().y.as_f32().max(0.0);
                                    let offset_changed = (last_scroll_offset.get() - offset).abs() > 0.5;
                                    let max_changed = (last_max_scroll.get() - max_scroll).abs() > 0.5;

                                    if offset_changed || max_changed {
                                        last_scroll_offset.set(offset);
                                        last_max_scroll.set(max_scroll);
                                        cx.notify(window.current_view());
                                    }
                                })
                                .child(
                                    div().w_full().flex().justify_start().pb(px(12.0)).child(
                                        div()
                                            .w_full()
                                            .max_w(px(980.0))
                                            .flex()
                                            .flex_col()
                                            .gap(px(20.0))
                                            .when_some(
                                                render_sparse_catalog_callout(self.look.as_ref()),
                                                |panel, callout| panel.child(callout),
                                            )
                                            .children(
                                                StyleGuideSection::ALL
                                                    .into_iter()
                                                    .map(|section| self.render_section_shell(section, window, cx)),
                                            ),
                                    ),
                                ),
                        )
                        .child(self.render_scroll_tracker(scroll_progress)),
                )
        })
    }
}

impl StyleGuidePanel {
    fn render_section_shell(
        &self,
        section: StyleGuideSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let content = self.render_section(section, window, cx);
        let shell = div().relative().w_full().child(content);

        if Self::section_allows_interaction(section) {
            return shell
                .on_scroll_wheel(cx.listener(Self::handle_section_overlay_scroll_wheel))
                .into_any_element();
        }

        shell
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_mouse_down(MouseButton::Right, |_, window, cx| {
                        window.prevent_default();
                        cx.stop_propagation();
                    })
                    .on_scroll_wheel(cx.listener(Self::handle_section_overlay_scroll_wheel))
                    .on_key_down(cx.listener(Self::handle_section_overlay_key_down)),
            )
            .into_any_element()
    }

    fn section_allows_interaction(section: StyleGuideSection) -> bool {
        matches!(section, StyleGuideSection::Buttons | StyleGuideSection::IconButtons)
    }

    fn scroll_progress(&self) -> f32 {
        let max_scroll = self.scroll_handle.max_offset().y.as_f32().max(0.0);
        if max_scroll <= 0.5 {
            return 0.0;
        }

        (-self.scroll_handle.offset().y.as_f32()).clamp(0.0, max_scroll) / max_scroll
    }

    fn render_section(&self, section: StyleGuideSection, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        match section {
            StyleGuideSection::Sidebar => cards::sidebar::render_sidebar_template_section(
                self.sidebar_preview.clone().expect("sidebar preview"),
                self.look.as_ref(),
            ),
            StyleGuideSection::Feedback => {
                cards::feedback::render_feedback_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Buttons => cards::buttons::render_button_template_matrix_section(
                self.look.clone(),
                self.buttons_preview_tabs.clone().expect("buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::IconButtons => cards::buttons::render_icon_button_template_matrix_section(
                self.look.clone(),
                self.icon_buttons_preview_tabs.clone().expect("icon buttons preview tabs"),
                window,
                cx,
            ),
            StyleGuideSection::Choice => {
                cards::buttons::render_choice_template_matrix_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Toggle => {
                cards::buttons::render_toggle_template_matrix_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Menus => cards::menus::render_menu_template_state_section(self.look.clone(), window, cx),
            StyleGuideSection::Selectors => {
                cards::selectors::render_selector_templates_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Tabs => {
                cards::menus::render_tabs_navigation_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Inputs => {
                cards::inputs::render_input_controls_template_section(self.look.clone(), window, cx)
            }
            StyleGuideSection::Typography => cards::typography::render_typography_section(self.look.as_ref()),
        }
    }

    fn render_scroll_tracker(&self, progress: f32) -> AnyElement {
        let chrome = self.look.chrome();
        let track_span = TRACKER_HEIGHT - (TRACKER_INSET * 2.0) - TRACKER_THUMB_HEIGHT;
        let thumb_top = TRACKER_INSET + (track_span.max(0.0) * progress.clamp(0.0, 1.0));

        div()
            .id("style-guide-scroll-tracker-shell")
            .w(px(28.0))
            .flex_shrink_0()
            .child(
                div().w_full().h_full().flex().items_center().justify_center().child(
                    div()
                        .relative()
                        .w(px(12.0))
                        .h(px(TRACKER_HEIGHT))
                        .child(
                            div()
                                .absolute()
                                .top(px(TRACKER_INSET))
                                .bottom(px(TRACKER_INSET))
                                .left(px(5.0))
                                .w(px(2.0))
                                .rounded_full()
                                .bg(gpui::hsla(chrome.border.h, chrome.border.s, chrome.border.l, 0.9)),
                        )
                        .children(
                            StyleGuideSection::ALL
                                .into_iter()
                                .enumerate()
                                .map(|(index, _)| render_tracker_marker(index, chrome.muted_text)),
                        )
                        .child(
                            div()
                                .absolute()
                                .top(px(thumb_top))
                                .left(px(0.0))
                                .w(px(12.0))
                                .h(px(TRACKER_THUMB_HEIGHT))
                                .rounded_full()
                                .bg(chrome.title_text),
                        ),
                ),
            )
            .into_any_element()
    }

    fn handle_section_overlay_scroll_wheel(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let delta = event.delta.pixel_delta(px(40.0)).y.as_f32();
        if !delta.is_finite() || delta.abs() <= f32::EPSILON {
            return;
        }

        if self.scroll_vertical_by(px(-delta), cx) {
            window.prevent_default();
            cx.stop_propagation();
        }
    }

    fn handle_section_overlay_key_down(&mut self, _: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.prevent_default();
        cx.stop_propagation();
    }
}

fn render_tracker_marker(index: usize, color: gpui::Hsla) -> AnyElement {
    let count = StyleGuideSection::ALL.len();
    let usable_height = TRACKER_HEIGHT - (TRACKER_INSET * 2.0);
    let step = if count > 1 {
        usable_height / (count.saturating_sub(1) as f32)
    } else {
        0.0
    };
    let top = TRACKER_INSET + (index as f32 * step) - 2.0;

    div()
        .absolute()
        .top(px(top))
        .left(px(3.0))
        .size(px(6.0))
        .rounded_full()
        .bg(gpui::hsla(color.h, color.s, color.l, 0.85))
        .into_any_element()
}

const INITIAL_SIDEBAR_SELECTION_ID: &str = "dimensions";

fn style_guide_sidebar_nodes() -> Vec<NavNode> {
    vec![
        NavNode::section("pinned-label", "Pinned"),
        sidebar_leaf_node("summary", "Summary", Some(LucideIcon::Info), true),
        sidebar_leaf_node("tokens", "Design Tokens", Some(LucideIcon::Tags), true),
        NavNode::section("properties-label", "Properties"),
        NavNode::new("layout").label("Layout").icon(LucideIcon::Ruler).expanded(true).children([
            sidebar_leaf_node("position", "Position", None, true),
            sidebar_leaf_node(INITIAL_SIDEBAR_SELECTION_ID, "Dimensions", None, true),
            sidebar_leaf_node("constraints", "Constraints", None, true),
            sidebar_leaf_node("grid", "Grid", None, true),
        ]),
        NavNode::new("look").label("Look").icon(LucideIcon::Palette).expanded(true).children([
            sidebar_leaf_node("fill", "Fill", None, true),
            sidebar_leaf_node("stroke", "Stroke", None, true),
            sidebar_leaf_node("typography", "Typography", None, true),
            sidebar_leaf_node("effects", "Effects", None, true),
        ]),
        NavNode::new("behavior")
            .label("Behavior")
            .icon(LucideIcon::MousePointer2)
            .expanded(false)
            .children([
                sidebar_leaf_node("interactions", "Interactions", None, true),
                sidebar_leaf_node("conditions", "Conditions", None, true),
                sidebar_leaf_node("validation", "Validation", None, true),
                sidebar_leaf_node("data-binding", "Data Binding", None, false),
            ]),
    ]
}

fn style_guide_sidebar_footer_nodes() -> Vec<NavNode> {
    vec![
        sidebar_leaf_node("audit-log", "Audit Log", Some(LucideIcon::FileText), true),
        sidebar_leaf_node("reset-overrides", "Reset Overrides", Some(LucideIcon::RotateCcw), false),
    ]
}

fn sidebar_leaf_node(id: &'static str, label: &'static str, icon: Option<LucideIcon>, enabled: bool) -> NavNode {
    let mut node = NavNode::new(id).label(label).enabled(enabled);
    if let Some(icon) = icon {
        node = node.icon(icon);
    }
    node
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
struct ProgressStateSample {
    id: &'static str,
    label: &'static str,
    value: f32,
    enabled: bool,
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
    _background: gpui::Hsla,
    content: AnyElement,
) -> AnyElement {
    div()
        .w_full()
        .flex()
        .flex_col()
        .py(px(12.0))
        .child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(div().text_h2().text_color(title_color).child(title))
                .child(
                    div().w_full().min_w(px(0.0)).truncate().typography_sm().text_color(muted_text).child(description),
                ),
        )
        .child(div().w_full().h(px(1.0)).mt(px(10.0)).bg(border))
        .child(
            div()
                .w_full()
                .flex()
                .justify_center()
                .mt(px(16.0))
                .child(div().w(px(width)).max_w_full().child(content)),
        )
        .into_any_element()
}

fn render_vertical_section_rail(label: &'static str, color: gpui::Hsla, data_rows: usize) -> AnyElement {
    let min_height = section_rail_min_height(data_rows);
    let label_height = section_rail_label_height(data_rows);

    div()
        .w(px(28.0))
        .min_h(px(min_height))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.0))
        .child(svg().path(section_label_asset_path(label)).w(px(20.0)).h(px(label_height)).text_color(color))
        .child(div().w(px(1.0)).h_full().bg(color))
        .into_any_element()
}

fn section_rail_min_height(data_rows: usize) -> f32 {
    match data_rows {
        0 => 48.0,
        1 => 64.0,
        _ => 188.0,
    }
}

fn section_rail_label_height(data_rows: usize) -> f32 {
    match data_rows {
        0 | 1 => 38.0,
        2 => 60.0,
        3 => 80.0,
        _ => 104.0,
    }
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
