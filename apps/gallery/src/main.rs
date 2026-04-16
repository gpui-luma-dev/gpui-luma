mod app_shell;
mod gallery;

use gpui_platform::application;

fn main() {
    application().run(|cx| {
        if let Err(error) = gpui_luma::init(cx).and_then(|_| app_shell::open(cx)) {
            eprintln!("failed to open GPUI-Luma gallery: {error:?}");
        }
    });
}
