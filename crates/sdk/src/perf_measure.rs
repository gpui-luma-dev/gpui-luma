//! Shared diagnostic measurement support. Compiled only by tests and the benchmark.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    time::Instant,
};

#[derive(Clone, Copy, Default)]
struct Counts {
    enabled: bool,
    allocations: usize,
    bytes: usize,
}

thread_local! {
    static COUNTS: Cell<Counts> = const { Cell::new(Counts { enabled: false, allocations: 0, bytes: 0 }) };
}

struct CountingAllocator;

fn count(bytes: usize) {
    // TLS can be unavailable during thread teardown. No allocation is needed to count.
    let _ = COUNTS.try_with(|cell| {
        let mut counts = cell.get();
        if counts.enabled {
            counts.allocations += 1;
            counts.bytes += bytes;
            cell.set(counts);
        }
    });
}

// SAFETY: Allocation and deallocation are delegated unchanged to System. The
// thread-local counters contain only integers and do not allocate or synchronize.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count(size);
        unsafe { System.realloc(ptr, layout, size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

struct CountingScope;
impl Drop for CountingScope {
    fn drop(&mut self) {
        COUNTS.with(|cell| cell.set(Counts { enabled: false, ..cell.get() }));
    }
}

pub(crate) fn allocations_for<T>(operation: impl FnOnce() -> T) -> (T, usize, usize) {
    COUNTS.with(|cell| cell.set(Counts { enabled: true, ..Counts::default() }));
    let scope = CountingScope;
    let result = operation();
    drop(scope);
    let counts = COUNTS.with(Cell::get);
    (result, counts.allocations, counts.bytes)
}

pub(crate) fn measure<T>(name: &str, iterations: usize, mut operation: impl FnMut() -> T) {
    // Warm the process, not necessarily the control: cold cases explicitly reset
    // their control caches on every operation. Output destruction is included.
    for _ in 0..3 {
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
    let (_, allocations, bytes) = allocations_for(|| {
        for _ in 0..iterations {
            black_box(operation());
        }
    });
    println!(
        "{name}: {:.1} ns/op [{:.1}, {:.1}], {:.3} allocations/op, {:.1} requested bytes/op",
        samples[2],
        samples[0],
        samples[4],
        allocations as f64 / iterations as f64,
        bytes as f64 / iterations as f64,
    );
}
