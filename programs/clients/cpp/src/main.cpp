// Written mostly in C
#include <Poco/Dynamic/Var.h>
#include <arpa/inet.h>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <netinet/tcp.h>
#include <stdexcept>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdlib.h>
#include <string.h>
#include <string>
#include <sys/socket.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>
#include <stdint.h>
#include <stdio.h>

#include <Poco/JSON/Object.h>
#include <Poco/JSON/Parser.h>
#include <jansson.h>

//#define BOOST_CONTAINER_NO_LIB
#include <boost/json/src.hpp>

#include "lib/modsecurity_json.h"
#include "lib/mjson.h"
#include "lib/cJSON.h"
#include "lib/frozen.h"
#include "lib/jsmn.h"
#include "lib/nlohmann/json.hpp"
#include "lib/mongoose/mongoose.h"

#include "json_parser.h"
#include "json_c_wrapper.h"

#include <sanitizer/coverage_interface.h>
// #include "vincenthz_wrapper.h"

#define SERVER_ADDR "127.0.0.1"
#define SERVER_PORT 5000
#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"


// std::vector<int> coverage_buf_0 = std::vector<int>();
// std::vector<int> coverage_buf_1 = std::vector<int>();
// Coverage information
uint16_t coverage_n = 0;
char* coverage_buf = NULL;
// int* coverage_buf1 = (int*)malloc(coverage_n);
// int coverage_i = 0;
// int coverage_i1 = 0;
bool save_coverage = false;
bool new_coverage_info = false;

// Clears recorded coverage information
// void start_coverage() {
// 	coverage_i = 0;
// 	save_coverage = true;
// }

// void switch_coverage_buffers() {
// 	auto temp = coverage_buf;
// 	coverage_buf = coverage_buf;
// 	coverage_buf = temp;
//
// 	coverage_i = coverage_i;
// }

// void stop_coverage() {
// 	save_coverage = false;
// }

// static const uint32_t Prime1 = 2654435761U;
// static const uint32_t Prime2 = 2246822519U;
//
// /// rotate bits, should compile to a single CPU instruction (ROL)l
// static inline uint32_t rotateLeft(uint32_t x, unsigned char bits) {   return (x << bits) | (x >> (32 - bits)); }
// /// process a block of 4x4 bytes, this is the main part of the XXHash32 algorithml
// static inline void process(const void* data, uint32_t& state0, uint32_t& state1, uint32_t& state2, uint32_t& state3) {   const uint32_t* block = (const uint32_t*) data;   state0 = rotateLeft(state0 + block[0] * Prime2, 13) * Prime1;   state1 = rotateLeft(state1 + block[1] * Prime2, 13) * Prime1;   state2 = rotateLeft(state2 + block[2] * Prime2, 13) * Prime1;   state3 = rotateLeft(state3 + block[3] * Prime2, 13) * Prime1; }
//

// Boost hash_combine hashing algorithm
// uint32_t coverage_hash() {
// 	uint32_t seed = 0x9e3779b9;
//
// 	for (int i = 0; i < coverage_i; ++i) {
// 		seed ^= coverage_buf[i] + 0x9e3779b9 + (seed << 6) + (seed >> 2);
// 	}
//
// 	return seed;
// }

// __attribute__((no_sanitize("coverage")))
// __attribute__((noinline))
// void push_coverage(int i) {
// 	coverage->push_back(i);
// }

// Clang's SanitizerCoverage implementation
// Copied from https://clang.llvm.org/docs/SanitizerCoverage.html
// This callback is inserted by the compiler as a module constructor
// into every DSO. 'start' and 'stop' correspond to the
// beginning and end of the section with the guards for the entire
// binary (executable or DSO). The callback will be called at least
// once per DSO and may be called multiple times with the same parameters.
extern "C" void __sanitizer_cov_trace_pc_guard_init(uint32_t *start,
													uint32_t *stop) {
	static uint64_t N;  // Counter for the guards.
	if (start == stop || *start) return;  // Initialize only once.

	int n = stop - start;

	if (n >= (1 << 16)) {
		printf("C/C++ Client: Coverage overflow\n");
		exit(1);
	}

	if (coverage_n < n) {
		coverage_n = n;
		coverage_buf = (char*)realloc(coverage_buf, coverage_n);
	}

	// printf("INIT: %p %p\n", start, stop);
	// printf("init: %ld\n", stop - start - 1);
	for (uint32_t *x = start; x < stop; x++)
		*x = ++N;  // Guards should start from 1.
}

