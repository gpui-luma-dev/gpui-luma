//! Radix toggle uses button-family variants (selected = solid accent).

use std::sync::Arc;

use luma::controls::button::{ButtonTemplate, DefaultButtonTemplate};
use luma::controls::button_family::ButtonFamilyRole;
use luma::controls::toggle::{ToggleData, apply_toggle_progress_chrome};

use crate::button::{RadixButtonVariant, button_family_theme};
use crate::look::RadixLook;

pub fn toggle_template(look: Arc<RadixLook>) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_for_variant(look, RadixButtonVariant::Soft)
}

pub fn page_toggle_template(look: Arc<RadixLook>) -> Arc<dyn ButtonTemplate<ToggleData>> {
    toggle_template_for_variant(look, RadixButtonVariant::Page)
}

fn toggle_template_for_variant(
    look: Arc<RadixLook>,
    variant: RadixButtonVariant,
) -> Arc<dyn ButtonTemplate<ToggleData>> {
    let family = button_family_theme(look, variant);
    let theme_for_mod = Arc::clone(&family);
    Arc::new(DefaultButtonTemplate::<ToggleData>::new(family).with_modifier(move |element, model| {
        if model.look.is_some() {
            return element;
        }
        let off = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: false }, model.size, model.state);
        let on = theme_for_mod.resolve(ButtonFamilyRole::Toggle { selected: true }, model.size, model.state);
        apply_toggle_progress_chrome(element, &off, &on, model.data.progress)
    }))
}
