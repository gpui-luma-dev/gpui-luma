use std::sync::Arc;

use gpui::SharedString;
use gpui::prelude::*;

use crate::controls::command::button::{ButtonBuilder, ButtonTemplate, DefaultButtonTemplate};
use crate::controls::button_family::{ButtonFamilyRole, button_variant, default_button_family_theme};

pub struct Toggle;

pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
    ButtonBuilder::new(id).data(false).template(default_toggle_template())
}

impl Toggle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: impl Into<SharedString>) -> ButtonBuilder<bool> {
        new(id)
    }
}

pub fn default_toggle_template() -> Arc<dyn ButtonTemplate<bool>> {
    let button_family_theme = default_button_family_theme();
    Arc::new(DefaultButtonTemplate::new(button_family_theme.clone()).with_modifier(move |element, model| {
        let appearance = button_family_theme.resolve(
            button_variant(model.kind),
            ButtonFamilyRole::Toggle { selected: model.data },
            model.size,
            model.state,
        );
        element.bg(appearance.background).text_color(appearance.foreground).border_color(appearance.border)
    }))
}
