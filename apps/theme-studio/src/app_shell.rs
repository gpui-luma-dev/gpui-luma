use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};

use gpui_luma::shell::TitleBar;

use crate::studio::ThemeStudioApp;
use crate::theme::StudioLaunchOptions;

const DEFAULT_WINDOW_WIDTH_PX: f32 = 1600.0;
const DEFAULT_WINDOW_HEIGHT_PX: f32 = 1000.0;

pub fn open(cx: &mut App, launch_options: StudioLaunchOptions) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, size(px(DEFAULT_WINDOW_WIDTH_PX), px(DEFAULT_WINDOW_HEIGHT_PX)), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| ThemeStudioApp::new(window, cx, launch_options)),
    )?;

    cx.activate(true);
    Ok(())
}
