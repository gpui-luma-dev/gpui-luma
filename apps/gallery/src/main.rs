mod app_shell;
mod gallery;

use gpui_platform::application;

fn main() {
    application().run(|cx| {
        if let Err(error) = gpui_luma::init(cx).and_then(|_| {
            gpui_luma::focus::bind_default_focus_keys(cx);
            gpui_luma::keyhandling::bind_default_control_keys(cx);
            app_shell::open(cx)
        }) {
            eprintln!("failed to open GPUI-Luma gallery: {error:?}");
        }
    });
}
