#[derive(Clone, Copy, Debug)]
pub struct ControlExpositionLayout {
    pub borderless: bool,
    pub skip_heading: bool,
    pub skip_snippet: bool,
}

impl ControlExpositionLayout {
    pub const BORDERLESS: Self = Self { borderless: true, skip_heading: false, skip_snippet: false };
    pub const BORDERLESS_NO_HEADING: Self = Self { skip_heading: true, ..Self::BORDERLESS };
}
