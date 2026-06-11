# Photoshop Rescue: Programmatic Asset Pipelines for GPUI

This document outlines the rationale, architecture, and workflow for using a **programmatic Photoshop design pipeline** to render high-fidelity, non-traditional (skeuomorphic, neumorphic, and textured) UI elements for GPUI.

---

## 1. Rationale: The Vector Engine Gap

Immediate-mode UI libraries and lightweight GPU-backed vector engines (like GPUI or custom game engine toolkits) are optimized for high-performance layout, text layout, and drawing flat shapes. However, they lack the advanced compositing features required to render soft, organic 3D depth in real time.

### The Limitations of GPUI Canvas Painting
* **No Inset Shadows:** GPUI's `paint_shadows` primitive only supports outset drop shadows. It lacks the layout-level stencil clip or mask operations needed to draw inset shadows (debossed pockets).
* **No Advanced Clipping Masks:** You cannot push/pop arbitrary vector clip paths (like clipping a shadow to the inside of a rounded rectangle) inside a paint closure.
* **No Real-Time Blurs/Filters:** GPUI has no native Gaussian blur or image filters. Attempting to implement CPU-bound box/Gaussian blurs on real-time texture buffers is too slow for 120 FPS interfaces.
* **Missing Shader Effects:** Complex surfaces (like radial brushed metal, carbon fiber, or wood grain) cannot be drawn dynamically without custom, platform-specific fragment shaders.

### The Solution: The Hybrid Pipeline
By rendering the complex 3D lighting, bevels, and shadows as static **high-density raster images (PNGs)** and overlaying live **GPUI vector coordinates** (lines, text, active indicator paths), we achieve professional-grade skeuomorphism with zero CPU/GPU overhead.

---

## 2. When to Image vs. When to Draw

To build a responsive, high-performance interface, you must draw a clear line between what is rendered statically in Photoshop and what is drawn dynamically in GPUI code:

| UI Component | Render in Photoshop (Static PNG) | Draw in GPUI Code (Live Overlay) |
| :--- | :--- | :--- |
| **EQ / Analyzer Screen** | The rounded outer frame, the dark top-left inset shadow, the light bottom-right inner glow. | The vertical grid lines, dashed horizontal indicators, filled area graphs, and active EQ plot curve. |
| **Rotary Dial / Knob** | The outer raised bevel, the face texture (brushed metal/matte), and the rotating pointer line. | The static surrounding tick marks, value readouts, and hover state rings. |
| **Fader Sliders** | The recessed track groove and the raised capsule handle knob. | The track fill level (active track line). |
| **Tactile Switches** | The recessed track groove, the raised thumb capsule, and state indicator lights (colored glows). | Mouse-down event boundaries and toggle state trigger logic. |

---

## 3. How to Program Photoshop (ExtendScript / UXP)

Manually drawing rounded rectangles, calculating offsets, setting up blend modes, and cropping bounds in Photoshop for every single slider, knob, or button scale is incredibly slow. Instead, we can **program Photoshop** using its JavaScript (`.jsx`) scripting engine.

This allows us to write scripts that read the exact dimensions of a selected layer, calculate the proportional lighting offsets, apply the layer styles, and export the file in a single click.

### A. Automatic Neumorphic Style Generator (`.jsx`)
This script automatically applies a mathematically proportional neumorphic layer style based on the selection’s bounding box. Save this code as `NeumorphicStyles.jsx` and run it in Photoshop via **File > Scripts > Browse...**:

