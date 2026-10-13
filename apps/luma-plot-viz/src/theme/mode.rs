use gpui::Context;
use gpui_luma::controls::radio_group::{self, RadioGroup, RadioGroupItem};
use gpui_luma::theme::ThemeMode;
use gpui_luma_look_radix::Look;

pub(crate) fn id(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

pub(crate) fn mode_for_id(id: &str) -> ThemeMode {
    if id == "light" {
        ThemeMode::Light
    } else {
        ThemeMode::Dark
    }
}

pub(crate) fn spawn<M: 'static>(id: &'static str, look: &Look, cx: &mut Context<M>) -> RadioGroup<RadioGroupItem> {
    radio_group::horizontal(id)
        .single_required()
        .selection_follows_active(true)
        .items([RadioGroupItem::new("light").label("Light"), RadioGroupItem::new("dark").label("Dark")])
        .selected(self::id(look.mode()))
        .template(gpui_luma_look_radix::segmented_radio_template(look))
        .spawn(cx)
}
