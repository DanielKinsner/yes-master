//! Per-thread allocation tracking for library tests; never part of production.
use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
};

struct TrackingAllocator;
thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static COUNT: Cell<usize> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                COUNT.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

pub(crate) fn count<T>(work: impl FnOnce() -> T) -> (usize, T) {
    struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            TRACKING.with(|tracking| tracking.set(false));
        }
    }
    assert!(
        !TRACKING.with(Cell::get),
        "allocation tracking must not nest"
    );
    COUNT.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    let guard = Guard;
    let result = work();
    drop(guard);
    (COUNT.with(Cell::get), result)
}
