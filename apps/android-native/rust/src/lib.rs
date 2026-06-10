//! Android native bridge (A0 skeleton).
//!
//! Proves the cross-compile of the full shared-engine chain
//! (yes_master_lib + the platform-neutral native facade) for Android
//! targets. The JNI surface lands in A1; live audition (oboe) in A3.

use std::ffi::CStr;

/// Bridge identity, read through the same C ABI the iPhone app uses —
/// exercises the cross-crate link end to end.
pub fn bridge_version() -> String {
    let ptr = native_bridge::yes_master_native_bridge_version();
    // SAFETY: the facade returns a pointer to a static NUL-terminated string.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_the_shared_facade() {
        assert!(bridge_version().contains("yes-master"));
    }
}
