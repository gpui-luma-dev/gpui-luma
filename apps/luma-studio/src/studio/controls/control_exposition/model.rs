#[derive(Clone, Copy, Debug)]
pub struct ControlExpositionLayout {
    pub borderless: bool,
}

impl ControlExpositionLayout {
    pub const BORDERLESS: Self = Self { borderless: true };
    pub const CARD: Self = Self { borderless: false };
}

pub const EVENT_SECTION_INDENT: f32 = 100.0;

#[derive(Clone, Copy)]
pub struct EventReferenceSpec {
    pub event: &'static str,
    pub trigger: &'static str,
    pub notes: &'static str,
}
