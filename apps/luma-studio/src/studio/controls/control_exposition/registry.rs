use std::sync::Arc;

use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement};

use super::accordion::AccordionControlExposition;
use super::autocomplete_textfield::AutocompleteTextFieldControlExposition;
use super::badge::BadgeControlExposition;
use super::button::ButtonControlExposition;
use super::checkbox::CheckboxControlExposition;
use super::color_arc::ColorArcControlExposition;
use super::color_field::ColorFieldControlExposition;
use super::color_harmonies::ColorHarmoniesControlExposition;
use super::color_hsv_plane::ColorHsvPlaneControlExposition;
use super::color_hsv_wheel::ColorHsvWheelControlExposition;
use super::color_multi_mixer::ColorMultiMixerControlExposition;
use super::color_picker::ColorPickerControlExposition;
use super::color_ring::ColorRingControlExposition;
use super::color_slider::ColorSliderControlExposition;
use super::color_slider_revealed::ColorSliderRevealedControlExposition;
use super::color_split_ring::ColorSplitRingControlExposition;
use super::color_sv_triangle::ColorSvTriangleControlExposition;
use super::combobox::ComboBoxControlExposition;
use super::context_menu::ContextMenuControlExposition;
use super::custom_button::CustomButtonControlExposition;
use super::dock_panel::DockPanelControlExposition;
use super::floating_menu::FloatingMenuControlExposition;
use super::listbox::ListBoxControlExposition;
use super::modal_overlay::ModalOverlayControlExposition;
use super::navigation_sidebar::NavigationSidebarControlExposition;
use super::pager::PagerControlExposition;
use super::paging_list_view::PagingListViewControlExposition;
use super::popup_selector::PopupSelectorControlExposition;
use super::progress::ProgressControlExposition;
use super::radio_button::RadioButtonControlExposition;
use super::radio_group::RadioGroupControlExposition;
use super::resizable_panels::ResizablePanelsControlExposition;
use super::popup_menu::PopupMenuControlExposition;
use super::scrollbar::ScrollbarControlExposition;
use super::scrolling_list_view::ScrollingListViewControlExposition;
use super::search_selector::SearchSelectorControlExposition;
use super::selection_panel::SelectionPanelControlExposition;
use super::slider::SliderControlExposition;
use super::slide_panel::SlidePanelControlExposition;
use super::switch::SwitchControlExposition;
use super::tabs_navigation::TabsNavigationControlExposition;
use super::textarea::TextAreaControlExposition;
use super::textfield::TextFieldControlExposition;
use super::toggle::ToggleControlExposition;
use super::toggle_group::ToggleGroupControlExposition;
use super::toolbar::ToolbarControlExposition;
use super::tree_view::TreeViewControlExposition;
use crate::studio::controls::catalog::ControlDocEntry;

