use std::sync::Arc;

use gpui::AnyElement;

use super::button_item_template::button_item_template;
use super::model::ControlGroupItemLike;
use super::template::ControlGroupItemTemplate;
use crate::controls::button_family::ButtonFamilyRole;
use crate::controls::command::button::ButtonTemplate;

/// Icon/control-group items with [`ButtonFamilyRole::Toggle`].
///
/// Pass a button template whose theme resolves selected vs unselected toggle segments
/// (for example a styled secondary button-family template from the active theme).
pub fn toggle_button_item_template<T>(
    button_template: Arc<dyn ButtonTemplate<bool>>,
    round: bool,
    content_fn: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    button_item_template(button_template, |selected| ButtonFamilyRole::Toggle { selected }, round, content_fn)
}
