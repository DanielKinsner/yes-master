//! Pin the non-negotiable: the Adaptive Compressor MVP is gated **OFF by
//! default**. In the current architecture the sole startup flip is the explicit
//! `YES_MASTER_ADAPTIVE_COMPRESSION` env var consumed by
//! `init_adaptive_compression_from_env` (the only gate touch in `lib.rs run()`).
//! This test pins two things directly: (a) the fresh-process default is OFF,
//! and (b) `init_adaptive_compression_from_env` only enables on an explicit
//! truthy value. It does not exhaustively enumerate every startup callee — the
//! default-OFF assertion is the tripwire that catches an accidental flip.
//!
//! This lives in its own integration-test binary on purpose. The gate is a
//! process-global `AtomicBool`; lib unit tests flip it under Drop guards but do
//! not all share one mutex, so observing the *default* there would race. A
//! dedicated binary gives a pristine process where nothing else touches the
//! gate or the env var, so the assertions below are deterministic. A single
//! test function keeps the global/env mutations serialized within this binary.

use yes_master_lib::guardrails::{
    init_adaptive_compression_from_env, is_adaptive_compression_enabled,
    set_adaptive_compression_enabled,
};

const ENV_KEY: &str = "YES_MASTER_ADAPTIVE_COMPRESSION";

#[test]
fn adaptive_compression_is_gated_off_by_default_and_only_env_flips_it() {
    // 1. Fresh-process default MUST be OFF — the static initializer is the sole
    //    source of this guarantee; this is what breaks if someone changes
    //    `AtomicBool::new(false)` to `true`.
    assert!(
        !is_adaptive_compression_enabled(),
        "Adaptive Compressor MVP must be gated OFF in a fresh process (owner-gated; \
         do not flip without a listening signoff)"
    );

    // 2. The startup hook with the env var ABSENT must keep it OFF — no other
    //    startup path is allowed to enable it.
    std::env::remove_var(ENV_KEY);
    init_adaptive_compression_from_env();
    assert!(
        !is_adaptive_compression_enabled(),
        "startup init with no env var present must leave the gate OFF"
    );

    // 3. A falsey env value must also keep it OFF.
    std::env::set_var(ENV_KEY, "0");
    init_adaptive_compression_from_env();
    assert!(
        !is_adaptive_compression_enabled(),
        "a falsey env value must not enable the gate"
    );

    // 4. The documented explicit opt-in (a truthy env value) is the ONE path
    //    that flips it on at startup.
    std::env::set_var(ENV_KEY, "1");
    init_adaptive_compression_from_env();
    assert!(
        is_adaptive_compression_enabled(),
        "an explicit truthy YES_MASTER_ADAPTIVE_COMPRESSION must enable the gate"
    );

    // Restore process state so this binary leaves no global residue.
    std::env::remove_var(ENV_KEY);
    set_adaptive_compression_enabled(false);
}
