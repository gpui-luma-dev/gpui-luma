use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};

use crate::gallery::GalleryApp;

pub fn open(cx: &mut App) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(1200.0), px(1200.0)), cx);

    cx.open_window(
        WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
        |window, cx| cx.new(|cx| GalleryApp::new(window, cx)),
    )?;

    cx.activate(true);
    Ok(())
}
