#![cfg(test)]

use std::collections::BTreeMap;

use gpui_luma_look_shadcn::CssTokenMap;

pub fn sample_catalog() -> CssTokenMap {
    CssTokenMap::from_map(BTreeMap::from([
        ("primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("primary-foreground".into(), "oklch(1 0 0)".into()),
        ("secondary".into(), "oklch(0.6437 0.1019 187.3840)".into()),
        ("secondary-foreground".into(), "oklch(1 0 0)".into()),
        ("background".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("card".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("muted".into(), "oklch(0.6979 0.0159 196.7940)".into()),
        ("muted-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ("accent-foreground".into(), "oklch(1 0 0)".into()),
        ("border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("input".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("sidebar".into(), "oklch(0.9735 0.0261 90.0953)".into()),
        ("sidebar-foreground".into(), "oklch(0.3092 0.0518 219.6516)".into()),
        ("sidebar-primary".into(), "oklch(0.5924 0.2025 355.8943)".into()),
        ("sidebar-primary-foreground".into(), "oklch(1 0 0)".into()),
        ("sidebar-accent".into(), "oklch(0.5808 0.1732 39.5003)".into()),
        ("sidebar-accent-foreground".into(), "oklch(1 0 0)".into()),
        ("sidebar-border".into(), "oklch(0.6537 0.0197 205.2618)".into()),
        ("sidebar-ring".into(), "oklch(0.5924 0.2025 355.8943)".into()),
    ]))
}

pub fn retro_arcade_catalog() -> CssTokenMap {
    CssTokenMap::from_map(BTreeMap::from([
        ("primary".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
        ("primary-foreground".into(), "hsl(0 0% 100%)".into()),
        ("secondary".into(), "hsl(175.4622 58.6207% 39.8039%)".into()),
        ("secondary-foreground".into(), "hsl(0 0% 100%)".into()),
        ("background".into(), "hsl(43.8462 86.6667% 94.1176%)".into()),
        ("foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
        ("muted".into(), "hsl(180 6.9307% 60.3922%)".into()),
        ("muted-foreground".into(), "hsl(192.2034 80.8219% 14.3137%)".into()),
        ("accent".into(), "hsl(17.5691 80.4444% 44.1176%)".into()),
        ("accent-foreground".into(), "hsl(0 0% 100%)".into()),
        ("destructive".into(), "hsl(1.0405 71.1934% 52.3529%)".into()),
        ("destructive-foreground".into(), "hsl(0 0% 100%)".into()),
        ("border".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
        ("input".into(), "hsl(186.3158 8.2969% 55.0980%)".into()),
        ("ring".into(), "hsl(330.9554 64.0816% 51.9608%)".into()),
        ("card".into(), "hsl(45.6000 42.3729% 88.4314%)".into()),
        ("radius".into(), "0.25rem".into()),
        ("spacing".into(), "0.25rem".into()),
        ("font-sans".into(), "ui-sans-serif, system-ui, 'Outfit', sans-serif".into()),
        ("shadow-xs".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.05)".into()),
        ("shadow-sm".into(), "0 1px 3px 0px hsl(0 0% 0% / 0.10), 0 1px 2px -1px hsl(0 0% 0% / 0.10)".into()),
    ]))
}
