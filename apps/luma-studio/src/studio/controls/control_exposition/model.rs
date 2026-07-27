#[derive(Clone, Copy, Debug)]
pub struct ControlExpositionLayout {
    pub borderless: bool,
}

impl ControlExpositionLayout {
    pub const BORDERLESS: Self = Self { borderless: true };
}

#[derive(Clone, Copy)]
pub struct EventReferenceSpec {
    pub event: &'static str,
    pub trigger: &'static str,
    pub notes: &'static str,
}

#[derive(Clone, Copy)]
pub struct PublicInterfaceSpec {
    pub symbol: &'static str,
    pub surface: &'static str,
    pub notes: &'static str,
}
