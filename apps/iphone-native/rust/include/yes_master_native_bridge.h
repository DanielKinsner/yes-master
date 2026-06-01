#pragma once

#include <stdbool.h>

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
    float intensity
);
void yes_master_native_free_string(char *value);

#ifdef __cplusplus
}
#endif
