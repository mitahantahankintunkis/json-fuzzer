import socket
import struct
import json
import simplejson
import time
import sys
import orjson
import ujson
import msgspec
import rapidjson


KEY_NOT_FOUND = 'KEY_NOT_FOUND'
PARSE_ERROR = 'PARSE_ERROR'


def parse_json(data, key):
    try:
        parsed = json.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return json.dumps(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_simplejson(data, key):
    try:
        # parsed = simplejson.load(data)
        parsed = simplejson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return simplejson.dumps(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_ujson(data, key):
    try:
        # parsed = simplejson.load(data)
        parsed = ujson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return ujson.dumps(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_orjson(data, key):
    try:
        parsed = orjson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return orjson.dumps(parsed[key]).decode('utf-8')
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


class Query(msgspec.Struct):
    q: float


def parse_msgspec(data, key):
    try:
        if key == 'q':
            query = msgspec.json.decode(data, type=Query)

            if query.q == int(query.q):
                return json.dumps(int(query.q))

            return json.dumps(query.q)
        else:
            return PARSE_ERROR

    except Exception:
        return PARSE_ERROR


def parse_rapidjson(data, key):
    try:
        query = rapidjson.loads(data)
        return json.dumps(query[key])

    except Exception:
        return PARSE_ERROR


def main():
    parser_number = 0

    if len(sys.argv) == 2:
        parser_number = int(sys.argv[1])

    match parser_number:
        case 0:
            name = b'python_json'
            parser_fn = parse_json
        case 1:
            name = b'python_simplejson'
            parser_fn = parse_simplejson
        case 2:
            name = b'python_orjson'
            parser_fn = parse_orjson
        case 3:
            name = b'python_ujson'
            parser_fn = parse_ujson
        case 4:
            name = b'python_msgspec'
            parser_fn = parse_msgspec
        case 5:
            name = b'python_rapidjson'
            parser_fn = parse_rapidjson
        case _:
            sys.exit(1)

    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)

    while True:
        try:
            s.connect('/tmp/fuzzer.sock')
            # s = socket.create_connection(('127.0.0.1', 5000))
            # s.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
            break
        except OSError:
            time.sleep(0.1)

    info_buf = name.ljust(65, b'\0')
    s.sendall(info_buf)

    read_buffer = bytearray()
    write_buffer = bytearray()

    while True:
        header = s.recv(9)
        if len(header) < 9:
            return

        input_buffer_size, _, key_len = struct.unpack('<IBI', header)
        max_len = max(input_buffer_size, key_len)

        # Resize buffers if needed
        if len(read_buffer) < max_len:
            read_buffer = bytearray(max_len)
        if len(write_buffer) < input_buffer_size << 2:
            write_buffer = bytearray(input_buffer_size << 2)

        view = memoryview(read_buffer)
        total_read = 0
        while total_read < key_len:
            n = s.recv_into(view[total_read:], key_len - total_read)
            if n == 0:
                return
            total_read += n

        key = bytes(read_buffer[0:key_len]).decode()

        total_read = 0
        while total_read < input_buffer_size:
            n = s.recv_into(view[total_read:], input_buffer_size - total_read)
            if n == 0:
                return
            total_read += n

        read_offset = 0
        write_offset = 4

        while read_offset < input_buffer_size:
            json_size = struct.unpack('<H', read_buffer[read_offset:read_offset + 2])[0]
            read_offset += 2

            data = read_buffer[read_offset:read_offset + json_size]
            read_offset += json_size

            start = time.perf_counter_ns()
            message = parser_fn(data, key)
            end = time.perf_counter_ns()
            ns = (end - start) / 10

            ns_bytes = struct.pack('<I', int(ns))
            write_buffer[write_offset:write_offset + 4] = ns_bytes
            write_offset += 4

            msg_bytes = message.encode('utf-8')

            if len(message) > (1 << 16) - 1:
                message = message[:(1 << 16) - 1]

            size_bytes = struct.pack('<H', len(msg_bytes))

            write_buffer[write_offset:write_offset + 2] = size_bytes
            write_offset += 2

            write_buffer[write_offset:write_offset + len(msg_bytes)] = msg_bytes
            write_offset += len(msg_bytes)

        write_buffer[0:4] = struct.pack('<I', write_offset - 4)
        s.sendall(write_buffer[:write_offset])


if __name__ == '__main__':
    main()
