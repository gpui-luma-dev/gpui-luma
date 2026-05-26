mod app_shell;
mod gallery;
#[path = "assets/assets.rs"]
mod assets;
#[path = "assets/fonts.rs"]
mod fonts;

use assets::Assets;
use gallery::GalleryThemeChoice;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};

actions!(gallery_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let theme_choice = GalleryThemeChoice::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("GPUI-Luma Gallery").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = gpui_luma::init(cx).and_then(|_| {
            gpui_luma::focus::bind_default_focus_keys(cx);
            gpui_luma::keyhandling::bind_default_control_keys(cx);
            if theme_choice.loads_rajdhani_font() {
                fonts::load_rajdhani(cx)?;
            }
            app_shell::open(cx, theme_choice)
        }) {
            eprintln!("failed to open GPUI-Luma gallery: {error:?}");
        }
    });
}
