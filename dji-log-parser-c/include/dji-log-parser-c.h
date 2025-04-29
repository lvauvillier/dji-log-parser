#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

void c_api_free_string(char *s);

char *get_error(void);

char *parse_from_bytes(const uint8_t *bytes, uintptr_t length, const char *api_key);
