#ifndef DJI_LOG_PARSER_H
#define DJI_LOG_PARSER_H

#include <stdbool.h>

bool parse_dji_log(const char* input_path, const char* api_key);
char* get_last_error();
void free_string(char* s);
char* get_geojson_file_path(const char* input_path);
#endif