#include "vincenthz_wrapper.h"
// #include "lib/vincenthz/json.h"



char* vincenthz::parse(char* data, int json_size, char* key, char* buf, int buf_size) {
	// json_parser parser;
	// // json_parser_callback c;
	//
	// auto callback = [=](void *userdata, int type, const char *data, uint32_t length) {
	// 	return 0;
	// };

	// json_parser_init(&parser, NULL, &callback, NULL);
	// json_parser_string(&parser, data, json_size, NULL);

	// json_parser_free(&parser);

	return (char*)PARSE_ERROR;
	// double ret;
	// int err = json_scanf(data, json_size, "{q: %lf}", &ret);
	//
	// if (err == 0) {
	// 	return (char*)KEY_NOT_FOUND;
	// }
	//
	// if (err < 0) {
	// 	return (char*)PARSE_ERROR;
	// }
	//
	// snprintf((char*)buf, buf_size, "%g", ret);
	//
	// return buf;
}
