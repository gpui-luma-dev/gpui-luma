use std::sync::Arc;

use gpui::{AnyElement, App, IntoElement, SharedString};

pub struct SearchSelectorItemRenderModel<'a, T> {
    pub search_selector_id: &'a SharedString,
    pub item: &'a T,
    pub source_index: usize,
    pub visible_index: usize,
    pub selected: bool,
    pub active: bool,
    pub open: bool,
    pub enabled: bool,
}

pub type SearchSelectorItemTemplate<T> =
    Arc<dyn for<'a> Fn(&SearchSelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static>;

pub type SearchSelectorItemTemplateModifier<T> = Box<
    dyn for<'a> Fn(AnyElement, &SearchSelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
>;

pub fn make_search_selector_item_template<T, F, E>(template: F) -> SearchSelectorItemTemplate<T>
where
    F: for<'a> Fn(&SearchSelectorItemRenderModel<'a, T>, &mut App) -> E + Send + Sync + 'static,
    E: IntoElement + 'static,
{
    Arc::new(move |model, cx| template(model, cx).into_any_element())
}

struct ModifiedSearchSelectorItemTemplate<T> {
    base: SearchSelectorItemTemplate<T>,
    modifiers: Vec<SearchSelectorItemTemplateModifier<T>>,
}

impl<T: 'static> ModifiedSearchSelectorItemTemplate<T> {
    fn new(base: SearchSelectorItemTemplate<T>) -> Self {
        Self { base, modifiers: Vec::new() }
    }

    fn with_modifier<F>(mut self, modifier: F) -> Self
    where
        F: for<'a> Fn(AnyElement, &SearchSelectorItemRenderModel<'a, T>, &mut App) -> AnyElement
            + Send
            + Sync
            + 'static,
    {
        self.modifiers.push(Box::new(modifier));
        self
    }

    fn into_template(self) -> SearchSelectorItemTemplate<T> {
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

pub fn item_template_with_modifier<T, F>(
    template: SearchSelectorItemTemplate<T>,
    modifier: F,
) -> SearchSelectorItemTemplate<T>
where
    T: 'static,
    F: for<'a> Fn(AnyElement, &SearchSelectorItemRenderModel<'a, T>, &mut App) -> AnyElement + Send + Sync + 'static,
{
    ModifiedSearchSelectorItemTemplate::new(template).with_modifier(modifier).into_template()
}