pub enum ControlExposition {
    Accordion(Entity<AccordionControlExposition>),
    Button(Entity<ButtonControlExposition>),
    CustomButton(Entity<CustomButtonControlExposition>),
    Checkbox(Entity<CheckboxControlExposition>),
    RadioButton(Entity<RadioButtonControlExposition>),
    RadioGroup(Entity<RadioGroupControlExposition>),
    Switch(Entity<SwitchControlExposition>),
    Toggle(Entity<ToggleControlExposition>),
    ToggleGroup(Entity<ToggleGroupControlExposition>),
    Toolbar(Entity<ToolbarControlExposition>),
    Pager(Entity<PagerControlExposition>),
    TreeView(Entity<TreeViewControlExposition>),
    Badge(Entity<BadgeControlExposition>),
    Progress(Entity<ProgressControlExposition>),
    ColorArc(Entity<ColorArcControlExposition>),
    ColorField(Entity<ColorFieldControlExposition>),
    ColorHarmonies(Entity<ColorHarmoniesControlExposition>),
    ColorHsvPlane(Entity<ColorHsvPlaneControlExposition>),
    ColorHsvWheel(Entity<ColorHsvWheelControlExposition>),
    ColorMultiMixer(Entity<ColorMultiMixerControlExposition>),
    ColorPicker(Entity<ColorPickerControlExposition>),
    ColorRing(Entity<ColorRingControlExposition>),
    ColorSlider(Entity<ColorSliderControlExposition>),
    ColorSliderRevealed(Entity<ColorSliderRevealedControlExposition>),
    ColorSplitRing(Entity<ColorSplitRingControlExposition>),
    ColorSvTriangle(Entity<ColorSvTriangleControlExposition>),
    TextField(Entity<TextFieldControlExposition>),
    TextArea(Entity<TextAreaControlExposition>),
    Slider(Entity<SliderControlExposition>),
    Scrollbar(Entity<ScrollbarControlExposition>),
    DockPanel(Entity<DockPanelControlExposition>),
    ResizablePanels(Entity<ResizablePanelsControlExposition>),
    NavigationSidebar(Entity<NavigationSidebarControlExposition>),
    TabsNavigation(Entity<TabsNavigationControlExposition>),
    ContextMenu(Entity<ContextMenuControlExposition>),
    FloatingMenu(Entity<FloatingMenuControlExposition>),
    PagingListView(Entity<PagingListViewControlExposition>),
    PopupMenu(Entity<PopupMenuControlExposition>),
    AutocompleteTextField(Entity<AutocompleteTextFieldControlExposition>),
    ComboBox(Entity<ComboBoxControlExposition>),
    ScrollingListView(Entity<ScrollingListViewControlExposition>),
    SearchSelector(Entity<SearchSelectorControlExposition>),
    PopupSelector(Entity<PopupSelectorControlExposition>),
    SelectionPanel(Entity<SelectionPanelControlExposition>),
    ListBox(Entity<ListBoxControlExposition>),
    ModalOverlay(Entity<ModalOverlayControlExposition>),
    SlidePanel(Entity<SlidePanelControlExposition>),
}

