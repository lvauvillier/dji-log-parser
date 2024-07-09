#ifndef DJI_LOG_PARSER_H
#define DJI_LOG_PARSER_H

// standard C header for boolean types
#include <stdbool.h>

// Code to allow Go code to call Rust functions as if they were C functions.
// It handles memory management and type conversions between the two languages. 
// A wrapper serves as a bridge, herein defining the interface both Rust (implementation side)
// and Go (calling side) agree to use. 

// main function requiring the path to the file to be parsed and the DJI API-Key.
bool parse_dji_log(const char* input_path, const char* api_key);

// self explanatory, returns a char pointer if an error occurs. 
char* get_last_error();

// this function is used to free memory allocated for strings, use for the prevention of memory leaks.
void free_string(char* s);

// self explanatory, returns a char pointer to the path of the generated GeoJson file. 
char* get_geojson_file_path(const char* input_path);
#endif