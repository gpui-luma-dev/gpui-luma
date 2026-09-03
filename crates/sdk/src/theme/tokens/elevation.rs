use gpui::{BoxShadow, Hsla, hsla, point, px};

#[derive(Clone, Debug)]
pub struct LumaElevation {
    pub none: LumaShadow,
    pub control: LumaShadow,
    pub thumb: LumaShadow,
    pub menu: LumaShadow,
    pub popover: LumaShadow,
    pub panel: LumaShadow,
    pub dialog: LumaShadow,
}

#[derive(Clone, Debug, Default)]
pub struct LumaShadow {
    pub layers: Vec<LumaShadowLayer>,
}

#[derive(Clone, Copy, Debug)]
pub struct LumaShadowLayer {
    pub color: Hsla,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

impl Default for LumaElevation {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaElevation {
    pub fn light() -> Self {
        Self {
            none: LumaShadow::default(),
            control: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.05), 0.0, 1.0, 2.0, 0.0),
            thumb: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.14), 0.0, 1.0, 2.0, 0.0),
            menu: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.10), 0.0, 1.0, 3.0, 0.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.10), 0.0, 1.0, 2.0, -1.0),
            ]),
            popover: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.12), 0.0, 4.0, 8.0, -2.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.06), 0.0, 2.0, 4.0, -1.0),
            ]),
            panel: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.06), 0.0, 1.0, 3.0, 0.0),
            dialog: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.16), 0.0, 18.0, 32.0, -8.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.08), 0.0, 6.0, 12.0, -4.0),
            ]),
        }
    }

    pub fn dark() -> Self {
        Self {
            none: LumaShadow::default(),
            control: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.18), 0.0, 1.0, 2.0, 0.0),
            thumb: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.28), 0.0, 1.0, 2.0, 0.0),
            menu: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.28), 0.0, 6.0, 16.0, -6.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.18), 0.0, 2.0, 6.0, -2.0),
            ]),
            popover: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.34), 0.0, 10.0, 24.0, -8.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.18), 0.0, 2.0, 6.0, -2.0),
            ]),
            panel: LumaShadow::single(hsla(0.0, 0.0, 0.0, 0.20), 0.0, 1.0, 3.0, 0.0),
            dialog: LumaShadow::new(vec![
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.42), 0.0, 18.0, 32.0, -8.0),
                LumaShadowLayer::new(hsla(0.0, 0.0, 0.0, 0.24), 0.0, 6.0, 12.0, -4.0),
            ]),
        }
    }
}

impl LumaShadow {
    pub fn new(layers: Vec<LumaShadowLayer>) -> Self {
        Self { layers }
    }

    pub fn single(color: Hsla, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self::new(vec![LumaShadowLayer::new(color, offset_x, offset_y, blur, spread)])
    }

    pub fn to_box_shadows(&self) -> Vec<BoxShadow> {
        self.layers.iter().map(LumaShadowLayer::to_box_shadow).collect()
    }
}

impl LumaShadowLayer {
    pub fn new(color: Hsla, offset_x: f32, offset_y: f32, blur: f32, spread: f32) -> Self {
        Self { color, offset_x, offset_y, blur, spread }
    }

    pub fn to_box_shadow(&self) -> BoxShadow {
        BoxShadow {
            color: self.color,
            offset: point(px(self.offset_x), px(self.offset_y)),
            blur_radius: px(self.blur),
            spread_radius: px(self.spread),
            inset: false,
        }
    }
}
