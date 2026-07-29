#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod app_shell;
#[path = "assets/assets.rs"]
mod assets;
#[path = "assets/fonts.rs"]
mod fonts;
mod studio;
mod theme;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};
use theme::LumaStudioLaunchOptions;

actions!(luma_studio_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let launch_options = LumaStudioLaunchOptions::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Luma Studio").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = gpui_luma::init(cx).and_then(|_| {
            gpui_luma::focus::bind_default_focus_keys(cx);
            gpui_luma::keyhandling::bind_default_control_keys(cx);
            if launch_options.loads_rajdhani_font() {
                fonts::load_rajdhani(cx)?;
            }
            app_shell::open(cx, launch_options.clone())
        }) {
            eprintln!("failed to open Luma Studio: {error:?}");
        }
    });
}
