#ifndef DJI_LOG_PARSER_H
#define DJI_LOG_PARSER_H

// standard C header for boolean types
#include <stdbool.h>
#include <stddef.h>

// Code to allow other languages call Rust functions as if they were C functions. 
// It handles memory management and type conversions between the two languages. 
// A wrapper serves as a bridge, herein defining the interface both Rust (implementation side)
// and Go (calling side) agree to use. 

// main function requiring the path to the file to be parsed and the DJI API-Key.
bool parse_dji_log(const char* input_path, const char* api_key);

// self explanatory, returns a char pointer if an error occurs. 
char* get_last_error();

// this function is used to free memory allocated for strings, use for the prevention of memory leaks.
void c_api_free_string(char* s);

// Generate a GeoJSON string from a byte array containing DJI log data
// bytes: Pointer to the byte array containing DJI log data
// length: Length of the byte array
// api_key: DJI API key for decryption (if needed)
// Returns: A pointer to a null-terminated string containing the GeoJSON data
char* get_geojson_string_from_bytes(const unsigned char* bytes, size_t length, const char* api_key);

#endif