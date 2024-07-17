#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

char *get_geojson_string(const char *input_path, const char *api_key);

char *get_geojson_string_from_bytes(const uint8_t *bytes, uintptr_t length, const char *api_key);

bool parse_dji_log(const char *input_path, const char *api_key);

char *get_last_error(void);

void c_api_free_string(char *s);
