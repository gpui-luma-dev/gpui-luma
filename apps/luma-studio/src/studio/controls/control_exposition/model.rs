#[derive(Clone, Copy, Debug)]
pub struct ControlExpositionLayout {
    pub borderless: bool,
    pub skip_heading: bool,
}

impl ControlExpositionLayout {
    pub const BORDERLESS: Self = Self { borderless: true, skip_heading: false };
    pub const BORDERLESS_NO_HEADING: Self = Self { borderless: true, skip_heading: true };
}
