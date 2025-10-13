#pragma once

namespace json_c_wrapper {
#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"

char* parse_json_c(char* data, int json_size, char* key, char* buf, int buf_size);
void json_c_free();
}
