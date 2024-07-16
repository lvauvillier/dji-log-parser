#ifndef DJI_LOG_PARSER_H
#define DJI_LOG_PARSER_H

#include <stdbool.h>
#include <stddef.h>

bool parse_dji_log(const char* input_path, const char* api_key);
char* get_last_error();
void c_api_free_string(char* s);
char* get_geojson_string_from_bytes(const unsigned char* bytes, size_t length, const char* api_key);

#endif
