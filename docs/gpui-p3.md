# Cross-Platform Color Space & Bit-Depth Compatibility in GPUI

This document details how GPUI can maintain backward compatibility with standard 8-bit sRGB screens while enabling high bit-depth and wide-gamut color spaces on macOS (Metal), Windows (Direct3D via `wgpu`), and Linux (Vulkan/Wayland via `wgpu`).

---

## 1. Unified Architecture: Runtime Negotiation & Fallbacks

We cannot assume all systems support >8-bit color or wide gamuts. The core principle of maintaining backward compatibility is **runtime negotiation with a graceful fallback**:

```
[GPUI Startup]
      │
      ▼
[Query Window Surface Capabilities]
      │
      ├─► Display P3 / HDR Supported ──► Set Surface: RGBA16Float + P3/HDR Gamut
      │                                   Set Shader Uniform: ColorSpace = P3/HDR
      │
      └─► Legacy/Unsupported Display ──► Set Surface: BGRA8Unorm + sRGB Gamut (Fallback)
                                          Set Shader Uniform: ColorSpace = sRGB
```

---

## 2. Platform-Specific Integration

GPUI divides its rendering between the native Metal renderer (`gpui_macos`) and the shared wgpu renderer (`gpui_wgpu`), which powers Windows and Linux.

### A. macOS (Metal Layer)
*   **Negotiation**: Query the screen's color space via AppKit:
    ```rust
    let screen = native_window.screen();
    let color_space: id = msg_send![screen, colorSpace];
    ```
*   **Fallback**: If the screen does not support Display P3 (rare on modern Macs), fall back to `MTLPixelFormat::BGRA8Unorm` and sRGB.

### B. Windows (Direct3D 12 & DXGI via `wgpu`)
*   **Negotiation**: In Windows, wide-gamut and HDR are managed through the DXGI swapchain. 
*   **Wgpu Integration**: In [`wgpu_renderer.rs`](file:///Users/scg/Developer/GitHub/gpui-ce/crates/gpui_wgpu/src/wgpu_renderer.rs#L378-L394), the surface capabilities are queried:
    ```rust
    let surface_caps = surface.get_capabilities(&context.adapter);
    ```
    To support high bit-depth, we append `wgpu::TextureFormat::Rgba16Float` or `wgpu::TextureFormat::Rgb10a2Unorm` to the preferred formats list:
    ```rust
    let preferred_formats = [
        wgpu::TextureFormat::Rgba16Float, // Check for 16-bit Float
        wgpu::TextureFormat::Rgb10a2Unorm, // Check for 10-bit Integer
        wgpu::TextureFormat::Bgra8Unorm,
    ];
    ```
*   **DXGI Handling**: Windows automatically configures the DXGI swapchain presentation color space based on the monitor's HDR settings when `Rgba16Float` is selected. If the user has HDR disabled, `wgpu` falls back to `Bgra8Unorm` sRGB.

### C. Linux (Vulkan & Wayland/X11 via `wgpu`)
*   **Negotiation**: 
    *   **Wayland**: Modern Wayland protocols (`wp-color-management-v1`) allow clients to specify color spaces and ICC profiles. Wgpu handles Wayland swapchain formats.
    *   **X11**: X11 lacks native color space negotiation for Vulkan swapchains; the system defaults to sRGB, and color correction is performed by composters.
*   **Fallback**: If the Wayland compositor or Vulkan adapter does not advertise support for high bit-depth swapchains, the renderer defaults to `Bgra8Unorm` sRGB.

---

## 3. Shader Compatibility: Unified Uniform Strategy

Shaders on all platforms already output raw floating-point vectors (`float4` in Metal, `vec4<f32>` in WGSL). The GPU driver handles the translation to the framebuffer's bit depth. 

To maintain color accuracy without maintaining separate shader versions, we use a **Runtime Uniform Flag**:

### A. Metal (`shaders.metal`)
Add a flag to the `GlobalParams` or instance buffer indicating the active color space:
```metal
struct GlobalParams {
    float2 viewport_size;
    uint color_space_mode; // 0 = sRGB, 1 = Display P3
};
```
Inside the fragment shader, conditionally convert colors:
```metal
float4 final_color = fill_color(...);
if (params.color_space_mode == 1) {
    final_color.rgb = srgb_to_display_p3(final_color.rgb);
}
```

### B. WGSL (`shaders.wgsl`)
A similar uniform is passed to the WGSL pipeline:
```wgsl
struct GlobalParams {
    viewport_size: vec2<f32>,
    color_space_mode: u32, // 0 = sRGB, 1 = Display P3 / HDR
}

// In fragment shader:
var color = fill_color(...);
if (global_params.color_space_mode == 1u) {
    color = vec4<f32>(srgb_to_display_p3(color.rgb), color.a);
}
```

---

## 4. Why This Architecture Maintains Backward Compatibility
1.  **Zero Changes to Theme Assets**: All assets, color values, and themes are defined in standard sRGB. The application operates in sRGB by default.
2.  **No Code Duplication**: Shaders and pipelines are shared. The color space remapping is a simple 3x3 matrix multiplication branch in the shader that is enabled dynamically.
3.  **Automatic OS Fallback**: Wgpu and AppKit provide clean capability querying. If a platform or monitor does not support high-fidelity spaces, the system falls back to standard sRGB, ensuring identical visual behavior to the current codebase.