// This callback is inserted by the compiler on every edge in the
// control flow (some optimizations apply).
// Typically, the compiler will emit the code like this:
//    if(*guard)
//      __sanitizer_cov_trace_pc_guard(guard);
// But for large functions it will emit a simple call:
//    __sanitizer_cov_trace_pc_guard(guard);
extern "C" void __sanitizer_cov_trace_pc_guard(uint32_t *guard) {
	if (!*guard) return;  // Duplicate the guard check.
	// If you set *guard to 0 this code will not be called again for this edge.
	// Now you can get the PC and do whatever you want:
	//   store it somewhere or symbolize it and print right away.
	// The values of `*guard` are as you set them in
	// __sanitizer_cov_trace_pc_guard_init and so you can make them consecutive
	// and use them to dereference an array or a bit vector.

	if (!coverage_buf[*guard - 1]) {
		coverage_buf[*guard - 1] = 1;
		new_coverage_info = true;
	}

	// if (save_coverage && coverage_i < (coverage_n)) {
	// 	// coverage_buf_0.push_back(*guard);
	// 	// push_coverage(*guard);
	// }

	// *guard = 0;

	// void *PC = __builtin_return_address(0);
	// char PcDescr[1024];
	// // This function is a part of the sanitizer run-time.
	// // To use it, link with AddressSanitizer or other sanitizer.
	// __sanitizer_symbolize_pc(PC, "%p %F %L", PcDescr, sizeof(PcDescr));
	// printf("guard: %p %x PC %s\n", guard, *guard, PcDescr);
}

//
// This callback is inserted by the compiler as a module constructor
// into every DSO. 'start' and 'stop' correspond to the
// beginning and end of the section with the guards for the entire
// binary (executable or DSO). The callback will be called at least
// once per DSO and may be called multiple times with the same parameters.
// int coverage_start = -1;
// int coverage_end = -1;

// extern "C" void __sanitizer_cov_trace_pc_guard_init(uint32_t *start,
// 													uint32_t *stop) {
// 	static uint64_t N;  // Counter for the guards.
// 	if (start == stop || *start) return;  // Initialize only once.
// 	// printf("INIT: %p %p %ld\n", start, stop, stop - start);
//
// 	for (uint32_t *x = start; x < stop; x++)
// 		*x = ++N;  // Guards should start from 1.
// }
//
// std::vector<int>* coverage = new std::vector<int>();
// bool save_coverage = false;
//
// void start_coverage() {
// 	coverage->clear();
// 	save_coverage = true;
// }
//
// void stop_coverage() {
// 	save_coverage = false;
// }
//
// __attribute__((no_sanitize("coverage")))
// void push_coverage(int i) {
// 	coverage->push_back(i);
// }
//
// // This callback is inserted by the compiler on every edge in the
// // control flow (some optimizations apply).
// // Typically, the compiler will emit the code like this:
// //    if(*guard)
// //      __sanitizer_cov_trace_pc_guard(guard);
// // But for large functions it will emit a simple call:
// //    __sanitizer_cov_trace_pc_guard(guard);
// extern "C" void __sanitizer_cov_trace_pc_guard(uint32_t *guard) {
// 	if (!*guard) return;  // Duplicate the guard check.
// 	// If you set *guard to 0 this code will not be called again for this edge.
// 	// Now you can get the PC and do whatever you want:
// 	//   store it somewhere or symbolize it and print right away.
// 	// The values of `*guard` are as you set them in
// 	// __sanitizer_cov_trace_pc_guard_init and so you can make them consecutive
// 	// and use them to dereference an array or a bit vector.
// 	if (save_coverage) {
// 		// *guard = 0;
// 		// push_coverage(*guard);
// 	}
//
// 	void *PC = __builtin_return_address(0);
// 	char PcDescr[1024];
// 	// This function is a part of the sanitizer run-time.
// 	// To use it, link with AddressSanitizer or other sanitizer.
// 	__sanitizer_symbolize_pc(PC, "%p %F %L", PcDescr, sizeof(PcDescr));
// 	printf("guard: %p %x PC %s\n", guard, *guard, PcDescr);
// }




