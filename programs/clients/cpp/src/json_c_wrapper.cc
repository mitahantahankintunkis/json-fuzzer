#include "json_c_wrapper.h"
#include <cstdio>
#include <json-c/json_object.h>
#include <json-c/json_tokener.h>
#include <json-c/json_types.h>

json_tokener* tok = json_tokener_new();


char* json_c_wrapper::parse_json_c(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	json_tokener_reset(tok);
	json_object *jobj = json_tokener_parse_ex(tok, data, json_size);
	enum json_tokener_error jerr = json_tokener_get_error(tok);

	char* ret = buf;

	if (jerr != json_tokener_success) {
		ret = (char*)PARSE_ERROR;
	} else if (json_tokener_get_parse_end(tok) < (long unsigned int)json_size) {
		ret = (char*)PARSE_ERROR;
	} else {
		json_object* q;
		if (!json_object_object_get_ex(jobj, key, &q)) {
			ret = (char*)KEY_NOT_FOUND;
		}

		double d = json_object_get_double(q);
		snprintf((char*)buf, buf_size, "%g", d);
	}

	json_object_put(jobj);
	return ret;
}


void json_c_wrapper::json_c_free() {
	json_tokener_free(tok);
}
