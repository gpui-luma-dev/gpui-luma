use gpui::FontWeight;

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
