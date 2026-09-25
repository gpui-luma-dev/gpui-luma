use anyhow::{Context, bail};
use gpui::Window;
use objc2_app_kit::{NSView, NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

pub fn configure_sidebar_blur(window: &Window) -> anyhow::Result<()> {
    let handle = HasWindowHandle::window_handle(window)?;
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        bail!("expected an AppKit window");
    };

    // SAFETY: GPUI provides a live NSView for this window. This callback runs on
    // the UI thread, and the borrowed view never outlives the window handle.
    let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
    // SAFETY: GPUI owns the parent content view on this same UI thread.
    let content_view = unsafe { view.superview() }.context("missing GPUI content view")?;

    // GPUI owns this full-window, autoresizing effect view and recreates it on
    // activation. The opaque right pane masks it; the left pane retains its tint.
    for child in content_view.subviews() {
        if let Some(effect) = child.downcast_ref::<NSVisualEffectView>() {
            effect.setMaterial(NSVisualEffectMaterial::Sidebar);
            effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
            effect.setState(NSVisualEffectState::Active);
            return Ok(());
        }
    }

    bail!("missing GPUI visual effect view")
}