int recv_all(int sock, void *buf, size_t len) {
    size_t received = 0;
    while (received < len) {
        ssize_t n = recv(sock, (char*)buf + received, len - received, 0);
        if (n <= 0) return -1;
        received += n;
    }
    return 0;
}

int send_all(int sock, const void *buf, size_t len) {
    size_t sent = 0;
    while (sent < len) {
        ssize_t n = send(sock, (char*)buf + sent, len - sent, 0);
        if (n <= 0) return -1;
        sent += n;
    }
    return 0;
}


char* parse_cjson(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
    cJSON *parsed = cJSON_Parse(data);
    if (parsed) {
		if (!cJSON_HasObjectItem(parsed, key)) {
        cJSON_Delete(parsed);
			return (char*)KEY_NOT_FOUND;
		}

        cJSON *value = cJSON_GetObjectItemCaseSensitive(parsed, key);
		cJSON_PrintPreallocated(value, buf, buf_size, false);
        cJSON_Delete(parsed);
		return buf;

			//      if (cJSON_IsNumber(value) && value->valuedouble) {
			//          // if (value->valuedouble == floorf(value->valuedouble)) {
			//          //     snprintf((char*)buf, buf_size, "%d", value->valueint);
			//          // } else {
			//          snprintf((char*)buf, buf_size, "%g", value->valuedouble);
			//          // }
			//          ret = (char*)buf;
			//      } else if (cJSON_IsString(value) && value->valuestring) {
			// strncpy(buf, value->valuestring, buf_size - 1);
			//          ret = (char*)buf;
			//      } else {
			//          ret = (char*)KEY_NOT_FOUND;
			//      }

    } else {
        return (char*)PARSE_ERROR;
    }
}


char* parse_jansson(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
    json_t *root;
    json_error_t error;

    root = json_loads(data, 0, &error);

    if (root) {
        char* ret = (char*)PARSE_ERROR;

        if (json_is_object(root)) {
            json_t* query = json_object_get(root, key);

            if (query) {
                if (json_is_string(query)) {
                    // ret = strdup(json_string_value(query));
					strncpy(buf, json_string_value(query), buf_size - 1);
                    ret = (char*)buf;
                } else if (json_is_integer(query)) {
                    snprintf((char*)buf, buf_size, "%lld", json_integer_value(query));
                    ret = (char*)buf;
                } else  if (json_is_real(query)) {
                    snprintf((char*)buf, buf_size, "%g", json_real_value(query));
                    ret = (char*)buf;
                }
            } else {
                ret = (char*)KEY_NOT_FOUND;
            }
        }

        json_decref(root);
        return ret;
    } else {
        return (char*)PARSE_ERROR;
    }
}


char* parse_yajl(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	// TODO - optimize
	char* path = (char*)malloc(key_size + 6);
	sprintf(path, "json.%s", key);

	char* ret = buf;

	modsecurity::RequestBodyProcessor::JSON parser = modsecurity::RequestBodyProcessor::JSON();
	std::string error;
	parser.processChunk(data, json_size, &error);
	parser.complete(&error);

	if (!error.empty()) {
		ret = (char*)PARSE_ERROR;
	} else if (!parser.parsed.contains(path)) {
		ret = (char*)KEY_NOT_FOUND;
	} else {
		strncpy(buf, parser.parsed[path].c_str(), buf_size - 1);
	}

	free(path);
	return ret;
}


