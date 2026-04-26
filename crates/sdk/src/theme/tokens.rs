use anyhow::{Context as _, anyhow};
use gpui::{BoxShadow, FontWeight, Hsla, hsla, point, px, rgb};
use serde::Deserialize;

pub const DEFAULT_THEME_TOML: &str = include_str!("default-theme.toml");

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    pub colors: ColorTokens,
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
    pub action: ActionPalette,
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
pub struct ActionPalette {
    pub primary: ActionRolePalette,
    pub secondary: ActionRolePalette,
}

#[derive(Clone, Copy, Debug)]
pub struct ActionRolePalette {
    pub background: Hsla,
    pub foreground: Hsla,
    pub hover_background: Hsla,
    pub pressed_background: Hsla,
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
pub struct ColorTokens {
    pub surface: Hsla,
    pub surface_hover: Hsla,
    pub surface_pressed: Hsla,
    pub surface_disabled: Hsla,
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_pressed: Hsla,
    pub selected: Hsla,
    pub selected_hover: Hsla,
    pub selected_pressed: Hsla,
    pub text: Hsla,
    pub text_inverse: Hsla,
    pub text_disabled: Hsla,
    pub border: Hsla,
}

#[derive(Clone, Copy, Debug)]
pub struct MetricTokens {
    pub spacing: SpacingTokens,
    pub radius: RadiusTokens,
    pub border_width: BorderWidthTokens,
    pub focus: FocusMetricTokens,
    pub control: ControlMetricScale,
    pub sm: ControlMetricTokens,
    pub md: ControlMetricTokens,
    pub lg: ControlMetricTokens,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct SpacingTokens {
    pub s0: f32,
    pub s1: f32,
    pub s2: f32,
    pub s3: f32,
    pub s4: f32,
    pub s5: f32,
    pub s6: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct RadiusTokens {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub pill: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct BorderWidthTokens {
    pub hairline: f32,
    pub default: f32,
    pub strong: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
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

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct ControlMetricTokens {
    pub radius: f32,
    pub height: f32,
    pub control_height: f32,
    pub padding_x: f32,
    pub padding_y: f32,
    pub gap: f32,
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

#[derive(Clone, Debug, Deserialize)]
pub struct FontFamilyToken {
    pub family: String,
}

#[derive(Clone, Copy, Debug)]
pub struct TextTokens {
    pub body: LumaTextStyle,
    pub label: LumaTextStyle,
    pub caption: LumaTextStyle,
    pub title: LumaTextStyle,
    pub code: LumaTextStyle,
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
    pub fn native() -> Self {
        Self::load_default().expect("embedded Luma default theme TOML should parse")
    }

    pub fn load_default() -> anyhow::Result<Self> {
        Self::from_toml_str(DEFAULT_THEME_TOML)
    }

    pub fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        RawTheme::from_toml_str(source)?.try_into_theme()
    }

    pub fn mode(&self, mode: ThemeMode) -> &LumaThemeMode {
        self.modes.tokens(mode)
    }
}

impl Default for LumaTheme {
    fn default() -> Self {
        Self::native()
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
        let colors = ColorTokens::from_palette(&palette);

        Self { palette, metrics, typography, elevation, colors }
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
        Self { light: LumaThemeMode::light(), dark: LumaThemeMode::dark() }
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
            action: ActionPalette {
                primary: ActionRolePalette {
                    background: rgb(0x2563eb).into(),
                    foreground: rgb(0xffffff).into(),
                    hover_background: rgb(0x1d4ed8).into(),
                    pressed_background: rgb(0x1e40af).into(),
                },
                secondary: ActionRolePalette {
                    background: rgb(0xf8fafc).into(),
                    foreground: rgb(0x0f172a).into(),
                    hover_background: rgb(0xe2e8f0).into(),
                    pressed_background: rgb(0xcbd5e1).into(),
                },
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
            action: ActionPalette {
                primary: ActionRolePalette {
                    background: rgb(0x60a5fa).into(),
                    foreground: rgb(0x082f49).into(),
                    hover_background: rgb(0x93c5fd).into(),
                    pressed_background: rgb(0xbfdbfe).into(),
                },
                secondary: ActionRolePalette {
                    background: rgb(0x1e293b).into(),
                    foreground: rgb(0xf8fafc).into(),
                    hover_background: rgb(0x334155).into(),
                    pressed_background: rgb(0x475569).into(),
                },
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

impl Default for ColorTokens {
    fn default() -> Self {
        Self::light()
    }
}

impl ColorTokens {
    pub fn light() -> Self {
        Self::from_palette(&LumaPalette::light())
    }

    pub fn dark() -> Self {
        Self::from_palette(&LumaPalette::dark())
    }

    pub fn from_palette(palette: &LumaPalette) -> Self {
        Self {
            surface: palette.action.secondary.background,
            surface_hover: palette.action.secondary.hover_background,
            surface_pressed: palette.action.secondary.pressed_background,
            surface_disabled: palette.state.disabled.background,
            primary: palette.action.primary.background,
            primary_hover: palette.action.primary.hover_background,
            primary_pressed: palette.action.primary.pressed_background,
            selected: palette.state.selected.background,
            selected_hover: palette.action.primary.hover_background,
            selected_pressed: palette.action.primary.pressed_background,
            text: palette.app.foreground,
            text_inverse: palette.state.selected.foreground,
            text_disabled: palette.state.disabled.foreground,
            border: palette.border.default,
        }
    }
}

impl Default for MetricTokens {
    fn default() -> Self {
        let sm = ControlMetricTokens::new(28.0, 10.0, 5.0, 6.0, 5.0);
        let md = ControlMetricTokens::new(36.0, 14.0, 8.0, 8.0, 6.0);
        let lg = ControlMetricTokens::new(44.0, 18.0, 10.0, 10.0, 8.0);

        Self {
            spacing: SpacingTokens { s0: 0.0, s1: 4.0, s2: 6.0, s3: 8.0, s4: 12.0, s5: 16.0, s6: 24.0 },
            radius: RadiusTokens { none: 0.0, sm: 3.0, md: 6.0, lg: 8.0, xl: 12.0, pill: 999.0 },
            border_width: BorderWidthTokens { hairline: 0.5, default: 1.0, strong: 2.0 },
            focus: FocusMetricTokens { width: 1.0, offset: 0.0 },
            control: ControlMetricScale { sm, md, lg },
            sm,
            md,
            lg,
        }
    }
}

impl ControlMetricTokens {
    pub fn new(height: f32, padding_x: f32, padding_y: f32, gap: f32, radius: f32) -> Self {
        Self { radius, height, control_height: height, padding_x, padding_y, gap }
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
}

impl Default for LumaTypography {
    fn default() -> Self {
        Self {
            font: FontTokens {
                sans: FontFamilyToken { family: "System UI".to_string() },
                mono: FontFamilyToken { family: "Monaco".to_string() },
                serif: FontFamilyToken { family: "New York".to_string() },
            },
            text: TextTokens {
                body: LumaTextStyle { size: 14.0, line_height: 20.0, weight: FontWeight::NORMAL },
                label: LumaTextStyle { size: 13.0, line_height: 18.0, weight: FontWeight::MEDIUM },
                caption: LumaTextStyle { size: 11.0, line_height: 14.0, weight: FontWeight::MEDIUM },
                title: LumaTextStyle { size: 20.0, line_height: 28.0, weight: FontWeight::SEMIBOLD },
                code: LumaTextStyle { size: 13.0, line_height: 18.0, weight: FontWeight::NORMAL },
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
        }
    }
}

#[derive(Deserialize)]
struct RawTheme {
    name: String,
    version: u32,
    light: RawThemeMode,
    dark: RawThemeMode,
}

#[derive(Deserialize)]
struct RawThemeMode {
    palette: RawPalette,
    metrics: RawMetricTokens,
    typography: RawTypography,
    elevation: RawElevation,
    colors: RawColorTokens,
}

#[derive(Deserialize)]
struct RawPalette {
    app: RawAppPalette,
    surface: RawSurfacePalette,
    action: RawActionPalette,
    state: RawStatePalette,
    form: RawFormPalette,
    focus: RawFocusPalette,
    border: RawBorderPalette,
    navigation: RawNavigationPalette,
    data: RawDataPalette,
}

#[derive(Deserialize)]
struct RawAppPalette {
    background: String,
    foreground: String,
    muted_foreground: String,
}

#[derive(Deserialize)]
struct RawSurfacePalette {
    panel: RawSurfaceWithBorderPalette,
    floating: RawSurfaceWithBorderPalette,
    subtle: RawSurfaceTonePalette,
}

#[derive(Deserialize)]
struct RawSurfaceWithBorderPalette {
    background: String,
    foreground: String,
    border: String,
}

#[derive(Deserialize)]
struct RawSurfaceTonePalette {
    background: String,
    foreground: String,
}

#[derive(Deserialize)]
struct RawActionPalette {
    primary: RawActionRolePalette,
    secondary: RawActionRolePalette,
}

#[derive(Deserialize)]
struct RawActionRolePalette {
    background: String,
    foreground: String,
    hover_background: String,
    pressed_background: String,
}

#[derive(Deserialize)]
struct RawStatePalette {
    hover: RawStateTonePalette,
    pressed: RawStateBackgroundPalette,
    selected: RawStateTonePalette,
    disabled: RawStateTonePalette,
}

#[derive(Deserialize)]
struct RawStateTonePalette {
    background: String,
    foreground: String,
}

#[derive(Deserialize)]
struct RawStateBackgroundPalette {
    background: String,
}

#[derive(Deserialize)]
struct RawFormPalette {
    input: RawFormInputPalette,
}

#[derive(Deserialize)]
struct RawFormInputPalette {
    background: String,
    foreground: String,
    border: String,
    invalid_border: String,
    placeholder: String,
}

#[derive(Deserialize)]
struct RawFocusPalette {
    ring: String,
}

#[derive(Deserialize)]
struct RawBorderPalette {
    default: String,
    strong: String,
}

#[derive(Deserialize)]
struct RawNavigationPalette {
    background: String,
    foreground: String,
    muted_foreground: String,
    hover_background: String,
    selected_background: String,
    selected_foreground: String,
    border: String,
}

#[derive(Deserialize)]
struct RawDataPalette {
    accent_1: String,
    accent_2: String,
    accent_3: String,
    accent_4: String,
    accent_5: String,
}

#[derive(Deserialize)]
struct RawColorTokens {
    surface: String,
    surface_hover: String,
    surface_pressed: String,
    surface_disabled: String,
    primary: String,
    primary_hover: String,
    primary_pressed: String,
    selected: String,
    selected_hover: String,
    selected_pressed: String,
    text: String,
    text_inverse: String,
    text_disabled: String,
    border: String,
}

#[derive(Deserialize)]
struct RawMetricTokens {
    spacing: SpacingTokens,
    radius: RadiusTokens,
    border_width: BorderWidthTokens,
    focus: FocusMetricTokens,
    control: RawControlMetricScale,
    sm: ControlMetricTokens,
    md: ControlMetricTokens,
    lg: ControlMetricTokens,
}

#[derive(Deserialize)]
struct RawControlMetricScale {
    sm: ControlMetricTokens,
    md: ControlMetricTokens,
    lg: ControlMetricTokens,
}

#[derive(Deserialize)]
struct RawTypography {
    font: RawFontTokens,
    text: RawTextTokens,
}

#[derive(Deserialize)]
struct RawFontTokens {
    sans: FontFamilyToken,
    mono: FontFamilyToken,
    serif: FontFamilyToken,
}

#[derive(Deserialize)]
struct RawTextTokens {
    body: RawTextStyle,
    label: RawTextStyle,
    caption: RawTextStyle,
    title: RawTextStyle,
    code: RawTextStyle,
}

#[derive(Deserialize)]
struct RawTextStyle {
    size: f32,
    line_height: f32,
    weight: f32,
}

#[derive(Deserialize)]
struct RawElevation {
    none: RawShadow,
    control: RawShadow,
    thumb: RawShadow,
    menu: RawShadow,
    popover: RawShadow,
    panel: RawShadow,
    dialog: RawShadow,
}

#[derive(Deserialize)]
struct RawShadow {
    layers: Vec<RawShadowLayer>,
}

#[derive(Deserialize)]
struct RawShadowLayer {
    color: String,
    offset_x: f32,
    offset_y: f32,
    blur: f32,
    spread: f32,
}

impl RawTheme {
    fn from_toml_str(source: &str) -> anyhow::Result<Self> {
        toml::from_str(source).context("failed to parse Luma theme TOML")
    }

    fn try_into_theme(self) -> anyhow::Result<LumaTheme> {
        Ok(LumaTheme {
            name: self.name,
            version: self.version,
            modes: ThemeModes::new(
                self.light.try_into_theme_mode().context("failed to load light theme mode")?,
                self.dark.try_into_theme_mode().context("failed to load dark theme mode")?,
            ),
        })
    }
}

impl RawThemeMode {
    fn try_into_theme_mode(self) -> anyhow::Result<LumaThemeMode> {
        Ok(LumaThemeMode {
            palette: self.palette.try_into_palette()?,
            metrics: self.metrics.into_metrics(),
            typography: self.typography.into_typography(),
            elevation: self.elevation.try_into_elevation()?,
            colors: self.colors.try_into_colors()?,
        })
    }
}

impl RawPalette {
    fn try_into_palette(self) -> anyhow::Result<LumaPalette> {
        Ok(LumaPalette {
            app: self.app.try_into_app()?,
            surface: self.surface.try_into_surface()?,
            action: self.action.try_into_action()?,
            state: self.state.try_into_state()?,
            form: self.form.try_into_form()?,
            focus: self.focus.try_into_focus()?,
            border: self.border.try_into_border()?,
            navigation: self.navigation.try_into_navigation()?,
            data: self.data.try_into_data()?,
        })
    }
}

impl RawAppPalette {
    fn try_into_app(self) -> anyhow::Result<AppPalette> {
        Ok(AppPalette {
            background: parse_hsla(&self.background)?,
            foreground: parse_hsla(&self.foreground)?,
            muted_foreground: parse_hsla(&self.muted_foreground)?,
        })
    }
}

impl RawSurfacePalette {
    fn try_into_surface(self) -> anyhow::Result<SurfacePalette> {
        Ok(SurfacePalette {
            panel: self.panel.try_into_surface_with_border()?,
            floating: self.floating.try_into_surface_with_border()?,
            subtle: self.subtle.try_into_surface_tone()?,
        })
    }
}

impl RawSurfaceWithBorderPalette {
    fn try_into_surface_with_border(self) -> anyhow::Result<SurfaceWithBorderPalette> {
        Ok(SurfaceWithBorderPalette {
            background: parse_hsla(&self.background)?,
            foreground: parse_hsla(&self.foreground)?,
            border: parse_hsla(&self.border)?,
        })
    }
}

impl RawSurfaceTonePalette {
    fn try_into_surface_tone(self) -> anyhow::Result<SurfaceTonePalette> {
        Ok(SurfaceTonePalette { background: parse_hsla(&self.background)?, foreground: parse_hsla(&self.foreground)? })
    }
}

impl RawActionPalette {
    fn try_into_action(self) -> anyhow::Result<ActionPalette> {
        Ok(ActionPalette {
            primary: self.primary.try_into_action_role()?,
            secondary: self.secondary.try_into_action_role()?,
        })
    }
}

impl RawActionRolePalette {
    fn try_into_action_role(self) -> anyhow::Result<ActionRolePalette> {
        Ok(ActionRolePalette {
            background: parse_hsla(&self.background)?,
            foreground: parse_hsla(&self.foreground)?,
            hover_background: parse_hsla(&self.hover_background)?,
            pressed_background: parse_hsla(&self.pressed_background)?,
        })
    }
}

impl RawStatePalette {
    fn try_into_state(self) -> anyhow::Result<StatePalette> {
        Ok(StatePalette {
            hover: self.hover.try_into_state_tone()?,
            pressed: self.pressed.try_into_state_background()?,
            selected: self.selected.try_into_state_tone()?,
            disabled: self.disabled.try_into_state_tone()?,
        })
    }
}

impl RawStateTonePalette {
    fn try_into_state_tone(self) -> anyhow::Result<StateTonePalette> {
        Ok(StateTonePalette { background: parse_hsla(&self.background)?, foreground: parse_hsla(&self.foreground)? })
    }
}

impl RawStateBackgroundPalette {
    fn try_into_state_background(self) -> anyhow::Result<StateBackgroundPalette> {
        Ok(StateBackgroundPalette { background: parse_hsla(&self.background)? })
    }
}

impl RawFormPalette {
    fn try_into_form(self) -> anyhow::Result<FormPalette> {
        Ok(FormPalette { input: self.input.try_into_form_input()? })
    }
}

impl RawFormInputPalette {
    fn try_into_form_input(self) -> anyhow::Result<FormInputPalette> {
        Ok(FormInputPalette {
            background: parse_hsla(&self.background)?,
            foreground: parse_hsla(&self.foreground)?,
            border: parse_hsla(&self.border)?,
            invalid_border: parse_hsla(&self.invalid_border)?,
            placeholder: parse_hsla(&self.placeholder)?,
        })
    }
}

impl RawFocusPalette {
    fn try_into_focus(self) -> anyhow::Result<FocusPalette> {
        Ok(FocusPalette { ring: parse_hsla(&self.ring)? })
    }
}

impl RawBorderPalette {
    fn try_into_border(self) -> anyhow::Result<BorderPalette> {
        Ok(BorderPalette { default: parse_hsla(&self.default)?, strong: parse_hsla(&self.strong)? })
    }
}

impl RawNavigationPalette {
    fn try_into_navigation(self) -> anyhow::Result<NavigationPalette> {
        Ok(NavigationPalette {
            background: parse_hsla(&self.background)?,
            foreground: parse_hsla(&self.foreground)?,
            muted_foreground: parse_hsla(&self.muted_foreground)?,
            hover_background: parse_hsla(&self.hover_background)?,
            selected_background: parse_hsla(&self.selected_background)?,
            selected_foreground: parse_hsla(&self.selected_foreground)?,
            border: parse_hsla(&self.border)?,
        })
    }
}

impl RawDataPalette {
    fn try_into_data(self) -> anyhow::Result<DataPalette> {
        Ok(DataPalette {
            accent_1: parse_hsla(&self.accent_1)?,
            accent_2: parse_hsla(&self.accent_2)?,
            accent_3: parse_hsla(&self.accent_3)?,
            accent_4: parse_hsla(&self.accent_4)?,
            accent_5: parse_hsla(&self.accent_5)?,
        })
    }
}

impl RawColorTokens {
    fn try_into_colors(self) -> anyhow::Result<ColorTokens> {
        Ok(ColorTokens {
            surface: parse_hsla(&self.surface)?,
            surface_hover: parse_hsla(&self.surface_hover)?,
            surface_pressed: parse_hsla(&self.surface_pressed)?,
            surface_disabled: parse_hsla(&self.surface_disabled)?,
            primary: parse_hsla(&self.primary)?,
            primary_hover: parse_hsla(&self.primary_hover)?,
            primary_pressed: parse_hsla(&self.primary_pressed)?,
            selected: parse_hsla(&self.selected)?,
            selected_hover: parse_hsla(&self.selected_hover)?,
            selected_pressed: parse_hsla(&self.selected_pressed)?,
            text: parse_hsla(&self.text)?,
            text_inverse: parse_hsla(&self.text_inverse)?,
            text_disabled: parse_hsla(&self.text_disabled)?,
            border: parse_hsla(&self.border)?,
        })
    }
}

impl RawMetricTokens {
    fn into_metrics(self) -> MetricTokens {
        MetricTokens {
            spacing: self.spacing,
            radius: self.radius,
            border_width: self.border_width,
            focus: self.focus,
            control: ControlMetricScale { sm: self.control.sm, md: self.control.md, lg: self.control.lg },
            sm: self.sm,
            md: self.md,
            lg: self.lg,
        }
    }
}

impl RawTypography {
    fn into_typography(self) -> LumaTypography {
        LumaTypography {
            font: FontTokens { sans: self.font.sans, mono: self.font.mono, serif: self.font.serif },
            text: TextTokens {
                body: self.text.body.into_text_style(),
                label: self.text.label.into_text_style(),
                caption: self.text.caption.into_text_style(),
                title: self.text.title.into_text_style(),
                code: self.text.code.into_text_style(),
            },
        }
    }
}

impl RawTextStyle {
    fn into_text_style(self) -> LumaTextStyle {
        LumaTextStyle { size: self.size, line_height: self.line_height, weight: self.weight.into() }
    }
}

impl RawElevation {
    fn try_into_elevation(self) -> anyhow::Result<LumaElevation> {
        Ok(LumaElevation {
            none: self.none.try_into_shadow()?,
            control: self.control.try_into_shadow()?,
            thumb: self.thumb.try_into_shadow()?,
            menu: self.menu.try_into_shadow()?,
            popover: self.popover.try_into_shadow()?,
            panel: self.panel.try_into_shadow()?,
            dialog: self.dialog.try_into_shadow()?,
        })
    }
}

impl RawShadow {
    fn try_into_shadow(self) -> anyhow::Result<LumaShadow> {
        Ok(LumaShadow {
            layers: self.layers.into_iter().map(RawShadowLayer::try_into_layer).collect::<anyhow::Result<_>>()?,
        })
    }
}

impl RawShadowLayer {
    fn try_into_layer(self) -> anyhow::Result<LumaShadowLayer> {
        Ok(LumaShadowLayer {
            color: parse_hsla(&self.color)?,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            blur: self.blur,
            spread: self.spread,
        })
    }
}

fn parse_hsla(value: &str) -> anyhow::Result<Hsla> {
    let value = value.trim();
    let (inner, requires_alpha) =
        if let Some(inner) = value.strip_prefix("hsla(").and_then(|value| value.strip_suffix(')')) {
            (inner, true)
        } else if let Some(inner) = value.strip_prefix("hsl(").and_then(|value| value.strip_suffix(')')) {
            (inner, false)
        } else {
            return Err(anyhow!("unsupported color syntax `{value}`"));
        };

    let (channels, alpha) = match inner.split_once('/') {
        Some((channels, alpha)) => (
            channels,
            alpha.trim().parse::<f32>().with_context(|| format!("invalid alpha channel in color `{value}`"))?,
        ),
        None if requires_alpha => return Err(anyhow!("missing alpha channel in color `{value}`")),
        None => (inner, 1.0),
    };

    let mut channels = channels.split_whitespace();
    let hue = parse_number(channels.next(), value, "hue")?;
    let saturation = parse_percent(channels.next(), value, "saturation")?;
    let lightness = parse_percent(channels.next(), value, "lightness")?;

    if channels.next().is_some() {
        return Err(anyhow!("too many color channels in `{value}`"));
    }

    Ok(hsla(hue / 360.0, saturation / 100.0, lightness / 100.0, alpha))
}

fn parse_number(value: Option<&str>, color: &str, channel: &str) -> anyhow::Result<f32> {
    value
        .ok_or_else(|| anyhow!("missing {channel} channel in color `{color}`"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {channel} channel in color `{color}`"))
}

fn parse_percent(value: Option<&str>, color: &str, channel: &str) -> anyhow::Result<f32> {
    value
        .ok_or_else(|| anyhow!("missing {channel} channel in color `{color}`"))?
        .strip_suffix('%')
        .ok_or_else(|| anyhow!("missing percent sign for {channel} channel in color `{color}`"))?
        .parse::<f32>()
        .with_context(|| format!("invalid {channel} channel in color `{color}`"))
}

#[cfg(test)]
mod tests {
    use super::{ColorTokens, DEFAULT_THEME_TOML, LumaTheme, ThemeMode, ThemeModes, ThemeTokens};

    #[test]
    fn default_tokens_are_light_tokens() {
        assert_eq!(ThemeTokens::default().colors.surface, ColorTokens::light().surface);
    }

    #[test]
    fn theme_modes_select_the_requested_token_set() {
        let modes = ThemeModes::default();

        assert_eq!(modes.tokens(ThemeMode::Light).colors.surface, ThemeTokens::light().colors.surface);
        assert_eq!(modes.tokens(ThemeMode::Dark).colors.surface, ThemeTokens::dark().colors.surface);
        assert_ne!(modes.tokens(ThemeMode::Light).colors.surface, modes.tokens(ThemeMode::Dark).colors.surface);
    }

    #[test]
    fn native_theme_exposes_complete_semantic_layers() {
        let theme = LumaTheme::native();
        let light = theme.mode(ThemeMode::Light);

        assert_eq!(theme.name, "Luma Native");
        assert_eq!(theme.version, 1);
        assert_eq!(light.palette.surface.panel.background, light.palette.navigation.background);
        assert_eq!(light.metrics.radius.pill, 999.0);
        assert_eq!(light.typography.text.label.weight, gpui::FontWeight::MEDIUM);
        assert!(!light.elevation.menu.layers.is_empty());
    }

    #[test]
    fn native_theme_rejects_unsupported_color_syntax() {
        let source = DEFAULT_THEME_TOML.replace("background = \"hsl(210 40% 98%)\"", "background = \"#f8fafc\"");

        assert!(LumaTheme::from_toml_str(&source).is_err());
    }
}
