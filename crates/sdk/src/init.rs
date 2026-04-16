use std::borrow::Cow;

use gpui::App;
use lucide_icons::LUCIDE_FONT_BYTES;

pub fn init(cx: &mut App) {
    if let Err(error) = cx
        .text_system()
        .add_fonts(vec![Cow::Borrowed(LUCIDE_FONT_BYTES)])
    {
        eprintln!("failed to load Lucide icon font: {error:?}");
    }
}
