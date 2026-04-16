mod app_shell;
mod gallery;

use gpui_platform::application;

fn main() {
    application().run(|cx| {
        gpui_luma::init(cx);

        if let Err(error) = app_shell::open(cx) {
            eprintln!("failed to open GPUI-Luma gallery: {error:?}");
        }
    });
}
