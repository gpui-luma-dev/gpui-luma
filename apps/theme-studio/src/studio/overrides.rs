use std::collections::HashMap;

use gpui::Hsla;

#[derive(Clone, Debug, Default)]
pub struct StudioOverrides {
    pub global_color_overrides: HashMap<String, Hsla>,
}

impl StudioOverrides {
    pub fn global_color_override(&self, token: &str) -> Option<Hsla> {
        self.global_color_overrides.get(token).copied()
    }

    pub fn set_global_color(&mut self, token: String, color: Hsla) {
        self.global_color_overrides.insert(token, color);
    }
}
