#pragma once

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

const char *yes_master_native_bridge_version(void);
bool yes_master_native_supports_import_extension(const char *extension);
char *yes_master_native_fixed_export_settings_json(void);
char *yes_master_native_analyze_file_json(const char *path);
char *yes_master_native_render_master_json(const char *source_path, const char *output_dir);
char *yes_master_native_render_master_with_options_json(
    const char *source_path,
    const char *output_dir,
    const char *preset,
    float intensity,
    float lufs_target
);
void yes_master_native_free_string(char *value);

/* --- Live mastered audition --------------------------------------------- *
 * Swift owns the iOS output graph (AVAudioEngine + one AVAudioSourceNode) and
 * calls `_process` from the render thread to pull mastered PCM. Rust owns the
 * decoded PCM, the frame cursor, and the MasteringChain. The handle is opaque.
 * `_process` is real-time safe; every other call is for non-audio threads.    */
typedef struct YMLiveHandle YMLiveHandle;

YMLiveHandle *yes_master_native_live_create(
    const char *source_path,
    const char *preset,
    float intensity,
    float lufs_target
);
uint32_t yes_master_native_live_process(
    YMLiveHandle *handle,
    float *out_interleaved,
    uint32_t frames
);
void yes_master_native_live_set_bypass(YMLiveHandle *handle, bool original);
void yes_master_native_live_set_params(
    YMLiveHandle *handle,
    const char *preset,
    float intensity,
    float lufs_target
);
void yes_master_native_live_set_volume_match(YMLiveHandle *handle, float linear_gain);
void yes_master_native_live_set_landing_gain(YMLiveHandle *handle, float linear_gain);
float yes_master_native_live_measure_landing(
    const YMLiveHandle *handle,
    const char *preset,
    float intensity,
    float lufs_target,
    float *out_mastered_lufs
);
void yes_master_native_live_seek(YMLiveHandle *handle, double position_seconds);
double yes_master_native_live_position_seconds(const YMLiveHandle *handle);
double yes_master_native_live_duration_seconds(const YMLiveHandle *handle);
uint32_t yes_master_native_live_channels(const YMLiveHandle *handle);
double yes_master_native_live_sample_rate(const YMLiveHandle *handle);
void yes_master_native_live_destroy(YMLiveHandle *handle);

#ifdef __cplusplus
}
#endif
