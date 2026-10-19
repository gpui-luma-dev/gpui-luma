mod app;
mod catalog;
use gpui::{App, AppContext, Bounds, KeyBinding, Menu, MenuItem, WindowBounds, WindowOptions, actions, px, size};
use gpui_luma::shell::TitleBar;
actions!(sdk_controls, [Quit]);
fn quit(_: &Quit, cx: &mut App) {
    cx.quit();
}
fn main() {
    gpui_platform::application().run(|cx| {
        cx.on_action(quit);
        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.set_menus([Menu::new("SDK Controls").items([MenuItem::action("Quit", Quit)])]);
        if let Err(error) = open(cx) {
            eprintln!("failed to open SDK Controls: {error:?}");
        }
    });
}
fn open(cx: &mut App) -> anyhow::Result<()> {
    gpui_luma::init(cx)?;
    gpui_luma::focus::bind_default_focus_keys(cx);
    gpui_luma::key_handling::bind_default_control_keys(cx);
    let bounds = Bounds::centered(None, size(px(1350.0), px(900.0)), cx);
    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(900.0), px(600.0))),
            titlebar: Some(TitleBar::title_bar_options()),
            app_owns_titlebar_drag: true,
            ..Default::default()
        },
        |window, cx| cx.new(|cx| app::SdkControlsApp::new(window, cx)),
    )?;
    cx.activate(true);
    Ok(())
}
