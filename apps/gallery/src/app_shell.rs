use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use gpui_luma::shell::TitleBar;

use crate::gallery::{GalleryApp, GalleryThemeChoice};

pub fn open(cx: &mut App, theme_choice: GalleryThemeChoice) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1800.0), px(1200.0)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| GalleryApp::new(window, cx, theme_choice)),
    )?;

    cx.activate(true);
    Ok(())
}
