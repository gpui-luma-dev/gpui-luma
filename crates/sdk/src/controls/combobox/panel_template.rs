use std::sync::{Arc, OnceLock};

use gpui::{AnyElement, App, Bounds, Pixels, SharedString, anchored, deferred, div, point, prelude::*, px};

use crate::controls::selector_panel::SelectorItemsPanelLook;

use super::behavior::SelectionItem;
use super::item_template::ComboBoxItemTemplate;

pub struct ComboBoxPanelRenderModel<'a> {
    pub id: &'a SharedString,
    pub items: &'a [SelectionItem],
    pub visible_indices: &'a [usize],
    pub selected_source_index: Option<usize>,
    pub active_visible_index: Option<usize>,
    pub open: bool,
    pub enabled: bool,
    pub item_template: Option<&'a ComboBoxItemTemplate<SelectionItem>>,
    pub popup_bounds: Option<Bounds<Pixels>>,
    pub popup_look: SelectorItemsPanelLook,
    pub list_content: AnyElement,
}

pub trait ComboBoxPanelTemplate: Send + Sync {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement;
}

pub type ComboBoxPanelTemplateModifier = Box<dyn Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static>;

pub struct DefaultComboBoxPanelTemplate;

pub fn default_combobox_panel_template() -> Arc<dyn ComboBoxPanelTemplate> {
    static TEMPLATE: OnceLock<Arc<dyn ComboBoxPanelTemplate>> = OnceLock::new();
    TEMPLATE.get_or_init(|| Arc::new(DefaultComboBoxPanelTemplate)).clone()
}

struct ModifiedComboBoxPanelTemplate {
    base: Arc<dyn ComboBoxPanelTemplate>,
    modifiers: Vec<ComboBoxPanelTemplateModifier>,
}

impl ModifiedComboBoxPanelTemplate {
    fn new(base: Arc<dyn ComboBoxPanelTemplate>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }
}

impl ComboBoxPanelTemplate for ModifiedComboBoxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, cx: &mut App) -> AnyElement {
        let mut element = self.base.render(model, cx);
        for modifier in &self.modifiers {
            element = modifier(element, cx);
        }
        element
    }
}

pub fn panel_template_with_modifier<F>(
    template: Arc<dyn ComboBoxPanelTemplate>,
    modifier: F,
) -> Arc<dyn ComboBoxPanelTemplate>
where
    F: Fn(AnyElement, &mut App) -> AnyElement + Send + Sync + 'static,
{
    Arc::new(ModifiedComboBoxPanelTemplate::new(template).with_modifier(modifier))
}

impl ComboBoxPanelTemplate for DefaultComboBoxPanelTemplate {
    fn render(&self, model: ComboBoxPanelRenderModel<'_>, _cx: &mut App) -> AnyElement {
        if let Some(bounds) = model.popup_bounds {
            let look = model.popup_look;

            return deferred(
                anchored()
                    .snap_to_window_with_margin(px(8.0))
                    .anchor(gpui::Corner::TopLeft)
                    .position(point(bounds.left(), bounds.bottom()))
                    .offset(point(px(0.0), px(4.0)))
                    .child(
                        div()
                            .id(format!("{}-popup-shell", model.id))
                            .w(bounds.size.width)
                            .bg(look.background)
                            .border_1()
                            .border_color(look.border)
                            .rounded(px(look.radius))
                            .shadow(look.shadow)
                            .overflow_hidden()
                            .occlude()
                            .child(model.list_content),
                    ),
            )
            .with_priority(1)
            .into_any_element();
        }

        let look = model.popup_look;
        div()
            .id(format!("{}-panel", model.id))
            .w_full()
            .bg(look.background)
            .border_1()
            .border_color(look.border)
            .rounded(px(look.radius))
            .shadow(look.shadow)
            .overflow_hidden()
            .occlude()
            .child(model.list_content)
            .into_any_element()
    }
}