char* parse_mjson(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	double ret;

	// TODO - optimize
	char* path = (char*)malloc(key_size + 3);
	sprintf(path, "$.%s", key);

	if (mjson_get_number(data, json_size, path, &ret)) {
		if (std::floor(ret) == ret) {
			snprintf((char*)buf, buf_size, "%d", (int)ret);
		} else {
			snprintf((char*)buf, buf_size, "%g", ret);
		}

		free(path);
		return buf;
	}

	free(path);
	return (char*)PARSE_ERROR;
}


char* parse_frozen(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	char* ret = buf;
	char* query = (char*)malloc(key_size + 8);
	json_out out = JSON_OUT_BUF(buf, (size_t)buf_size);

	double value_dbl;
	char* value_str;
	bool value_bool;
	int err = -1;

	sprintf(query, "{%s: %%lf}", key);
	if (json_scanf(data, json_size, query, &value_dbl) > 0) {
		json_printf(&out, "%g", value_dbl);
		goto end;
	}

	sprintf(query, "{%s: %%B}", key);
	if (json_scanf(data, json_size, query, &value_bool) > 0) {
		json_printf(&out, "%B", value_bool);
		goto end;
	}

	sprintf(query, "{%s: %%Q}", key);
	if (json_scanf(data, json_size, query, &value_str) > 0) {
		json_printf(&out, "%Q", value_dbl);
		free(value_str);
		goto end;
	}

	ret = (char*)PARSE_ERROR;

end:
	free(query);

	if (err == 0) {
		ret = (char*)KEY_NOT_FOUND;
	}

	return ret;
}

jsmntok_t jsmn_tokens[4096];
jsmn_parser parser_jsmn;

char* parse_jsmn(char* data, int json_size, char* query_key, int key_size, char* buf, int buf_size) {
	jsmn_init(&parser_jsmn);

	int parsed_n = jsmn_parse(&parser_jsmn, data, json_size, jsmn_tokens, 4096);

	if (parsed_n < 0) {
		return (char*)PARSE_ERROR;
	}

	if (parsed_n == 0) {
		return (char*)KEY_NOT_FOUND;
	}

	if (jsmn_tokens[0].type != JSMN_OBJECT) {
		return (char*)KEY_NOT_FOUND;
	}

	int offset = 1;
	jsmntok_t* ret_val = NULL;

	for (int i = 0; i < jsmn_tokens[0].size; ++i) {
		jsmntok_t* key = &jsmn_tokens[offset++];

		if (offset >= parsed_n) {
			return (char*)PARSE_ERROR;
		}

		jsmntok_t* value = &jsmn_tokens[offset++];

		if (key->type != JSMN_STRING || value->type != JSMN_PRIMITIVE) {
			return (char*)PARSE_ERROR;
		}

		int key_len = key->end - key->start;

		if (key_len + 1 > (int)buf_size) {
			return (char*)PARSE_ERROR;
		}

		memcpy(buf, data + key->start, key_len);
		buf[key_len] = 0;

		if (std::strcmp(buf, query_key) == 0) {
			ret_val = value;
		}
	}

	if (ret_val != NULL) {
		int j = ret_val->start;
		char first = data[j];

		if (first == 'n') {
			snprintf((char*)buf, buf_size, "null");
		} else if (first == 'f') {
			snprintf((char*)buf, buf_size, "false");
		} else if (first == 't') {
			snprintf((char*)buf, buf_size, "true");
		} else if (first == '-' || (first >= '0' && first <= '9')) {
			int val_len = ret_val->end - j;

			if (val_len > (int)buf_size) {
				return (char*)PARSE_ERROR;
			}

			memcpy(buf, data + j, val_len);
			buf[val_len] = 0;

			try {
				double d = std::stod(buf);
				snprintf((char*)buf, buf_size, "%g", d);
			} catch (const std::out_of_range& e) {
				return (char*)PARSE_ERROR;
			} catch (const std::invalid_argument& e) {
				return (char*)PARSE_ERROR;
			}
		} else {
			buf = (char*)PARSE_ERROR;
		}

		return buf;
	}

	return (char*)PARSE_ERROR;
}


