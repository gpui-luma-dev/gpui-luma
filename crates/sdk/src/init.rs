use std::borrow::Cow;

use gpui::App;
use lucide_icons::LUCIDE_FONT_BYTES;

pub fn init(cx: &mut App) -> anyhow::Result<()> {
    cx.text_system()
        .add_fonts(vec![Cow::Borrowed(LUCIDE_FONT_BYTES)])?;

    Ok(())
}
