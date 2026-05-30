// luma-native.rhai

// 1. HELPER: Build a dynamic HSL map or string
fn hsl(h, s, l) {
    return `hsl(${h} ${s}% ${l}%)`;
}
fn hsla(h, s, l, a) {
    return `hsla(${h} ${s}% ${l}% / ${a})`;
}

// 2. SHARED METRICS (No more duplication!)
let shared_metrics = #{
    spacing: #{ s0: 0, s1: 4, s2: 6, s3: 8, s4: 12, s5: 16, s6: 24 },
    radius: #{ none: 0, sm: 3, md: 6, lg: 8, xl: 12, pill: 999 },
    border_width: #{ hairline: 0.5, default: 1, strong: 2 }
};

// 3. BASE BRAND HUES (Tweak here, updates everywhere)
let brand_hue = 221;
let danger_hue = 0;

// =============================================================================
// THE THEME OBJECT
// =============================================================================
let theme = #{
    name: "Luma Native",
    version: 2,

    light: #{
        metrics: shared_metrics, // Direct inheritance!
        
        primitives: #{
            app_canvas: hsl(210, 40, 98),
            app_ink: hsl(222, 47, 11),
            
            // Dynamic generation of action states!
            action_prominent_bg: hsl(brand_hue, 83, 53),
            action_prominent_bg_hover: hsl(brand_hue, 76, 48), // Or use math: brand_hue + 3
            action_prominent_bg_pressed: hsl(brand_hue, 71, 40),
            action_prominent_fg: hsl(0, 0, 100),
            
            transparent: hsla(0, 0, 0, 0)
        }
    },

    dark: #{
        metrics: shared_metrics,
        
        primitives: #{
            app_canvas: hsl(223, 49, 8),
            app_ink: hsl(210, 40, 98),
            
            // Dark mode overrides brand saturation/lightness automatically
            action_prominent_bg: hsl(213, 94, 68),
            action_prominent_bg_hover: hsl(212, 96, 78),
            action_prominent_bg_pressed: hsl(213, 97, 87),
            action_prominent_fg: hsl(204, 80, 16),
            
            transparent: hsla(0, 0, 0, 0)
        }
    }
};

// 4. RESOLVE PALETTES PROGRAMMATICALLY
// Instead of manually mapping `{ ref = "..." }`, you can script the resolution
theme.light.palette = #{
    app: #{
        background: theme.light.primitives.app_canvas,
        foreground: theme.light.primitives.app_ink
    },
    action: #{
        prominent: #{
            background: theme.light.primitives.action_prominent_bg,
            foreground: theme.light.primitives.action_prominent_fg,
            hover_background: theme.light.primitives.action_prominent_bg_hover
        }
    }
};

// Mirror the palette resolution logic for dark...
theme.dark.palette = #{ /* ... */ };

theme; // Return the fully resolved object to Rust