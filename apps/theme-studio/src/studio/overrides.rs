use std::collections::HashMap;

use gpui::Hsla;
use gpui_luma::controls::switch::SwitchScale;

use super::inspectable::InspectableId;

#[derive(Clone, Debug, Default)]
pub struct StudioOverrides {
    pub global_color_overrides: HashMap<String, Hsla>,
    pub color_overrides: HashMap<InspectableId, HashMap<String, Hsla>>,
    pub scale_overrides: HashMap<InspectableId, HashMap<String, f32>>,
    pub switch_scale: Option<SwitchScale>,
}

impl StudioOverrides {
    pub fn global_color_override(&self, token: &str) -> Option<Hsla> {
        self.global_color_overrides.get(token).copied()
    }

    pub fn set_global_color(&mut self, token: String, color: Hsla) {
        self.global_color_overrides.insert(token, color);
    }

    pub fn color_override(&self, id: InspectableId, token: &str) -> Option<Hsla> {
        self.global_color_override(token).or_else(|| self.color_overrides.get(&id)?.get(token).copied())
    }

    pub fn scale_override(&self, id: InspectableId, key: &str) -> Option<f32> {
        self.scale_overrides.get(&id)?.get(key).copied()
    }

    pub fn set_scale(&mut self, id: InspectableId, key: String, value: f32) {
        self.scale_overrides.entry(id).or_default().insert(key, value);
    }

    pub fn effective_switch_scale(&self, base: SwitchScale) -> SwitchScale {
        let id = InspectableId::CookieSettings;
        let mut scale = self.switch_scale.unwrap_or(base);

        if let Some(v) = self.scale_override(id, "track_width") {
            scale.track_width = v;
        }
        if let Some(v) = self.scale_override(id, "track_height") {
            scale.track_height = v;
        }
        if let Some(v) = self.scale_override(id, "thumb_size") {
            scale.thumb_size = v;
        }

        scale
    }
}
