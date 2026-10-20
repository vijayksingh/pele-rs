//! Allocation diagnostic through the public engine constructor. No timing claims.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed},
};
struct Counter;
static ENABLED: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static REALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static ONE_BYTE: AtomicU64 = AtomicU64::new(0);
fn record(size: usize) {
    ALLOCATIONS.fetch_add(1, Relaxed);
    BYTES.fetch_add(size as u64, Relaxed);
    if size == 1 {
        ONE_BYTE.fetch_add(1, Relaxed);
    }
}
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc(layout);
        if !pointer.is_null() && ENABLED.load(Relaxed) {
            record(layout.size());
        }
        pointer
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = System.alloc_zeroed(layout);
        if !pointer.is_null() && ENABLED.load(Relaxed) {
            record(layout.size());
        }
        pointer
    }
    unsafe fn realloc(&self, pointer: *mut u8, old: Layout, size: usize) -> *mut u8 {
        let result = System.realloc(pointer, old, size);
        if !result.is_null() && ENABLED.load(Relaxed) {
            REALLOCATIONS.fetch_add(1, Relaxed);
            BYTES.fetch_add(size as u64, Relaxed);
        }
        result
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        System.dealloc(pointer, layout);
    }
}
#[global_allocator]
static ALLOCATOR: Counter = Counter;
fn main() {
    drop(black_box(pele::compat::Engine::with_builtins()));
    let mut rows = Vec::new();
    for _ in 0..5 {
        ALLOCATIONS.store(0, Relaxed);
        REALLOCATIONS.store(0, Relaxed);
        BYTES.store(0, Relaxed);
        ONE_BYTE.store(0, Relaxed);
        ENABLED.store(true, Relaxed);
        let engine = black_box(pele::compat::Engine::with_builtins());
        ENABLED.store(false, Relaxed);
        let values = (
            ALLOCATIONS.load(Relaxed),
            REALLOCATIONS.load(Relaxed),
            BYTES.load(Relaxed),
            ONE_BYTE.load(Relaxed),
        );
        assert_eq!(engine.capabilities().len(), 33);
        drop(engine);
        rows.push(values);
    }
    println!(
        "{}",
        serde_json::json!({"scope":"Public Engine::with_builtins construction only; one warmup; destruction and capability checks outside capture. Requested allocation bytes, not RSS/live bytes or elapsed time.","rows":rows.into_iter().map(|(allocations,reallocations,requested_bytes,one_byte_allocations)|serde_json::json!({"allocations":allocations,"reallocations":reallocations,"requestedBytes":requested_bytes,"oneByteAllocations":one_byte_allocations})).collect::<Vec<_>>()})
    );
}