Poco::JSON::Parser parser_poco;

char* parse_poco(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	try {
	    parser_poco.reset();
		Poco::Dynamic::Var result = parser_poco.parse(data);
		Poco::JSON::Object::Ptr object = result.extract<Poco::JSON::Object::Ptr>();
		double value = object->getValue<double>(key);

		snprintf((char*)buf, buf_size, "%g", value);
	} catch (const std::exception& e) {
		return (char*)PARSE_ERROR;
	}

	return buf;
}

char* parse_boost(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	boost::system::error_code ec;
	auto value = boost::json::parse(data, ec);

	if (ec) {
		return (char*)PARSE_ERROR;
	}

	if (auto object = value.if_object()) {
		if (!object->contains(key)) {
			return (char*)KEY_NOT_FOUND;
		}

		boost::json::value val = object->at(key);

		if (val.is_number()) {
			if (val.is_int64()) {
				snprintf((char*)buf, buf_size, "%lld", (long long)val.get_int64());
			} else if (val.is_double()) {
				snprintf((char*)buf, buf_size, "%g", val.get_double());
			} else {
				return (char*)PARSE_ERROR;
			}

			return buf;
		}
	}

	return (char*)PARSE_ERROR;
}


char* parse_nlohmann(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	try {
		nlohmann::json parsed = nlohmann::json::parse(data);

		if (!parsed.contains(key)) {
			return (char*)KEY_NOT_FOUND;
		}

		// if (parsed[key].is_string()) {
		// 	snprintf((char*)buf, buf_size, "%s", (char*)parsed[key]);
		// 	return buf;
		// }

		if (parsed[key].is_number()) {
			snprintf((char*)buf, buf_size, "%g", (double)parsed[key]);
			return buf;
		}
	} catch(const std::exception& e) {
	}

	return (char*)PARSE_ERROR;
}


char* parse_mongoose(char* data, int json_size, char* key, int key_size, char* buf, int buf_size) {
	mg_str json = mg_str_n(data, json_size);
	double num_val;
	bool bool_val;
	char* query = (char*)malloc(key_size + 8);
	char* ret = (char*)PARSE_ERROR;

	sprintf(query, "$.%s", key);

	int toklen;
	if (mg_json_get(json, query, &toklen) == MG_JSON_NOT_FOUND) {
		ret = (char*)KEY_NOT_FOUND;
	} else {
		if (mg_json_get_num(json, query, &num_val)) {
			snprintf((char*)buf, buf_size, "%g", num_val);
			ret = buf;
		}

		if (mg_json_get_bool(json, query, &bool_val)) {
			if (bool_val) snprintf((char*)buf, buf_size, "true");
			else snprintf((char*)buf, buf_size, "false");
			ret = buf;
		}

		char* str_val = mg_json_get_str(json, query);
		if (str_val) {
			snprintf((char*)buf, buf_size, "%s", str_val);
			ret = buf;
		}
	}

	free(query);
	return ret;
}