```javascript
#target photoshop

if (app.documents.length === 0) {
    alert("Please open a document with a shape layer selected.");
} else {
    var doc = app.activeDocument;
    var layer = doc.activeLayer;
    var bounds = layer.bounds; // [left, top, right, bottom]
    var width = bounds[2].value - bounds[0].value;
    var height = bounds[3].value - bounds[1].value;
    var size = Math.min(width, height);

    var styleChoice = confirm(
        "Choose Style:\n\nYES for [RAISED] (Knobs, Buttons)\nNO for [RECESSED] (Tracks, Screens)"
    );

    // Calculate proportional shadows based on shape size
    var distance = Math.round(size * 0.06).clamp(2, 12);
    var blur = Math.round(size * 0.10).clamp(3, 18);
    var strokeWidth = size < 50 ? 1 : 2;

    applyEffects(layer, {
        type: styleChoice ? "raised" : "recessed",
        distance: distance,
        blur: blur,
        strokeWidth: strokeWidth,
        darkColor: "8c94a0",
        lightColor: "ffffff",
        strokeColor: "bcc0c8"
    });
}

Number.prototype.clamp = function(min, max) {
    return Math.min(Math.max(this, min), max);
};

function applyEffects(layer, config) {
    var idsetd = charIDToTypeID("setd");
    var desc = new ActionDescriptor();
    var ref = new ActionReference();
    ref.putProperty(charIDToTypeID("Prpr"), stringIDToTypeID("layerEffects"));
    ref.putEnumerated(charIDToTypeID("Lyr "), charIDToTypeID("Ordn"), charIDToTypeID("Trgt"));
    desc.putReference(charIDToTypeID("null"), ref);

    var effectsDesc = new ActionDescriptor();

    // 1. Inside Border (Stroke)
    var strokeDesc = new ActionDescriptor();
    strokeDesc.putBoolean(charIDToTypeID("enab"), true);
    strokeDesc.putEnumerated(charIDToTypeID("Styl"), charIDToTypeID("FStl"), stringIDToTypeID("insetFrame"));
    strokeDesc.putUnitDouble(charIDToTypeID("Sz  "), charIDToTypeID("#Pxl"), config.strokeWidth);
    strokeDesc.putUnitDouble(charIDToTypeID("Opct"), charIDToTypeID("#Prc"), 50.0);
    var strokeColor = new ActionDescriptor();
    strokeColor.putDouble(charIDToTypeID("Rd  "), parseInt(config.strokeColor.substring(0,2), 16));
    strokeColor.putDouble(charIDToTypeID("Grn "), parseInt(config.strokeColor.substring(2,4), 16));
    strokeColor.putDouble(charIDToTypeID("Bl  "), parseInt(config.strokeColor.substring(4,6), 16));
    strokeDesc.putObject(charIDToTypeID("Clr "), charIDToTypeID("RGBC"), strokeColor);
    effectsDesc.putObject(charIDToTypeID("FrSt"), charIDToTypeID("FrSt"), strokeDesc);

    // 2. Dark Shadow (Top-Left or Bottom-Right)
    var shadowDesc = new ActionDescriptor();
    shadowDesc.putBoolean(charIDToTypeID("enab"), true);
    shadowDesc.putEnumerated(charIDToTypeID("Md  "), charIDToTypeID("BldM"), charIDToTypeID("Mltp")); // Multiply
    shadowDesc.putUnitDouble(charIDToTypeID("Opct"), charIDToTypeID("#Prc"), config.type === "raised" ? 25.0 : 40.0);
    shadowDesc.putUnitDouble(charIDToTypeID("Lagl"), charIDToTypeID("#Ang"), config.type === "raised" ? -45.0 : 135.0);
    shadowDesc.putBoolean(stringIDToTypeID("useGlobalAngle"), false);
    shadowDesc.putUnitDouble(charIDToTypeID("Dstn"), charIDToTypeID("#Pxl"), config.distance);
    shadowDesc.putUnitDouble(charIDToTypeID("Blur"), charIDToTypeID("#Pxl"), config.blur);
    var shadowColor = new ActionDescriptor();
    shadowColor.putDouble(charIDToTypeID("Rd  "), parseInt(config.darkColor.substring(0,2), 16));
    shadowColor.putDouble(charIDToTypeID("Grn "), parseInt(config.darkColor.substring(2,4), 16));
    shadowColor.putDouble(charIDToTypeID("Bl  "), parseInt(config.darkColor.substring(4,6), 16));
    shadowDesc.putObject(charIDToTypeID("Clr "), charIDToTypeID("RGBC"), shadowColor);
    effectsDesc.putObject(charIDToTypeID("IrSh"), charIDToTypeID("IrSh"), shadowDesc);

    // 3. Light Highlight (Opposite Corner)
    var highlightDesc = new ActionDescriptor();
    highlightDesc.putBoolean(charIDToTypeID("enab"), true);
    highlightDesc.putEnumerated(charIDToTypeID("Md  "), charIDToTypeID("BldM"), charIDToTypeID("Nrml")); // Normal
    highlightDesc.putUnitDouble(charIDToTypeID("Opct"), charIDToTypeID("#Prc"), 85.0);
    highlightDesc.putUnitDouble(charIDToTypeID("Lagl"), charIDToTypeID("#Ang"), config.type === "raised" ? 135.0 : -45.0);
    highlightDesc.putBoolean(stringIDToTypeID("useGlobalAngle"), false);
    highlightDesc.putUnitDouble(charIDToTypeID("Dstn"), charIDToTypeID("#Pxl"), Math.max(1, config.distance - 1));
    highlightDesc.putUnitDouble(charIDToTypeID("Blur"), charIDToTypeID("#Pxl"), Math.max(1, config.blur - 2));
    var highlightColor = new ActionDescriptor();
    highlightColor.putDouble(charIDToTypeID("Rd  "), parseInt(config.lightColor.substring(0,2), 16));
    highlightColor.putDouble(charIDToTypeID("Grn "), parseInt(config.lightColor.substring(2,4), 16));
    highlightColor.putDouble(charIDToTypeID("Bl  "), parseInt(config.lightColor.substring(4,6), 16));
    highlightDesc.putObject(charIDToTypeID("Clr "), charIDToTypeID("RGBC"), highlightColor);
    effectsDesc.putObject(stringIDToTypeID("innerShadowMulti"), stringIDToTypeID("innerShadowMulti"), highlightDesc);

    desc.putObject(charIDToTypeID("T   "), charIDToTypeID("Lefc"), effectsDesc);
    executeAction(idsetd, desc, DialogModes.NO);
}
```

