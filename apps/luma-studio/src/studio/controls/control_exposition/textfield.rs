use std::sync::Arc;

use gpui::{Context, Render, Window, div, prelude::*, px};
use gpui_luma::controls::textfield::TextField;
use gpui_luma_look_shadcn::prelude::*;
use gpui_luma_look_shadcn::ShadcnLook;

use crate::studio::controls::catalog::{ControlDocEntry, catalog_entry};
use super::model::ControlExpositionLayout;
use super::template::render_control_exposition_card;

pub struct TextFieldControlExposition {
    look: Arc<ShadcnLook>,
    entry: ControlDocEntry,
    preview: TextField,
}

impl TextFieldControlExposition {
    pub fn new(cx: &mut Context<Self>, look: Arc<ShadcnLook>) -> Self {
        let entry = *catalog_entry("textfield").expect("textfield catalog entry");
        let preview = look
            .textfield("controls-doc-textfield-preview")
            .placeholder("Email address")
            .full_width(true)
            .spawn(cx);

        Self { look, entry, preview }
    }

    pub fn entry(&self) -> ControlDocEntry {
        self.entry
    }

    pub fn sync_look(&mut self, look: Arc<ShadcnLook>, cx: &mut Context<Self>) {
        self.look = look;
        self.preview.update(cx, |_, cx| cx.notify());
        cx.notify();
    }
}

impl Render for TextFieldControlExposition {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        with_look(&self.look, || {
            let preview = div().w_full().max_w(px(360.0)).child(self.preview.clone()).into_any_element();

            render_control_exposition_card(&self.look, self.entry, preview, None, ControlExpositionLayout::BORDERLESS)
        })
    }
}
