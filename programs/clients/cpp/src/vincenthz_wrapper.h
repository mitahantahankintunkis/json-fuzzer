#pragma once

namespace vincenthz {
#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"

char* parse(char* data, int json_size, char* key, char* buf, int buf_size);
}
