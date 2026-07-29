use std::sync::Arc;

use gpui_luma::controls::textfield::TextFieldState;
use gpui_luma_look_shadcn::{ShadcnLook, ShadcnTextFieldStyle};
use gpui_luma_look_shadcn_inspect::ShadcnInspect;

use super::inspector::input::{floating_menu_color_rows, textfield_color_rows_prefixed};
use super::inspector::metrics::textfield_menu_layout_section;
use super::inspector::{
    ControlInspectorResolver, InspectColorRow, InspectorCategoryContent, InspectorSelection, SharedInspectorResolver,
};

pub struct TextFieldMenuInspectorAdapter {
    diagram_id: &'static str,
}

impl TextFieldMenuInspectorAdapter {
    pub fn shared(diagram_id: &'static str) -> SharedInspectorResolver {
        Arc::new(Self { diagram_id })
    }
}

impl ControlInspectorResolver for TextFieldMenuInspectorAdapter {
    fn resolve_category(
        &self,
        look: &ShadcnLook,
        selection: InspectorSelection<'_>,
        category_id: &str,
    ) -> InspectorCategoryContent {
        match category_id {
            "color" => InspectorCategoryContent::Colors(resolve_color_rows(look)),
            "layout" => InspectorCategoryContent::Layout(textfield_menu_layout_section(
                look,
                self.diagram_id,
                selection.size_id,
            )),
            _ => InspectorCategoryContent::Colors(Vec::new()),
        }
    }
}

fn resolve_color_rows(look: &ShadcnLook) -> Vec<InspectColorRow> {
    let palette = ShadcnInspect::new(look).inspect_textfield_color_palette(
        ShadcnTextFieldStyle::Input,
        TextFieldState::default(),
        true,
    );
    let mut rows = textfield_color_rows_prefixed("textfield", &palette);
    rows.extend(floating_menu_color_rows(look));
    rows
}
