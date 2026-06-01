use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions};

use gpui_luma::shell::TitleBar;

use crate::studio::ThemeStudioApp;
use crate::studio::panel_layout_config::load_window_size;
use crate::theme::StudioThemeChoice;

pub fn open(cx: &mut App, theme_choice: StudioThemeChoice) -> anyhow::Result<()> {
    let bounds = Bounds::centered(None, load_window_size(), cx);

    cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitleBar::title_bar_options()),
            ..Default::default()
        },
        |window, cx| cx.new(|cx| ThemeStudioApp::new(window, cx, theme_choice)),
    )?;

    cx.activate(true);
    Ok(())
}