### B. The Spritesheet Compiler (Knob Strip Generator)
For rotary dials, audio engines require a "knob strip" (a single image containing sequential frames of the rotating knob). 

Instead of manually rotating and exporting 61 frames, this script duplicates your active layer, rotates the pointer group by step angles, and stitches them into a single vertical spritesheet ready for your GPUI rendering pipeline:

```javascript
#target photoshop

// Automates KnobMan-style spritesheet compilation directly in Photoshop
function compileKnobSpritesheet(numFrames, startAngle, endAngle) {
    var doc = app.activeDocument;
    var knobSize = doc.width.value; // Assumes square canvas
    var stripHeight = knobSize * numFrames;
    
    // Create new vertical document
    var stripDoc = app.documents.add(
        knobSize, 
        stripHeight, 
        72, 
        "Knob_Spritesheet", 
        NewDocumentMode.RGB, 
        DocumentFill.TRANSPARENT
    );
    
    app.activeDocument = doc;
    var pointerLayer = doc.artLayers.getByName("Rotating Pointer");
    var angleStep = (endAngle - startAngle) / (numFrames - 1);

    for (var i = 0; i < numFrames; i++) {
        var currentAngle = startAngle + (i * angleStep);
        
        // Rotate, copy, paste, translate down, and reset
        pointerLayer.rotate(currentAngle, AnchorPosition.MIDDLECENTER);
        doc.selection.selectAll();
        doc.selection.copy(true); // Copy merged visual state
        
        app.activeDocument = stripDoc;
        var pastedLayer = stripDoc.paste();
        pastedLayer.translate(0, i * knobSize);
        
        app.activeDocument = doc;
        pointerLayer.rotate(-currentAngle, AnchorPosition.MIDDLECENTER); // Reset rotation
    }
}
```

---

## 4. Best Practices for High-Density UI Asset Export

To ensure the exported assets blend seamlessly with the GPUI environment:

1. **Calculate Retina Scaling (@2x and @3x):**
   * GPUI renders in **logical pixels**, but devices display in **physical pixels** based on `window.scale_factor()`.
   * If your GPUI layout expects a `246 x 148` panel, design your shape at `246 x 148` in Photoshop, but export it at **`2.0x`** ($492 \times 296$ pixels) for macOS Retina screens.
2. **Crop to Bounding Box:**
   * Do not include extra blank margins around the asset. Use **Image > Trim** based on transparent pixels to ensure the canvas bounds match the shape edges exactly.
3. **Avoid double-clipping fringes:**
   * Let Photoshop handle the corner rounding entirely. Do not call `.rounded()` in your GPUI layout code on the image element. The transparent corners of your PNG will blend perfectly, avoiding sub-pixel rasterization artifacts (halos or jagged fringes).
