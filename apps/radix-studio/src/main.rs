#![windows_subsystem = "windows"]

mod app;
mod app_shell;
mod assets;
mod color_hex;
mod controls;
mod screens;
mod tabs;

use assets::Assets;
use gpui::{actions, App, KeyBinding, Menu, MenuItem};

actions!(radix_studio_app, [Quit]);

fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(|cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("Radix Studio").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = luma::init(cx).and_then(|_| {
            luma::focus::bind_default_focus_keys(cx);
            luma::key_handling::bind_default_control_keys(cx);
            app_shell::open(cx)
        }) {
            eprintln!("failed to open Radix Studio: {error:?}");
        }
    });
}
