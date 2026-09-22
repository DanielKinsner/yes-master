/* tslint:disable */
/* eslint-disable */

/**
 * One mastered excerpt plus the measurement of what was delivered. The
 * landing stage already measured the chain output once; the post-landing
 * numbers are that measurement shifted by the applied gain, so the caller
 * never needs a second loudness pass.
 */
export class Render {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Take the samples out (consumes the object).
     */
    samples(): Float32Array;
    readonly lufs: number;
    readonly tp: number;
}

/**
 * Stage 2 — dynamics. P95−P10 of 100 ms RMS blocks in dB; `NaN` when the
 * signal is too short or silent (the desktop's "no DR trigger" case).
 */
export function analyze_dynamics(samples: Float32Array, channels: number, sample_rate: number): number;

/**
 * Stage 1 — loudness. Returns `[integrated LUFS, true peak dBTP, LRA LU]`.
 */
export function analyze_loudness(samples: Float32Array, channels: number, sample_rate: number): Float32Array;

/**
 * Stage 3 — stereo field. Returns `[L/R correlation or NaN for mono, side-energy width]`.
 */
export function analyze_stereo(samples: Float32Array, channels: number): Float32Array;

/**
 * Stage 4 — tonal balance. Whole-track 6-band energy shares as JSON, or an
 * empty string when the track is too short for a spectral read.
 */
export function analyze_tonal(samples: Float32Array, channels: number, sample_rate: number): string;

/**
 * Stage 5 — build the mastering context: the desktop's `SourceProfile`, via
 * the same constructor `SourceProfile::from_analysis` uses. Empty string when
 * no profile can be derived (the chain then runs exactly as the app's own
 * "no analysis" path).
 */
export function build_profile(tonal_json: string, dynamics_db: number, lra_lu: number, correlation: number, width: number): string;

/**
 * Master interleaved f32 audio with a Standard style, intensity 0..1 and an
 * absolute LUFS target (−14 / −11 / −9 in Standard), landed under the −1 dBTP
 * ceiling. `source_lufs` is the WHOLE-TRACK integrated loudness from
 * `analyze_loudness` (the desktop injects the same value); `profile_json` is
 * the `build_profile` output, or empty for the app's no-analysis path.
 */
export function master_standard(samples: Float32Array, channels: number, sample_rate: number, style: string, intensity: number, target_lufs: number, source_lufs: number, profile_json: string): Render;

/**
 * Integrated LUFS + true peak of interleaved f32 audio. Returns `[lufs, tp]`.
 */
export function measure_loudness(samples: Float32Array, channels: number, sample_rate: number): Float32Array;

/**
 * Plain-language digest of a profile for the UI ("bright 0.31 / low 0.28 / …").
 */
export function profile_digest(profile_json: string): string;

export function version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_render_free: (a: number, b: number) => void;
    readonly analyze_dynamics: (a: number, b: number, c: number, d: number) => number;
    readonly analyze_loudness: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly analyze_stereo: (a: number, b: number, c: number, d: number) => void;
    readonly analyze_tonal: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly build_profile: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly master_standard: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => void;
    readonly measure_loudness: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly profile_digest: (a: number, b: number, c: number) => void;
    readonly render_lufs: (a: number) => number;
    readonly render_samples: (a: number, b: number) => void;
    readonly render_tp: (a: number) => number;
    readonly version: (a: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
