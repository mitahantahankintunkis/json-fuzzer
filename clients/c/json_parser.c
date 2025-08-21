#include "json_parser.h"

#include "json.h"
#include <stdio.h>
#include <string.h>

#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"

char* return_buffer2[256];

char* parse_json_parser(char* data, char* key) {
    json_value* value = json_parse(data, strlen(data));

    if (value && value->type == json_object) {
        char* ret = PARSE_ERROR;
        json_object_entry* matched_entry = NULL;

        for (int i = 0; i < value->u.object.length; ++i) {
            json_object_entry entry = value->u.object.values[i];

            if (strncmp(entry.name, key, entry.name_length) == 0) {
                matched_entry = &entry;
            }
        }

        if (matched_entry && matched_entry->value) {
            json_value* match = matched_entry->value;

            switch (match->type) {
                case json_none:
                    break;

                case json_null:
                    strcpy((char*)return_buffer2, "null");
                    ret = (char*)return_buffer2;
                    break;

                case json_object:
                    break;

                case json_array:
                    break;

                case json_integer:
                    snprintf((char*)return_buffer2, sizeof(return_buffer2), "%d", (int)match->u.integer);
                    ret = (char*)return_buffer2;
                    break;

                case json_double:
                    snprintf((char*)return_buffer2, sizeof(return_buffer2), "%f", match->u.dbl);
                    ret = (char*)return_buffer2;
                    break;

                case json_string:
                    ret = strdup(match->u.string.ptr);
                    break;

                case json_boolean:
                    if (match->u.boolean) {
                        strcpy((char*)return_buffer2, "true");
                    } else {
                        strcpy((char*)return_buffer2, "false");
                    }
                    ret = (char*)return_buffer2;
                    break;
            }
        }

        json_value_free(value);
        return ret;
    } else {
        return PARSE_ERROR;
    }
}
