#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <arpa/inet.h>
#include <netinet/tcp.h>
#include <sys/socket.h>
// #include <errno.h>
#include <stdint.h>
#include <time.h>

#include <jansson.h>
#include "cJSON.h"
#include "json_parser.h"

#define SERVER_ADDR "127.0.0.1"
#define SERVER_PORT 5000
// #define NAME "c_cjson"
#define KEY_NOT_FOUND "KEY_NOT_FOUND"
#define PARSE_ERROR "PARSE_ERROR"

char* return_buffer[256];


int recv_all(int sock, void *buf, size_t len) {
    size_t received = 0;
    while (received < len) {
        ssize_t n = recv(sock, (char*)buf + received, len - received, 0);
        if (n < 0) return -1;
        received += n;
    }
    return 0;
}

int send_all(int sock, const void *buf, size_t len) {
    size_t sent = 0;
    while (sent < len) {
        ssize_t n = send(sock, (char*)buf + sent, len - sent, 0);
        if (n < 0) return -1;
        sent += n;
    }
    return 0;
}

char* parse_cjson(uint8_t* data, char* key) {
    cJSON *parsed = cJSON_Parse((char*)data);
    if (parsed) {
        cJSON *q = cJSON_GetObjectItemCaseSensitive(parsed, "q");
        char* ret;

        if (cJSON_IsNumber(q) && q->valuedouble) {
            if (q->valuedouble == floorf(q->valuedouble)) {
                snprintf((char*)return_buffer, sizeof(return_buffer), "%d", q->valueint);
            } else {
                snprintf((char*)return_buffer, sizeof(return_buffer), "%f", q->valuedouble);
            }
            ret = (char*)return_buffer;
        } else if (cJSON_IsString(q) && q->valuestring) {
            ret = strdup(q->valuestring);
        } else {
            ret = KEY_NOT_FOUND;
        }

        cJSON_Delete(parsed);
        return ret;
    } else {
        return PARSE_ERROR;
    }
}

char* parse_jansson(char* data, char* key) {
    json_t *root;
    json_error_t error;

    root = json_loads(data, 0, &error);

    if (root) {
        char* ret = PARSE_ERROR;

        if (json_is_object(root)) {
            json_t* query = json_object_get(root, key);

            if (query) {
                if (json_is_string(query)) {
                    ret = strdup(json_string_value(query));
                } else if (json_is_integer(query)) {
                    snprintf((char*)return_buffer, sizeof(return_buffer), "%lld", json_integer_value(query));
                    ret = (char*)return_buffer;
                } else  if (json_is_real(query)) {
                    snprintf((char*)return_buffer, sizeof(return_buffer), "%f", json_real_value(query));
                    ret = (char*)return_buffer;
                }
            } else {
                ret = KEY_NOT_FOUND;
            }
        }

        json_decref(root);
        return ret;
    } else {
        return PARSE_ERROR;
    }
}

int main(int argc, char* argv[]) {
    int parser_number = 0;

    if (argc == 2) {
        parser_number = atoi(argv[1]);
    }

    char* parser_name;

    switch (parser_number) {
        case 0:
            parser_name = "c_cjson";
            break;
        case 1:
            parser_name = "c_json_parser";
            break;
        case 2:
            parser_name = "c_jansson";
            break;
        default:
            // perror("Invalid parser number in arguments");
            return 1;
    }

    int sock;
    struct sockaddr_in addr;

    // Wait for connection
    while (1) {
        sock = socket(AF_INET, SOCK_STREAM, 0);

        if (sock < 0) {
            return 1;
        }

        addr.sin_family = AF_INET;
        addr.sin_port = htons(SERVER_PORT);
        inet_pton(AF_INET, SERVER_ADDR, &addr.sin_addr);

        if (connect(sock, (struct sockaddr*)&addr, sizeof(addr)) == 0) {
            break;
        }

        close(sock);
        struct timespec ts = {0, 100 * 1000000};
        nanosleep(&ts, NULL);
    }

    int flag = 1;
    setsockopt(sock, IPPROTO_TCP, TCP_NODELAY, &flag, sizeof(flag));

    // Send parser name
    char name_buf[64] = {0};
    memcpy(name_buf, parser_name, strlen(parser_name));

    if (send_all(sock, name_buf, 64) < 0) {
        perror("Could not send name");
        return 1;
    }

    uint8_t* read_buffer = NULL;
    uint8_t* write_buffer = NULL;
    int read_size = -1;
    int write_size = -1;
    char* message = NULL;

    while (1) {
        uint8_t header[8];

        if (recv_all(sock, header, 8) < 0) {
            // printf("Connection closed (header)\n");
            break;
        }

        uint32_t buffer_size = *((uint32_t*)&header[0]);
        uint16_t payload_size = *((uint16_t*)&header[4]);
        uint16_t batch_size = *((uint16_t*)&header[6]);

        size_t total_payload = (size_t)payload_size * batch_size;

        if (read_size != total_payload) {
            read_buffer = realloc(read_buffer, total_payload);
        }

        if (write_size != buffer_size) {
            write_buffer = realloc(write_buffer, buffer_size);
        }

        if (recv_all(sock, read_buffer, total_payload) < 0) {
            printf("Connection closed (read)\n");
            break;
        }

        size_t byte_offset = 4;

        for (int i = 0; i < batch_size; i++) {
            uint8_t* data = read_buffer + i * payload_size;

            char* json_str = strndup((char*)data, payload_size);

            switch (parser_number) {
                case 0:
                    message = parse_cjson((uint8_t*)json_str, "q");
                    break;
                case 1:
                    message = parse_json_parser(json_str, "q");
                    break;
                case 2:
                    message = parse_jansson(json_str, "q");
                    break;
            }

            uint16_t msg_len = (uint16_t)strlen(message);
            // uint16_t msg_len = htons(strlen(message));
            memcpy(write_buffer + byte_offset, &msg_len, 2);
            byte_offset += 2;

            memcpy(write_buffer + byte_offset, message, msg_len);
            byte_offset += msg_len;
        }

        // uint32_t payload_len = htonl((byte_offset - 4));
        uint32_t payload_len = byte_offset - 4;
        memcpy(write_buffer, &payload_len, 4);

        if (send_all(sock, write_buffer, byte_offset) < 0) {
            perror("C Client: Write error\n");
            break;
        }
    }

    free(message);
    free(read_buffer);
    free(write_buffer);
    close(sock);

    return 0;
}
