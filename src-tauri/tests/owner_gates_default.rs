//! Pin the remaining owner-gated defaults: the **album character system**
//! (`YES_MASTER_ALBUM_CHARACTER`, owner decision 2026-07-03 D7) and **Phase-B
//! confidence gating** (`YES_MASTER_CONFIDENCE_GATING`) are both OFF by
//! default. Each test pins (a) the fresh-process default is OFF — this is what
//! breaks if someone edits the static `AtomicBool::new(false)` initializer —
//! and (b) the env seed only enables on an explicit truthy value.
//!
//! Same rationale as `adaptive_gate_default.rs`: these gates are process-global
//! `AtomicBool`s that lib unit tests flip under guards, so observing the
//! *default* inside the lib-test process would race. A dedicated integration
//! binary gives a pristine process. The two tests below touch disjoint globals
//! and disjoint env keys, so they stay deterministic even when the harness
//! runs them on parallel threads.

use yes_master_lib::album::{
    init_album_character_from_env, is_album_character_enabled, set_album_character_enabled,
};
use yes_master_lib::confidence::{
    init_confidence_gating_from_env, is_confidence_gating_enabled, set_confidence_gating_enabled,
};

const ALBUM_ENV_KEY: &str = "YES_MASTER_ALBUM_CHARACTER";
const CONFIDENCE_ENV_KEY: &str = "YES_MASTER_CONFIDENCE_GATING";

#[test]
fn album_character_is_gated_off_by_default_and_only_env_flips_it() {
    // 1. Fresh-process default MUST be OFF — the album promise is "nothing
    //    silently altered"; a flipped initializer would apply hidden loudness
    //    pulls and bias EQ to every album render with the whole suite green.
    assert!(
        !is_album_character_enabled(),
        "album character system must be gated OFF in a fresh process (owner-gated; \
         do not flip without an owner listening signoff — OPEN_THREADS 7a)"
    );

    // 2. Startup init with the env var ABSENT must keep it OFF.
    std::env::remove_var(ALBUM_ENV_KEY);
    init_album_character_from_env();
    assert!(
        !is_album_character_enabled(),
        "startup init with no env var present must leave the album-character gate OFF"
    );

    // 3. A falsey env value must also keep it OFF.
    std::env::set_var(ALBUM_ENV_KEY, "0");
    init_album_character_from_env();
    assert!(
        !is_album_character_enabled(),
        "a falsey env value must not enable the album-character gate"
    );

    // 4. The documented explicit opt-in is the ONE startup path that flips it.
    std::env::set_var(ALBUM_ENV_KEY, "1");
    init_album_character_from_env();
    assert!(
        is_album_character_enabled(),
        "an explicit truthy YES_MASTER_ALBUM_CHARACTER must enable the gate"
    );

    // Restore process state so this binary leaves no global residue.
    std::env::remove_var(ALBUM_ENV_KEY);
    set_album_character_enabled(false);
}

#[test]
fn confidence_gating_is_off_by_default_and_only_env_flips_it() {
    // 1. Fresh-process default MUST be OFF. This is the authoritative pin;
    //    the lib-test assertion this replaces raced sibling tests that flip
    //    the gate under ADAPTIVE_COMPRESSION_GATE_TEST_LOCK.
    assert!(
        !is_confidence_gating_enabled(),
        "Phase-B confidence gating must be OFF in a fresh process (owner-gated; \
         calibration bundled into the AC-5 sitting — OPEN_THREADS #3)"
    );

    // 2. Startup init with the env var ABSENT must keep it OFF.
    std::env::remove_var(CONFIDENCE_ENV_KEY);
    init_confidence_gating_from_env();
    assert!(
        !is_confidence_gating_enabled(),
        "startup init with no env var present must leave confidence gating OFF"
    );

    // 3. A falsey env value must also keep it OFF.
    std::env::set_var(CONFIDENCE_ENV_KEY, "0");
    init_confidence_gating_from_env();
    assert!(
        !is_confidence_gating_enabled(),
        "a falsey env value must not enable confidence gating"
    );

    // 4. The documented explicit opt-in is the ONE startup path that flips it.
    std::env::set_var(CONFIDENCE_ENV_KEY, "1");
    init_confidence_gating_from_env();
    assert!(
        is_confidence_gating_enabled(),
        "an explicit truthy YES_MASTER_CONFIDENCE_GATING must enable the gate"
    );

    // Restore process state so this binary leaves no global residue.
    std::env::remove_var(CONFIDENCE_ENV_KEY);
    set_confidence_gating_enabled(false);
}
