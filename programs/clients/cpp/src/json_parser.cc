#include "json_parser.h"

#include "lib/json.h"
#include <stdio.h>
#include <cstring>

#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"

char* parse_json_parser(char* data, int json_size, char* key, char* buf, int buf_size) {
	unsigned int key_len = 1;
    json_value* value = json_parse(data, strlen(data));

    if (value && value->type == json_object) {
        char* ret = (char*)PARSE_ERROR;
        json_object_entry* matched_entry = NULL;

        for (unsigned int i = 0; i < value->u.object.length; ++i) {
            json_object_entry entry = value->u.object.values[i];

            if (entry.name_length == key_len && strncmp(entry.name, key, entry.name_length) == 0) {
                matched_entry = &value->u.object.values[i];
            }
        }

        if (matched_entry && matched_entry->value) {
            json_value* match = matched_entry->value;

            switch (match->type) {
                case json_none:
                    break;

                case json_null:
                    strcpy((char*)buf, "null");
                    ret = (char*)buf;
                    break;

                case json_object:
                    break;

                case json_array:
                    break;

                case json_integer:
                    snprintf((char*)buf, buf_size - 1, "%d", (int)match->u.integer);
                    ret = (char*)buf;
                    break;

                case json_double:
                    snprintf((char*)buf, buf_size - 1, "%f", match->u.dbl);
                    ret = (char*)buf;
                    break;

                case json_string:
                    // ret = strdup(match->u.string.ptr);
					strncpy((char*)buf, match->u.string.ptr, buf_size - 1);
                    ret = (char*)buf;
                    break;

                case json_boolean:
                    if (match->u.boolean) {
                        strcpy((char*)buf, "true");
                    } else {
                        strcpy((char*)buf, "false");
                    }
                    ret = (char*)buf;
                    break;
            }
        }

        json_value_free(value);
        return ret;
    } else {
        json_value_free(value);
        return (char*)PARSE_ERROR;
    }
}