int main(int argc, char* argv[]) {
	// char* asdf = (char*)"{\"4294967295\":2,\"0\":3}";
	// char* key0 = (char*)"4294967295";
	// char buf[1000];
	// std::cout << "adf " << parse_cjson(asdf, strlen(asdf), key0, strlen(key0), buf, 1000) << "\n";
    int parser_number = 0;

    if (argc == 2) {
        parser_number = atoi(argv[1]);
    }

	// char* buf[1024];
	// start_coverage();
	// parse_mjson((char*)"{\"q\":1}", 7, (char*)"q", 1, (char*)buf, 1024);
	// stop_coverage();
	// printf("ret: %s\n", (char*)buf);
	// // int coverage_start = coverage->front();
	// // int coverage_end = coverage->back();
	// printf("Coverage: ");
	// // for (auto c : coverage_buf_0) {
	// for (int i = 0; i < coverage_i0; ++i) {
	// 	int c = coverage_buf0[i];
	// 	printf("%x ", c);
	// 	// printf("%d ", c - coverage_start);
	// }
	// printf("\n");

    char* parser_name;
	char* (*parser_fn)(char*, int, char*, int, char*, int);

    switch (parser_number) {
        case 0:
            parser_name = (char*)"cpp_yajl";
			parser_fn = &parse_yajl;
            break;
        case 1:
            parser_name = (char*)"c_mjson";
			parser_fn = &parse_mjson;
            break;
        case 2:
            parser_name = (char*)"c_cjson";
			parser_fn = &parse_cjson;
            break;
        case 3:
            parser_name = (char*)"c_json_parser";
			parser_fn = &parse_json_parser;
            break;
        case 4:
            parser_name = (char*)"c_jansson";
			parser_fn = &parse_jansson;
            break;
        case 5:
            parser_name = (char*)"c_frozen";
			parser_fn = &parse_frozen;
            break;
        case 6:
            parser_name = (char*)"c_jsmn";
			parser_fn = &parse_jsmn;
            break;
        case 7:
            parser_name = (char*)"c_json_c";
			parser_fn = &json_c_wrapper::parse_json_c;
            break;
        case 8:
            parser_name = (char*)"cpp_poco";
			parser_fn = &parse_poco;
            break;
        case 9:
            parser_name = (char*)"cpp_boost";
			parser_fn = &parse_boost;
            break;
        case 10:
            parser_name = (char*)"cpp_nlohmann";
			parser_fn = &parse_nlohmann;
            break;
        case 11:
            parser_name = (char*)"cpp_mongoose";
			parser_fn = &parse_mongoose;
            break;
        default:
            return 1;
    }

    int sock;
    struct sockaddr_un addr;

    // Wait for connection
    while (1) {
        sock = socket(AF_UNIX, SOCK_STREAM, 0);

        if (sock < 0) {
            return 1;
        }

		memset(&addr, 0, sizeof(struct sockaddr_un));
        addr.sun_family = AF_UNIX;
        strcpy(addr.sun_path, "/tmp/fuzzer.sock");
        // inet_pton(AF_INET, SERVER_ADDR, &addr.sin_addr);

        if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) == 0) {
            break;
        }

        close(sock);
        struct timespec ts = {0, 100 * 1000000};
        nanosleep(&ts, NULL);
    }

    // int flag = 1;
    // setsockopt(sock, IPPROTO_TCP, TCP_NODELAY, &flag, sizeof(flag));

    // Send parser name
    char info_buf[65] = {0};
    memcpy(info_buf, parser_name, strlen(parser_name));
	uint8_t parser_flags = 0b00000001;
	info_buf[64] = parser_flags;

    if (send_all(sock, info_buf, 65) < 0) {
        perror("Could not send header");
        return 1;
    }

    uint8_t* read_buffer = (uint8_t*)malloc(1 << 20);
    uint8_t* write_buffer = (uint8_t*)malloc(1 << 22);
    size_t read_size = 0;
    size_t write_size = 0;
    char* message = NULL;
	uint32_t key_len = 0;
	char* key = (char*)malloc(key_len + 1);
	size_t json_len = 128;
	char* json_str = (char*)malloc(json_len);
	json_str[0] = 0;
	key[0] = 0;

    uint8_t* header = (uint8_t*)malloc(9);
	char message_buf[1 << 12];

    while (1) {
        if (recv_all(sock, header, 9) < 0) {
            break;
        }

        uint32_t input_buffer_size = *((uint32_t*)&header[0]);
        // char datatype = *((uint32_t*)&header[4]);
		uint32_t new_key_len = *((uint32_t*)&header[5]);

		if (key_len != new_key_len) {
			key_len = new_key_len;
			key = (char*)realloc(key, key_len + 1);
		}

        if (recv_all(sock, key, key_len) < 0) {
            break;
        }

		key[key_len] = 0;

		// std::printf("header %d %d %s\n", input_buffer_size, key_len, key);

        // uint32_t buffer_size = *((uint32_t*)&header[0]);
        // uint16_t payload_size = *((uint16_t*)&header[4]);
        // uint32_t batch_size = *((uint32_t*)&header[6]);
        // char mode = (Mode)header[10];

        // size_t total_payload = (size_t)payload_size * batch_size;
		size_t send_size = input_buffer_size << 2;

        if (read_size < input_buffer_size) {
            read_buffer = (uint8_t*)realloc(read_buffer, input_buffer_size);
			read_size = input_buffer_size;
        }

        if (write_size < send_size) {
            write_buffer = (uint8_t*)realloc(write_buffer, send_size);
			write_size = send_size;
        }

        if (recv_all(sock, read_buffer, input_buffer_size) < 0) {
            printf("C/C++ Client: Connection closed (read)\n");
            break;
        }

		size_t read_offset = 0;
        size_t write_offset = 4;
		// char* json_str = (char*)malloc(payload_size + 1);

        // for (uint32_t i = 0; i < batch_size; i++) {
		while (read_offset < input_buffer_size) {
			uint16_t new_json_len = *((uint16_t*)&read_buffer[read_offset]);
			read_offset += 2;

			if (json_len < new_json_len) {
				json_str = (char*)realloc(json_str, new_json_len + 1);
				json_len = new_json_len;
			}

            char* data = (char*)read_buffer + read_offset;
			strncpy(json_str, data, new_json_len);
			json_str[new_json_len] = 0;
			read_offset += new_json_len;

			auto start = std::chrono::high_resolution_clock::now();
			// start_coverage();
			message = parser_fn(json_str, new_json_len, key, key_len, message_buf, sizeof(message_buf));
			// stop_coverage();

			//int coverage_start = coverage.front();
			//int coverage_end = coverage.back();
			// printf("Coverage: ");
			// for (auto c : *coverage) {
			// 	printf("%d ", c);
			// 	// printf("%d ", c - coverage_start);
			// }
			// printf("\n");

			// if (mode == Mode::Time) {
			// std::cout << json_str << " -> " << message << ": " << ns << std::endl;
			auto end = std::chrono::high_resolution_clock::now();
			uint64_t elapsed = std::chrono::duration_cast<std::chrono::nanoseconds>(end - start).count() / 10;
			uint32_t dur = (uint32_t)elapsed;
			// dur /= 10;
			// snprintf((char*)buf, buf_size, "%lld", (unsigned long long)ns);
			// message = buf;
			// }

			uint16_t msg_len = (uint16_t)strlen(message);

			while (write_size < write_offset + 10 + msg_len + coverage_n) {
				write_size *= 2;
				write_buffer = (uint8_t*)realloc(write_buffer, write_size);
			}

			memcpy(write_buffer + write_offset, &dur, 4);
			write_offset += 4;

			if (new_coverage_info) {
				new_coverage_info = false;

				memcpy(write_buffer + write_offset, &coverage_n, 2);
				write_offset += 2;

				memcpy(write_buffer + write_offset, coverage_buf, coverage_n);
				write_offset += coverage_n;

				// uint32_t cov_hash = coverage_hash();
				// memcpy(write_buffer + write_offset, &cov_hash, 4);
				// write_offset += 4;
			} else {
				write_buffer[write_offset++] = 0;
				write_buffer[write_offset++] = 0;
			}

			memcpy(write_buffer + write_offset, &msg_len, 2);
			write_offset += 2;

			memcpy(write_buffer + write_offset, message, msg_len);
			write_offset += msg_len;
        }

		// std::printf("red %d\n", n);

        uint32_t payload_len = write_offset - 4;
        memcpy(write_buffer, &payload_len, 4);

        if (send_all(sock, write_buffer, write_offset) < 0) {
            perror("C/C++ Client: Write error\n");
            break;
        }
    }

	// delete coverage;
    free(json_str);
	free(key);
	free(header);
    free(read_buffer);
    free(write_buffer);
    close(sock);
	json_c_wrapper::json_c_free();

    return 0;
}
