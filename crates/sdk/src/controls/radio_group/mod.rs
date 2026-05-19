mod control;
mod focus;

use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::Arc;

use gpui::{App, Div, Stateful, Window, div, prelude::*};

pub use control::{RadioGroup, RadioGroupBuilder, RadioGroupEvent, RadioGroupTemplate, RadioItemState, SelectionMode, new};
use crate::controls::command::button::{ButtonRenderModel, ButtonTemplate};

struct SelectionStateTemplate<T> {
    base: Arc<dyn ButtonTemplate<bool>>,
    _marker: PhantomData<T>,
}

impl<T> ButtonTemplate<RadioItemState<T>> for SelectionStateTemplate<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn render(&self, model: &ButtonRenderModel<RadioItemState<T>>, window: &mut Window, cx: &mut App) -> Stateful<Div> {
        let value = model.data.value.clone();
        let content = model.content.clone();
        let bool_model = ButtonRenderModel {
            id: model.id.clone(),
            data: model.data.is_selected,
            content: Arc::new(move |bool_model, cx| {
                let rich_model = ButtonRenderModel {
                    id: bool_model.id.clone(),
                    data: RadioItemState { is_selected: bool_model.data, value: value.clone() },
                    content: content.clone(),
                    kind: bool_model.kind,
                    role: bool_model.role,
                    size: bool_model.size,
                    state: bool_model.state,
                    round: bool_model.round,
                    radius_override: Cell::new(bool_model.radius_override.get()),
                };
                content(&rich_model, cx)
            }),
            kind: model.kind,
            role: model.role,
            size: model.size,
            state: model.state,
            round: model.round,
            radius_override: Cell::new(model.radius_override.get()),
        };

        self.base.render(&bool_model, window, cx)
    }
}

pub fn selection_state_template<T>(base: Arc<dyn ButtonTemplate<bool>>) -> Arc<dyn ButtonTemplate<RadioItemState<T>>>
where
    T: Clone + Send + Sync + 'static,
{
    Arc::new(SelectionStateTemplate { base, _marker: PhantomData })
}

pub fn vertical_group_template<T>() -> RadioGroupTemplate<T>
where
    T: Clone + Eq + 'static,
{
    Arc::new(|group, _window, _cx| {
        div().flex().flex_col().gap_2().children(group.buttons().iter().cloned()).into_any_element()
    })
}

pub fn horizontal_group_template<T>() -> RadioGroupTemplate<T>
where
    T: Clone + Eq + 'static,
{
    Arc::new(|group, _window, _cx| {
        div().flex().items_center().gap_3().children(group.buttons().iter().cloned()).into_any_element()
    })
}
