use gpui::Pixels;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::gallery) enum CompositionSize {
    Sm,
    #[default]
    Md,
    Lg,
    Custom(Pixels),
}

impl CompositionSize {
    pub fn resolve_primary(self, sm: f32, old_md: f32, _old_lg: f32) -> f32 {
        let new_sm = sm * 0.8;
        let new_md = new_sm + (old_md - new_sm) * 0.5;
        match self {
            Self::Sm => new_sm,
            Self::Md => new_md,
            Self::Lg => old_md,
            Self::Custom(px) => px.as_f32(),
        }
    }
}
