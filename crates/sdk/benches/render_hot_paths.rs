//! Isolated headless costs, not application frame times. Run with:
//! `cargo bench -p gpui-luma --bench render_hot_paths -- --measure`
use std::{
    alloc::{GlobalAlloc, Layout, System},
    collections::HashMap,
    hint::black_box,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Instant,
};

use gpui::{BoxShadow, ElementId, SharedString};
use gpui_luma::{
    controls::{
        button_family::{ButtonFamilyRole, compose_button_family_look, default_button_family_theme},
        sidebar::SidebarPanelTemplateHandlers,
        tree_view::TreeViewTemplateHandlers,
    },
    theme::{ControlSize, InteractionState, MetricTokens, StandardBoxScale, ThemeTokens},
};
use smallvec::SmallVec;

struct CountingAllocator;
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: Every allocation operation delegates unchanged to System. Counters do
// not allocate and are only enabled during a separate, single-threaded pass.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn measure<T>(name: &str, iterations: usize, mut operation: impl FnMut() -> T) {
    for _ in 0..100 {
        black_box(operation());
    }
    let mut samples = [0.0; 5];
    for sample in &mut samples {
        let start = Instant::now();
        for _ in 0..iterations {
            black_box(operation());
        }
        *sample = start.elapsed().as_nanos() as f64 / iterations as f64;
    }
    samples.sort_by(f64::total_cmp);
    ALLOCATIONS.store(0, Ordering::Relaxed);
    COUNTING.store(true, Ordering::Relaxed);
    for _ in 0..iterations {
        black_box(operation());
    }
    COUNTING.store(false, Ordering::Relaxed);
    let allocations = ALLOCATIONS.load(Ordering::Relaxed);
    println!("{name}: {:.1} ns/op, {:.3} allocations/op", samples[2], allocations as f64 / iterations as f64);
}

fn main() {
    // `cargo test --all-targets` should compile this target without benchmarking.
    if !std::env::args().any(|argument| argument == "--measure") {
        return;
    }
    println!(
        "Isolated costs (median of five batches); fixtures and index/cache construction excluded. Allocation counts include reallocations."
    );
    let iterations = 100_000;
    let metrics = MetricTokens::default();
    measure("StandardBoxScale/direct", iterations, || {
        StandardBoxScale::compute(black_box(ControlSize::Md), black_box(&metrics), black_box(2.0))
    });

    let id = SharedString::from("benchmark-table");
    measure("ElementId/formatted", iterations, || {
        ElementId::from(format!("{}-row-{}", black_box(&id), black_box(42usize)))
    });
    measure("ElementId/composite", iterations, || ElementId::from((black_box(&id).clone(), black_box(42usize))));

    for count in [100usize, 10_000] {
        let ids: Vec<SharedString> = (0..count).map(|index| format!("node-{index}").into()).collect();
        let indices: HashMap<SharedString, usize> =
            ids.iter().cloned().enumerate().map(|(index, id)| (id, index)).collect();
        let target = &ids[count - 1];
        measure(&format!("Tree lookup/linear/{count}"), 10_000, || {
            black_box(&ids).iter().position(|id| id == black_box(target))
        });
        measure(&format!("Tree lookup/indexed/{count}"), iterations, || {
            black_box(&indices).get(black_box(target)).copied()
        });
    }

    let handlers = TreeViewTemplateHandlers {
        hover: Rc::new(|_, _, _| {}),
        mouse_down: Rc::new(|_, _, _| {}),
        mouse_up: Rc::new(|_, _, _| {}),
        click: Rc::new(|_, _, _| {}),
        disclosure: Rc::new(|_, _, _| {}),
    };
    measure("Tree handlers/shared clone", iterations, || black_box(&handlers).clone());

    for count in [100usize, 10_000] {
        let mut sidebar = SidebarPanelTemplateHandlers::default();
        sidebar.row_hovers = Rc::new((0..count).map(|_| handlers.hover.clone()).collect());
        sidebar.row_mouse_downs = Rc::new((0..count).map(|_| handlers.mouse_down.clone()).collect());
        sidebar.row_mouse_ups = Rc::new((0..count).map(|_| handlers.mouse_up.clone()).collect());
        sidebar.row_mouse_up_outs = Rc::clone(&sidebar.row_mouse_ups);
        sidebar.row_clicks = Rc::new((0..count).map(|_| handlers.click.clone()).collect());
        // Mirrors the cached row-container clone; excludes the O(N) signature check.
        measure(&format!("Sidebar/shared row containers/{count}"), iterations, || {
            let sidebar = black_box(&sidebar);
            (
                Rc::clone(&sidebar.row_bounds),
                Rc::clone(&sidebar.row_hovers),
                Rc::clone(&sidebar.row_mouse_downs),
                Rc::clone(&sidebar.row_mouse_ups),
                Rc::clone(&sidebar.row_mouse_up_outs),
                Rc::clone(&sidebar.row_clicks),
                Rc::clone(&sidebar.children_height_reports),
            )
        });
    }

    let theme = default_button_family_theme();
    let palette = theme.resolve(ButtonFamilyRole::Text, ControlSize::Md, InteractionState::default());
    let scale = StandardBoxScale::compute(ControlSize::Md, &metrics, 2.0);
    let mut look = compose_button_family_look(&palette, ButtonFamilyRole::Text, &scale, metrics.radius.pill);
    measure("Button look/no shadows clone", iterations, || black_box(&look).clone());
    let shadows = ThemeTokens::default().elevation.control.to_box_shadows();
    println!("Shadow fixture: {} layers", shadows.len());
    look.shadow = Some(shadows.clone());
    measure("Button look/Vec shadows clone", iterations, || black_box(&look).clone());
    let inline: SmallVec<[BoxShadow; 2]> = shadows.iter().cloned().collect();
    let shared: Arc<[BoxShadow]> = shadows.into();
    measure("Shadow storage/SmallVec clone", iterations, || black_box(&inline).clone());
    measure("Shadow storage/Arc clone", iterations, || Arc::clone(black_box(&shared)));
}
