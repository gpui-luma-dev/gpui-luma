use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions};

use gpui_luma::shell::TitleBar;

use crate::studio::ThemeStudioApp;
use crate::studio::panel_layout_config::load_window_size;
use crate::theme::StudioLaunchOptions;

pub fn open(cx: &mut App, launch_options: StudioLaunchOptions) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, load_window_size(), cx);

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
