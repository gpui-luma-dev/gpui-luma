use gpui::{Hsla, rgb};

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
