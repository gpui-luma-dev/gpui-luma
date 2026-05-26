use std::sync::Arc;

use gpui::AnyElement;

use super::button_item_template::button_item_template;
use super::model::ControlGroupItemLike;
use super::template::ControlGroupItemTemplate;
use crate::controls::button_family::{ButtonFamilyRole, ButtonKind};
use crate::controls::command::button::ButtonTemplate;

/// Icon/control-group items with [`ButtonFamilyRole::Toggle`].
///
/// Pass the **selected-segment** [`ButtonKind`] (e.g. [`ButtonKind::Standard`] for filled navy);
/// unselected segments resolve as Subtle (outline border from `action.subtle.*` in the theme).
pub fn toggle_button_item_template<T>(
    button_template: Arc<dyn ButtonTemplate<bool>>,
    selected_kind: ButtonKind,
    round: bool,
    content_fn: impl Fn(&T) -> AnyElement + Send + Sync + 'static,
) -> ControlGroupItemTemplate<T>
where
    T: ControlGroupItemLike + Clone + Send + Sync + 'static,
{
    button_item_template(
        button_template,
        selected_kind,
        |selected| ButtonFamilyRole::Toggle { selected },
        round,
        content_fn,
    )
}
