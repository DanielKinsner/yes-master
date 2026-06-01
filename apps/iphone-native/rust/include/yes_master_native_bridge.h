#pragma once

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

const char *yes_master_native_bridge_version(void);
bool yes_master_native_supports_import_extension(const char *extension);
char *yes_master_native_fixed_export_settings_json(void);
char *yes_master_native_analyze_file_json(const char *path);
void yes_master_native_free_string(char *value);

#ifdef __cplusplus
}
#endif
