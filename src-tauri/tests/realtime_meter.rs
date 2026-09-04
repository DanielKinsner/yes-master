//! Live metering must not allocate or grow work with listening duration.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use yes_master_lib::dsp::IntegratedLufs;

struct TrackingAllocator;
thread_local! {
    static TRACKING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                ALLOCATIONS.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let _ = TRACKING.try_with(|tracking| {
            if tracking.get() {
                ALLOCATIONS.with(|count| count.set(count.get() + 1));
            }
        });
        unsafe { System.realloc(ptr, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: TrackingAllocator = TrackingAllocator;

#[test]
fn live_meter_and_reset_allocate_nothing_during_a_long_listen() {
    let mut meter = IntegratedLufs::new(8000);
    ALLOCATIONS.with(|count| count.set(0));
    TRACKING.with(|tracking| tracking.set(true));
    for i in 0..8000 * 180 {
        let sample = if i % 8 < 4 { 0.2 } else { -0.2 };
        meter.process_frame(sample, sample);
    }
    meter.reset();
    TRACKING.with(|tracking| tracking.set(false));
    let allocations = ALLOCATIONS.with(Cell::get);
    assert_eq!(
        allocations, 0,
        "live metering allocated on the audio thread"
    );
    assert_eq!(meter.lufs(), -120.0);
}

#[test]
fn live_meter_agrees_with_exact_integration_for_loud_quiet_and_silent_sections() {
    for sr in [44100, 48000, 96000] {
        let frames = (sr as usize * 12).div_ceil(64) * 64;
        let mut samples = Vec::with_capacity(frames * 2);
        let mut live = IntegratedLufs::new(sr);
        for i in 0..frames {
            let amp = [0.2, 0.0, 0.0005, 0.6, 0.04, 0.3][(i / (sr as usize * 2)).min(5)];
            let s = amp * (std::f32::consts::TAU * 997.0 * i as f32 / sr as f32).sin();
            samples.extend([s, s * 0.8]);
            live.process_frame(s, s * 0.8);
        }
        let mut exact = ebur128::EbuR128::new(2, sr, ebur128::Mode::I).unwrap();
        exact.add_frames_f32(&samples).unwrap();
        let expected = exact.loudness_global().unwrap() as f32;
        assert!(
            (live.lufs() - expected).abs() < 0.15,
            "{sr} Hz live {}, exact {expected}",
            live.lufs()
        );
    }
}
