use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

pub struct ComboBoxItemRenderModel<'a, T> {
    pub combobox_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

pub type ComboBoxItemTemplate<T> =
    Arc<dyn for<'a> Fn(&ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub type ComboBoxItemTemplateModifier<T> =
    Box<dyn for<'a> Fn(AnyElement, &ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub fn make_combobox_item_template<T, F, E>(template: F) -> ComboBoxItemTemplate<T>
where
    F: for<'a> Fn(&ComboBoxItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

struct ModifiedComboBoxItemTemplate<T> {
    base: ComboBoxItemTemplate<T>,
    modifiers: Vec<ComboBoxItemTemplateModifier<T>>,
}

impl<T: 'static> ModifiedComboBoxItemTemplate<T> {
    fn new(base: ComboBoxItemTemplate<T>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(AnyElement, &ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn into_template(self) -> ComboBoxItemTemplate<T> {
        let base = self.base;
        let modifiers = self.modifiers;
        Arc::new(move |model, cx| {
            let mut element = base(model, cx);
            for modifier in &modifiers {
                element = modifier(element, model, cx);
            }
            element
        })
    }
}

pub fn item_template_with_modifier<T: 'static, F>(
    template: ComboBoxItemTemplate<T>,
    modifier: F,
) -> ComboBoxItemTemplate<T>
where
    F: for<'a> Fn(AnyElement, &ComboBoxItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
{
    ModifiedComboBoxItemTemplate::new(template).with_modifier(modifier).into_template()
}
