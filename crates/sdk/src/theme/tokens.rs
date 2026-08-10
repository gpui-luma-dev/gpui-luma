use gpui::{BoxShadow, FontWeight, Hsla, hsla, point, px, rgb};
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ControlSize {
    Sm,
    #[default]
    Md,
    Lg,
}

#[derive(Clone, Debug)]
pub struct LumaTheme {
    pub name: String,
    pub version: u32,
    pub modes: ThemeModes,
}

#[derive(Clone, Debug)]
pub struct LumaThemeMode {
    pub palette: LumaPalette,
    pub metrics: MetricTokens,
    pub typography: LumaTypography,
    pub elevation: LumaElevation,
}

pub type ThemeTokens = LumaThemeMode;

#[derive(Clone, Debug)]
pub struct ThemeModes {
    pub light: LumaThemeMode,
    pub dark: LumaThemeMode,
}

#[derive(Clone, Debug)]
pub struct LumaPalette {
    pub app: AppPalette,
    pub surface: SurfacePalette,
    pub state: StatePalette,
    pub form: FormPalette,
    pub focus: FocusPalette,
    pub border: BorderPalette,
    pub navigation: NavigationPalette,
    pub data: DataPalette,
}

#[derive(Clone, Copy, Debug)]
pub struct AppPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfacePalette {
    pub panel: SurfaceWithBorderPalette,
    pub floating: SurfaceWithBorderPalette,
    pub subtle: SurfaceTonePalette,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceWithBorderPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceTonePalette {
    pub background: Hsla,
    pub foreground: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct StatePalette {
    pub hover: StateTonePalette,
    pub pressed: StateBackgroundPalette,
    pub selected: StateTonePalette,
    pub disabled: StateTonePalette,
}

#[derive(Clone, Copy, Debug)]
pub struct StateTonePalette {
    pub background: Hsla,
    pub foreground: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct StateBackgroundPalette {
    pub background: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct FormPalette {
    pub input: FormInputPalette,
}

#[derive(Clone, Copy, Debug)]
pub struct FormInputPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub border: Hsla,
    pub invalid_border: Hsla,
    pub placeholder: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusPalette {
    pub ring: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderPalette {
    pub default: Hsla,
    pub strong: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct NavigationPalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    pub hover_background: Hsla,
    pub selected_background: Hsla,
    pub selected_foreground: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct DataPalette {
    pub accent_1: Hsla,
    pub accent_2: Hsla,
    pub accent_3: Hsla,
    pub accent_4: Hsla,
    pub accent_5: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct MetricTokens {
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub border_width: BorderWidthTokens,
    pub focus: FocusMetricTokens,
    pub control: ControlMetricScale,
}

#[derive(Clone, Copy, Debug)]
pub struct SpacingTokens {
    pub s0: f32,
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub s4: f32,
    pub s5: f32,
    pub s6: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct RadiusTokens {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub pill: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct BorderWidthTokens {
    pub hairline: f32,
    pub default: f32,
    pub strong: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct FocusMetricTokens {
    pub width: f32,
    pub offset: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlMetricScale {
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlMetricTokens {
    pub radius: f32,
    pub height: f32,
    pub control_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
    pub icon_size: f32,
}

#[derive(Clone, Debug)]
pub struct LumaTypography {
    pub font: FontTokens,
    pub text: TextTokens,
}

#[derive(Clone, Debug)]
pub struct FontTokens {
    pub sans: FontFamilyToken,
    pub mono: FontFamilyToken,
    pub serif: FontFamilyToken,
}

#[derive(Clone, Debug)]
pub struct FontFamilyToken {
    pub family: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LumaTextRole {
    H1,
    H2,
    H3,
    H4,
    P,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LumaTextScale {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    TwoXl,
}

#[derive(Clone, Copy, Debug)]
pub struct TextScaleTokens {
    pub xs: LumaTextStyle,
    pub sm: LumaTextStyle,
    pub md: LumaTextStyle,
    pub lg: LumaTextStyle,
    pub xl: LumaTextStyle,
    pub two_xl: LumaTextStyle,
}

#[derive(Clone, Copy, Debug)]
pub struct TextRoleTokens {
    pub h1: LumaTextStyle,
    pub h2: LumaTextStyle,
    pub h3: LumaTextStyle,
    pub h4: LumaTextStyle,
    pub p: LumaTextStyle,
}

#[derive(Clone, Copy, Debug)]
pub struct TextTokens {
    pub body: LumaTextStyle,
    pub label: LumaTextStyle,
    pub caption: LumaTextStyle,
    pub title: LumaTextStyle,
    pub code: LumaTextStyle,
    pub scale: TextScaleTokens,
    pub role: TextRoleTokens,
}

#[derive(Clone, Copy, Debug)]
pub struct LumaTextStyle {
    pub size: f32,
    pub line_height: f32,
    pub weight: FontWeight,
}

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

impl LumaTheme {
    pub fn structural_default() -> Self {
        Self {
            name: "Structural".to_string(),
            version: 1,
            modes: ThemeModes::new(LumaThemeMode::light(), LumaThemeMode::dark()),
        }
    }

    pub fn mode(&self, mode: ThemeMode) -> &LumaThemeMode {
        self.modes.tokens(mode)
    }
}

impl Default for LumaTheme {
    fn default() -> Self {
        Self::structural_default()
    }
}

impl LumaThemeMode {
    pub fn light() -> Self {
        Self::new(LumaPalette::light(), MetricTokens::default(), LumaTypography::default(), LumaElevation::light())
    }

    pub fn dark() -> Self {
        Self::new(LumaPalette::dark(), MetricTokens::default(), LumaTypography::default(), LumaElevation::dark())
    }

    pub fn new(
        palette: LumaPalette,
        metrics: MetricTokens,
        typography: LumaTypography,
        elevation: LumaElevation,
    ) -> Self {
        Self { palette, metrics, typography, elevation }
    }
}

impl ThemeModes {
    pub fn new(light: LumaThemeMode, dark: LumaThemeMode) -> Self {
        Self { light, dark }
    }

    pub fn single(tokens: LumaThemeMode) -> Self {
        Self { light: tokens.clone(), dark: tokens }
    }

    pub fn tokens(&self, mode: ThemeMode) -> &LumaThemeMode {
        match mode {
            ThemeMode::Light => &self.light,
            ThemeMode::Dark => &self.dark,
        }
    }

    pub fn tokens_mut(&mut self, mode: ThemeMode) -> &mut LumaThemeMode {
        match mode {
            ThemeMode::Light => &mut self.light,
            ThemeMode::Dark => &mut self.dark,
        }
    }

    pub fn into_tokens(self, mode: ThemeMode) -> LumaThemeMode {
        match mode {
            ThemeMode::Light => self.light,
            ThemeMode::Dark => self.dark,
        }
    }
}

impl Default for ThemeModes {
    fn default() -> Self {
        Self::new(LumaThemeMode::light(), LumaThemeMode::dark())
    }
}

impl Default for LumaThemeMode {
    fn default() -> Self {
        Self::light()
    }
}

impl Default for LumaPalette {
    fn default() -> Self {
        Self::light()
    }
}

impl LumaPalette {
    pub fn light() -> Self {
        Self {
            app: AppPalette {
                background: rgb(0xf8fafc).into(),
                foreground: rgb(0x0f172a).into(),
                muted_foreground: rgb(0x64748b).into(),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: rgb(0xffffff).into(),
                    foreground: rgb(0x0f172a).into(),
                    border: rgb(0xcbd5e1).into(),
                },
                floating: SurfaceWithBorderPalette {
                    background: rgb(0xffffff).into(),
                    foreground: rgb(0x0f172a).into(),
                    border: rgb(0xcbd5e1).into(),
                },
                subtle: SurfaceTonePalette { background: rgb(0xf1f5f9).into(), foreground: rgb(0x334155).into() },
            },
            state: StatePalette {
                hover: StateTonePalette { background: rgb(0xe2e8f0).into(), foreground: rgb(0x0f172a).into() },
                pressed: StateBackgroundPalette { background: rgb(0xcbd5e1).into() },
                selected: StateTonePalette { background: rgb(0x2563eb).into(), foreground: rgb(0xffffff).into() },
                disabled: StateTonePalette { background: rgb(0xf1f5f9).into(), foreground: rgb(0x94a3b8).into() },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: rgb(0xffffff).into(),
                    foreground: rgb(0x0f172a).into(),
                    border: rgb(0x94a3b8).into(),
                    invalid_border: rgb(0xdb2777).into(),
                    placeholder: rgb(0x94a3b8).into(),
                },
            },
            focus: FocusPalette { ring: rgb(0xf59e0b).into() },
            border: BorderPalette { default: rgb(0xcbd5e1).into(), strong: rgb(0x64748b).into() },
            navigation: NavigationPalette {
                background: rgb(0xffffff).into(),
                foreground: rgb(0x0f172a).into(),
                muted_foreground: rgb(0x64748b).into(),
                hover_background: rgb(0xf1f5f9).into(),
                selected_background: rgb(0x2563eb).into(),
                selected_foreground: rgb(0xffffff).into(),
                border: rgb(0xcbd5e1).into(),
            },
            data: DataPalette {
                accent_1: rgb(0x2563eb).into(),
                accent_2: rgb(0x0f766e).into(),
                accent_3: rgb(0xc2410c).into(),
                accent_4: rgb(0x7c3aed).into(),
                accent_5: rgb(0xbe123c).into(),
            },
        }
    }

    pub fn dark() -> Self {
        Self {
            app: AppPalette {
                background: rgb(0x0b1120).into(),
                foreground: rgb(0xf8fafc).into(),
                muted_foreground: rgb(0x94a3b8).into(),
            },
            surface: SurfacePalette {
                panel: SurfaceWithBorderPalette {
                    background: rgb(0x111827).into(),
                    foreground: rgb(0xf8fafc).into(),
                    border: rgb(0x334155).into(),
                },
                floating: SurfaceWithBorderPalette {
                    background: rgb(0x111827).into(),
                    foreground: rgb(0xf8fafc).into(),
                    border: rgb(0x334155).into(),
                },
                subtle: SurfaceTonePalette { background: rgb(0x1e293b).into(), foreground: rgb(0xcbd5e1).into() },
            },
            state: StatePalette {
                hover: StateTonePalette { background: rgb(0x334155).into(), foreground: rgb(0xf8fafc).into() },
                pressed: StateBackgroundPalette { background: rgb(0x475569).into() },
                selected: StateTonePalette { background: rgb(0x60a5fa).into(), foreground: rgb(0x082f49).into() },
                disabled: StateTonePalette { background: rgb(0x1e293b).into(), foreground: rgb(0x64748b).into() },
            },
            form: FormPalette {
                input: FormInputPalette {
                    background: rgb(0x0f172a).into(),
                    foreground: rgb(0xf8fafc).into(),
                    border: rgb(0x475569).into(),
                    invalid_border: rgb(0xf472b6).into(),
                    placeholder: rgb(0x64748b).into(),
                },
            },
            focus: FocusPalette { ring: rgb(0xfbbf24).into() },
            border: BorderPalette { default: rgb(0x334155).into(), strong: rgb(0x94a3b8).into() },
            navigation: NavigationPalette {
                background: rgb(0x111827).into(),
                foreground: rgb(0xf8fafc).into(),
                muted_foreground: rgb(0x94a3b8).into(),
                hover_background: rgb(0x1e293b).into(),
                selected_background: rgb(0x60a5fa).into(),
                selected_foreground: rgb(0x082f49).into(),
                border: rgb(0x334155).into(),
            },
            data: DataPalette {
                accent_1: rgb(0x60a5fa).into(),
                accent_2: rgb(0x2dd4bf).into(),
                accent_3: rgb(0xfb923c).into(),
                accent_4: rgb(0xa78bfa).into(),
                accent_5: rgb(0xfb7185).into(),
            },
        }
    }
}

impl Default for MetricTokens {
    fn default() -> Self {
        let sm = ControlMetricTokens::new(28.0, 10.0, 5.0, 6.0, 5.0, 14.0);
        let md = ControlMetricTokens::new(36.0, 14.0, 8.0, 8.0, 6.0, 16.0);
        let lg = ControlMetricTokens::new(44.0, 18.0, 10.0, 10.0, 8.0, 18.0);

        Self {
            spacing: SpacingTokens { s0: 0.0, s1: 4.0, s2: 6.0, s3: 8.0, s4: 12.0, s5: 16.0, s6: 24.0 },
            radius: RadiusTokens { none: 0.0, sm: 3.0, md: 6.0, lg: 8.0, xl: 12.0, pill: 999.0 },
            border_width: BorderWidthTokens { hairline: 0.5, default: 1.0, strong: 2.0 },
            focus: FocusMetricTokens { width: 1.0, offset: 0.0 },
            control: ControlMetricScale { sm, md, lg },
        }
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32, icon_size: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap, icon_size }
    }
}

impl MetricTokens {
    pub fn for_size(&self, size: ControlSize) -> ControlMetricTokens {
        match size {
            ControlSize::Sm => self.control.sm,
            ControlSize::Md => self.control.md,
            ControlSize::Lg => self.control.lg,
        }
    }

    pub fn radius(&self, size: ControlSize) -> f32 {
        self.for_size(size).radius
    }

    pub fn control_height(&self, size: ControlSize) -> f32 {
        self.for_size(size).control_height
    }

    pub fn padding_x(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_x
    }

    pub fn padding_y(&self, size: ControlSize) -> f32 {
        self.for_size(size).padding_y
    }

    pub fn gap(&self, size: ControlSize) -> f32 {
        self.for_size(size).gap
    }

    pub fn icon_size(&self, size: ControlSize) -> f32 {
        self.for_size(size).icon_size
    }
}

impl Default for LumaTypography {
    fn default() -> Self {
        let scale = TextScaleTokens {
            xs: LumaTextStyle { size: 11.0, line_height: 16.0, weight: FontWeight::MEDIUM },
            sm: LumaTextStyle { size: 12.5, line_height: 18.0, weight: FontWeight::NORMAL },
            md: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
            lg: LumaTextStyle { size: 16.0, line_height: 22.0, weight: FontWeight::MEDIUM },
            xl: LumaTextStyle { size: 18.0, line_height: 24.0, weight: FontWeight::SEMIBOLD },
            two_xl: LumaTextStyle { size: 20.0, line_height: 28.0, weight: FontWeight::SEMIBOLD },
        };
        let role = TextRoleTokens {
            h1: LumaTextStyle { size: 44.0, line_height: 52.0, weight: FontWeight::BOLD },
            h2: LumaTextStyle { size: 28.0, line_height: 36.0, weight: FontWeight::SEMIBOLD },
            h3: scale.two_xl,
            h4: scale.lg,
            p: scale.md,
        };

        Self {
            font: FontTokens {
                sans: FontFamilyToken { family: "System UI".to_string() },
                mono: FontFamilyToken { family: "Monaco".to_string() },
                serif: FontFamilyToken { family: "New York".to_string() },
            },
            text: TextTokens {
                body: role.p,
                label: LumaTextStyle {
                    size: scale.sm.size,
                    line_height: scale.sm.line_height,
                    weight: FontWeight::MEDIUM,
                },
                caption: scale.xs,
                title: role.h3,
                code: LumaTextStyle { size: 13.0, line_height: 18.0, weight: FontWeight::NORMAL },
                scale,
                role,
            },
        }
    }
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

impl TextScaleTokens {
    pub fn style(&self, scale: LumaTextScale) -> LumaTextStyle {
        match scale {
            LumaTextScale::Xs => self.xs,
            LumaTextScale::Sm => self.sm,
            LumaTextScale::Md => self.md,
            LumaTextScale::Lg => self.lg,
            LumaTextScale::Xl => self.xl,
            LumaTextScale::TwoXl => self.two_xl,
        }
    }
}

impl TextRoleTokens {
    pub fn style(&self, role: LumaTextRole) -> LumaTextStyle {
        match role {
            LumaTextRole::H1 => self.h1,
            LumaTextRole::H2 => self.h2,
            LumaTextRole::H3 => self.h3,
            LumaTextRole::H4 => self.h4,
            LumaTextRole::P => self.p,
        }
    }
}

impl TextTokens {
    pub fn scale(&self, scale: LumaTextScale) -> LumaTextStyle {
        self.scale.style(scale)
    }

    pub fn role(&self, role: LumaTextRole) -> LumaTextStyle {
        self.role.style(role)
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

#[cfg(test)]
mod tests {
    use super::{LumaTheme, LumaThemeMode, ThemeMode, ThemeModes, ThemeTokens};

    #[test]
    fn default_tokens_use_light_structural_palette() {
        assert_eq!(LumaThemeMode::light().palette.app.background, ThemeTokens::default().palette.app.background);
    }

    #[test]
    fn theme_modes_select_light_and_dark_palettes() {
        let modes = ThemeModes::default();
        assert_ne!(
            modes.tokens(ThemeMode::Light).palette.app.background,
            modes.tokens(ThemeMode::Dark).palette.app.background
        );
    }

    #[test]
    fn structural_default_theme_exposes_semantic_layers() {
        let theme = LumaTheme::structural_default();
        let light = theme.mode(ThemeMode::Light);
        assert_eq!(theme.name, "Structural");
        assert_eq!(theme.version, 1);
        assert_eq!(light.metrics.radius.pill, 999.0);
        assert_eq!(light.typography.text.label.weight, gpui::FontWeight::MEDIUM);
        assert!(!light.elevation.menu.layers.is_empty());
    }
}
