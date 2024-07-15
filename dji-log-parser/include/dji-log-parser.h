#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

extern "C" {

char *get_geojson_string(const char *input_path, const char *api_key);

char *get_geojson_string_from_bytes(const uint8_t *bytes, uintptr_t length, const char *api_key);

char *get_last_error();

void free_string(char *s);

bool parse_dji_log(const char *input_path, const char *api_key);

char *get_geojson_file_path(const char *input_path);

} // extern "C"
