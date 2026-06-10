package com.yesmaster.app

/**
 * The Standard vocabulary, mirroring src/standard-mapping-parity.json. The
 * style ids are what the bridge's native_preset() resolves — the Rust side
 * of this contract is pinned by the android crate's parity test, the
 * desktop side by standard-mapping.test.ts, the iPhone side by the facade
 * test. Loudness mirrors Swift's NativeLoudness (low/medium/high →
 * -14/-11/-9 LUFS).
 */

enum class StandardStyle(val id: String, val label: String, val subtitle: String) {
    BALANCED("balanced", "Balanced", "Clean balance"),
    BRIGHT("bright", "Bright", "Air & detail"),
    WARM("warm", "Warm", "Glue & body"),
    HEAVY("heavy", "Heavy", "Sub & weight"),
}

enum class StandardLoudness(val id: String, val label: String, val lufs: Float) {
    LOW("low", "Low", -14f),
    MEDIUM("medium", "Medium", -11f),
    HIGH("high", "High", -9f),
}
