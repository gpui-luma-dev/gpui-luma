#![windows_subsystem = "windows"]

mod app;

mod app_shell;
mod compositions;
mod gradient_builder;
mod theme;
mod studio_tabs;
#[path = "assets/assets.rs"]
mod assets;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};
use theme::ColorVizThemeChoice;

actions!(color_viz_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let theme_choice = ColorVizThemeChoice::from_args();
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Color Viz").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = gpui_luma::init(cx).and_then(|_| {
            gpui_luma::focus::bind_default_focus_keys(cx);
            gpui_luma::key_handling::bind_default_control_keys(cx);
            luma_app_common::register_all(cx)?;
            app_shell::open(cx, theme_choice)
        }) {
            eprintln!("failed to open Color Viz: {error:?}");
        }
    });
}