impl ControlExposition {
    pub fn spawn_all<T: 'static>(look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut Context<T>) -> Vec<Self> {
        vec![
            Self::Accordion(cx.new(|cx| AccordionControlExposition::new(cx, look.clone()))),
            Self::Button(cx.new(|cx| ButtonControlExposition::new(cx, look.clone()))),
            Self::CustomButton(cx.new(|cx| CustomButtonControlExposition::new(cx, look.clone()))),
            Self::Checkbox(cx.new(|cx| CheckboxControlExposition::new(cx, look.clone()))),
            Self::RadioButton(cx.new(|cx| RadioButtonControlExposition::new(cx, look.clone()))),
            Self::RadioGroup(cx.new(|cx| RadioGroupControlExposition::new(cx, look.clone()))),
            Self::Switch(cx.new(|cx| SwitchControlExposition::new(cx, look.clone()))),
            Self::Toggle(cx.new(|cx| ToggleControlExposition::new(cx, look.clone()))),
            Self::ToggleGroup(cx.new(|cx| ToggleGroupControlExposition::new(cx, look.clone()))),
            Self::ListBox(cx.new(|cx| ListBoxControlExposition::new(cx, look.clone()))),
            Self::Toolbar(cx.new(|cx| ToolbarControlExposition::new(cx, look.clone()))),
            Self::Pager(cx.new(|cx| PagerControlExposition::new(cx, look.clone()))),
            Self::ScrollingListView(cx.new(|cx| ScrollingListViewControlExposition::new(cx, look.clone()))),
            Self::PagingListView(cx.new(|cx| PagingListViewControlExposition::new(cx, look.clone()))),
            Self::TreeView(cx.new(|cx| TreeViewControlExposition::new(cx, look.clone()))),
            Self::Badge(cx.new(|cx| BadgeControlExposition::new(cx, look.clone()))),
            Self::Progress(cx.new(|cx| ProgressControlExposition::new(cx, look.clone()))),
            Self::ColorArc(cx.new(|cx| ColorArcControlExposition::new(cx, look.clone()))),
            Self::ColorField(cx.new(|cx| ColorFieldControlExposition::new(cx, look.clone()))),
            Self::ColorHarmonies(cx.new(|cx| ColorHarmoniesControlExposition::new(cx, look.clone()))),
            Self::ColorHsvPlane(cx.new(|cx| ColorHsvPlaneControlExposition::new(cx, look.clone()))),
            Self::ColorHsvWheel(cx.new(|cx| ColorHsvWheelControlExposition::new(cx, look.clone()))),
            Self::ColorMultiMixer(cx.new(|cx| ColorMultiMixerControlExposition::new(cx, look.clone()))),
            Self::ColorPicker(cx.new(|cx| ColorPickerControlExposition::new(cx, look.clone()))),
            Self::ColorRing(cx.new(|cx| ColorRingControlExposition::new(cx, look.clone()))),
            Self::ColorSlider(cx.new(|cx| ColorSliderControlExposition::new(cx, look.clone()))),
            Self::ColorSliderRevealed(cx.new(|cx| ColorSliderRevealedControlExposition::new(cx, look.clone()))),
            Self::ColorSplitRing(cx.new(|cx| ColorSplitRingControlExposition::new(cx, look.clone()))),
            Self::ColorSvTriangle(cx.new(|cx| ColorSvTriangleControlExposition::new(cx, look.clone()))),
            Self::TextField(cx.new(|cx| TextFieldControlExposition::new(cx, look.clone()))),
            Self::TextArea(cx.new(|cx| TextAreaControlExposition::new(cx, look.clone()))),
            Self::Slider(cx.new(|cx| SliderControlExposition::new(cx, look.clone()))),
            Self::Scrollbar(cx.new(|cx| ScrollbarControlExposition::new(cx, look.clone()))),
            Self::DockPanel(cx.new(|cx| DockPanelControlExposition::new(cx, look.clone()))),
            Self::ResizablePanels(cx.new(|cx| ResizablePanelsControlExposition::new(cx, look.clone()))),
            Self::NavigationSidebar(cx.new(|cx| NavigationSidebarControlExposition::new(cx, look.clone()))),
            Self::TabsNavigation(cx.new(|cx| TabsNavigationControlExposition::new(cx, look.clone()))),
            Self::ContextMenu(cx.new(|cx| ContextMenuControlExposition::new(cx, look.clone()))),
            Self::FloatingMenu(cx.new(|cx| FloatingMenuControlExposition::new(cx, look.clone()))),
            Self::PopupMenu(cx.new(|cx| PopupMenuControlExposition::new(cx, look.clone()))),
            Self::AutocompleteTextField(cx.new(|cx| AutocompleteTextFieldControlExposition::new(cx, look.clone()))),
            Self::ComboBox(cx.new(|cx| ComboBoxControlExposition::new(cx, look.clone()))),
            Self::SearchSelector(cx.new(|cx| SearchSelectorControlExposition::new(cx, look.clone()))),
            Self::PopupSelector(cx.new(|cx| PopupSelectorControlExposition::new(cx, look.clone()))),
            Self::SelectionPanel(cx.new(|cx| SelectionPanelControlExposition::new(cx, look.clone()))),
            Self::ModalOverlay(cx.new(|cx| ModalOverlayControlExposition::new(cx, look.clone()))),
            Self::SlidePanel(cx.new(|cx| SlidePanelControlExposition::new(cx, look.clone()))),
        ]
    }

    pub fn id(&self, cx: &App) -> &'static str {
        self.entry(cx).id
    }

    pub fn entry(&self, cx: &App) -> ControlDocEntry {
        match self {
            Self::Accordion(entity) => entity.read(cx).entry(),
            Self::Button(entity) => entity.read(cx).entry(),
            Self::CustomButton(entity) => entity.read(cx).entry(),
            Self::Checkbox(entity) => entity.read(cx).entry(),
            Self::RadioButton(entity) => entity.read(cx).entry(),
            Self::RadioGroup(entity) => entity.read(cx).entry(),
            Self::Switch(entity) => entity.read(cx).entry(),
            Self::Toggle(entity) => entity.read(cx).entry(),
            Self::ToggleGroup(entity) => entity.read(cx).entry(),
            Self::ListBox(entity) => entity.read(cx).entry(),
            Self::Toolbar(entity) => entity.read(cx).entry(),
            Self::Pager(entity) => entity.read(cx).entry(),
            Self::ScrollingListView(entity) => entity.read(cx).entry(),
            Self::PagingListView(entity) => entity.read(cx).entry(),
            Self::TreeView(entity) => entity.read(cx).entry(),
            Self::Badge(entity) => entity.read(cx).entry(),
            Self::Progress(entity) => entity.read(cx).entry(),
            Self::ColorArc(entity) => entity.read(cx).entry(),
            Self::ColorField(entity) => entity.read(cx).entry(),
            Self::ColorHarmonies(entity) => entity.read(cx).entry(),
            Self::ColorHsvPlane(entity) => entity.read(cx).entry(),
            Self::ColorHsvWheel(entity) => entity.read(cx).entry(),
            Self::ColorMultiMixer(entity) => entity.read(cx).entry(),
            Self::ColorPicker(entity) => entity.read(cx).entry(),
            Self::ColorRing(entity) => entity.read(cx).entry(),
            Self::ColorSlider(entity) => entity.read(cx).entry(),
            Self::ColorSliderRevealed(entity) => entity.read(cx).entry(),
            Self::ColorSplitRing(entity) => entity.read(cx).entry(),
            Self::ColorSvTriangle(entity) => entity.read(cx).entry(),
            Self::TextField(entity) => entity.read(cx).entry(),
            Self::TextArea(entity) => entity.read(cx).entry(),
            Self::Slider(entity) => entity.read(cx).entry(),
            Self::Scrollbar(entity) => entity.read(cx).entry(),
            Self::DockPanel(entity) => entity.read(cx).entry(),
            Self::ResizablePanels(entity) => entity.read(cx).entry(),
            Self::NavigationSidebar(entity) => entity.read(cx).entry(),
            Self::TabsNavigation(entity) => entity.read(cx).entry(),
            Self::ContextMenu(entity) => entity.read(cx).entry(),
            Self::FloatingMenu(entity) => entity.read(cx).entry(),
            Self::PopupMenu(entity) => entity.read(cx).entry(),
            Self::AutocompleteTextField(entity) => entity.read(cx).entry(),
            Self::ComboBox(entity) => entity.read(cx).entry(),
            Self::SearchSelector(entity) => entity.read(cx).entry(),
            Self::PopupSelector(entity) => entity.read(cx).entry(),
            Self::SelectionPanel(entity) => entity.read(cx).entry(),
            Self::ModalOverlay(entity) => entity.read(cx).entry(),
            Self::SlidePanel(entity) => entity.read(cx).entry(),
        }
    }

    pub fn sync_look(&self, look: Arc<gpui_luma_look_shadcn::ShadcnLook>, cx: &mut App) {
        match self {
            Self::Accordion(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Button(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::CustomButton(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Checkbox(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::RadioButton(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::RadioGroup(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Switch(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Toggle(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ToggleGroup(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ListBox(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Toolbar(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Pager(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ScrollingListView(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::PagingListView(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::TreeView(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Badge(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Progress(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorArc(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorField(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorHarmonies(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorHsvPlane(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorHsvWheel(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorMultiMixer(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorPicker(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorRing(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorSlider(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorSliderRevealed(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorSplitRing(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ColorSvTriangle(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::TextField(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::TextArea(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Slider(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::Scrollbar(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::DockPanel(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ResizablePanels(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::NavigationSidebar(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::TabsNavigation(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ContextMenu(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::FloatingMenu(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::PopupMenu(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::AutocompleteTextField(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ComboBox(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::SearchSelector(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::PopupSelector(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::SelectionPanel(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::ModalOverlay(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
            Self::SlidePanel(entity) => entity.update(cx, |exposition, cx| exposition.sync_look(look, cx)),
        }
    }

    pub fn render(&self, _cx: &App) -> AnyElement {
        match self {
            Self::Accordion(entity) => entity.clone().into_any_element(),
            Self::Button(entity) => entity.clone().into_any_element(),
            Self::CustomButton(entity) => entity.clone().into_any_element(),
            Self::Checkbox(entity) => entity.clone().into_any_element(),
            Self::RadioButton(entity) => entity.clone().into_any_element(),
            Self::RadioGroup(entity) => entity.clone().into_any_element(),
            Self::Switch(entity) => entity.clone().into_any_element(),
            Self::Toggle(entity) => entity.clone().into_any_element(),
            Self::ToggleGroup(entity) => entity.clone().into_any_element(),
            Self::ListBox(entity) => entity.clone().into_any_element(),
            Self::Toolbar(entity) => entity.clone().into_any_element(),
            Self::Pager(entity) => entity.clone().into_any_element(),
            Self::ScrollingListView(entity) => entity.clone().into_any_element(),
            Self::PagingListView(entity) => entity.clone().into_any_element(),
            Self::TreeView(entity) => entity.clone().into_any_element(),
            Self::Badge(entity) => entity.clone().into_any_element(),
            Self::Progress(entity) => entity.clone().into_any_element(),
            Self::ColorArc(entity) => entity.clone().into_any_element(),
            Self::ColorField(entity) => entity.clone().into_any_element(),
            Self::ColorHarmonies(entity) => entity.clone().into_any_element(),
            Self::ColorHsvPlane(entity) => entity.clone().into_any_element(),
            Self::ColorHsvWheel(entity) => entity.clone().into_any_element(),
            Self::ColorMultiMixer(entity) => entity.clone().into_any_element(),
            Self::ColorPicker(entity) => entity.clone().into_any_element(),
            Self::ColorRing(entity) => entity.clone().into_any_element(),
            Self::ColorSlider(entity) => entity.clone().into_any_element(),
            Self::ColorSliderRevealed(entity) => entity.clone().into_any_element(),
            Self::ColorSplitRing(entity) => entity.clone().into_any_element(),
            Self::ColorSvTriangle(entity) => entity.clone().into_any_element(),
            Self::TextField(entity) => entity.clone().into_any_element(),
            Self::TextArea(entity) => entity.clone().into_any_element(),
            Self::Slider(entity) => entity.clone().into_any_element(),
            Self::Scrollbar(entity) => entity.clone().into_any_element(),
            Self::DockPanel(entity) => entity.clone().into_any_element(),
            Self::ResizablePanels(entity) => entity.clone().into_any_element(),
            Self::NavigationSidebar(entity) => entity.clone().into_any_element(),
            Self::TabsNavigation(entity) => entity.clone().into_any_element(),
            Self::ContextMenu(entity) => entity.clone().into_any_element(),
            Self::FloatingMenu(entity) => entity.clone().into_any_element(),
            Self::PopupMenu(entity) => entity.clone().into_any_element(),
            Self::AutocompleteTextField(entity) => entity.clone().into_any_element(),
            Self::ComboBox(entity) => entity.clone().into_any_element(),
            Self::SearchSelector(entity) => entity.clone().into_any_element(),
            Self::PopupSelector(entity) => entity.clone().into_any_element(),
            Self::SelectionPanel(entity) => entity.clone().into_any_element(),
            Self::ModalOverlay(entity) => entity.clone().into_any_element(),
            Self::SlidePanel(entity) => entity.clone().into_any_element(),
        }
    }

    pub fn find<'a>(expositions: &'a [Self], id: &str, cx: &App) -> Option<&'a Self> {
        expositions.iter().find(|exposition| exposition.id(cx) == id)
    }
}
