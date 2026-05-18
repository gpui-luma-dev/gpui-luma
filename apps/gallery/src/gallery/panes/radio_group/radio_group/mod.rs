use std::sync::Arc;

use gpui::{SharedString, div, prelude::*};
use gpui_luma::controls::command::button::ButtonTemplate;

mod control;

pub(in crate::gallery) use control::{
    RadioGroup, RadioGroupBuilder, RadioGroupEvent, RadioGroupTemplate, SelectionMode, new,
};

pub(in crate::gallery) fn vertical_group<T>(
    id: impl Into<SharedString>,
    item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupBuilder<T>
where
    T: Clone + Eq + 'static,
{
    new(id, item_id, item_label, item_button_template).template(vertical_group_template())
}

pub(in crate::gallery) fn horizontal_group<T>(
    id: impl Into<SharedString>,
    item_id: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_label: impl Fn(&T) -> SharedString + Send + Sync + 'static,
    item_button_template: Arc<dyn ButtonTemplate<bool>>,
) -> RadioGroupBuilder<T>
where
    T: Clone + Eq + 'static,
{
    new(id, item_id, item_label, item_button_template).template(horizontal_group_template())
}

pub(in crate::gallery) fn vertical_group_template<T>() -> RadioGroupTemplate<T>
where
    T: Clone + Eq + 'static,
{
    Arc::new(|group, _window, _cx| {
        div().flex().flex_col().gap_2().children(group.buttons().iter().cloned()).into_any_element()
    })
}

pub(in crate::gallery) fn horizontal_group_template<T>() -> RadioGroupTemplate<T>
where
    T: Clone + Eq + 'static,
{
    Arc::new(|group, _window, _cx| {
        div().flex().items_center().gap_3().children(group.buttons().iter().cloned()).into_any_element()
    })
}
