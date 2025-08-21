import socket
import struct
import json
import simplejson
import time
import sys
import orjson
import ujson
import msgspec


KEY_NOT_FOUND = 'KEY_NOT_FOUND'
PARSE_ERROR = 'PARSE_ERROR'


def parse_json(data, key):
    try:
        parsed = json.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return str(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_simplejson(data, key):
    try:
        # parsed = simplejson.load(data)
        parsed = simplejson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return str(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_ujson(data, key):
    try:
        # parsed = simplejson.load(data)
        parsed = ujson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return str(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


def parse_orjson(data, key):
    try:
        parsed = orjson.loads(data.rstrip(b'\0').decode('utf-8'))
        if key in parsed:
            return str(parsed[key])
        else:
            return KEY_NOT_FOUND
    except Exception:
        return PARSE_ERROR


class Query(msgspec.Struct):
    q: int


def parse_msgspec(data):
    try:
        query = msgspec.json.decode(data, type=Query)
        return str(query.q)

    except Exception:
        return PARSE_ERROR


def main():
    parser_number = 0

    if len(sys.argv) == 2:
        parser_number = int(sys.argv[1])

    match parser_number:
        case 0:
            name = b'python_json'
        case 1:
            name = b'python_simplejson'
        case 2:
            name = b'python_orjson'
        case 3:
            name = b'python_ujson'
        case 4:
            name = b'python_msgspec'
        case _:
            sys.exit(1)

    while True:
        try:
            s = socket.create_connection(('127.0.0.1', 5000))
            break
        except OSError:
            time.sleep(0.1)

    s.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)

    name_buffer = name.ljust(64, b'\0')
    s.sendall(name_buffer)

    read_buffer = bytearray()
    write_buffer = bytearray()

    while True:
        # Read header (8 bytes)
        header = s.recv(8)
        if len(header) < 8:
            return

        buffer_size, payload_size, batch_size = struct.unpack('<IHH', header)

        total_payload = batch_size * payload_size

        # Resize buffers if needed
        if len(read_buffer) != total_payload:
            read_buffer = bytearray(total_payload)
        if len(write_buffer) != buffer_size:
            write_buffer = bytearray(buffer_size)

        # Read JSON payloads
        view = memoryview(read_buffer)
        total_read = 0
        while total_read < total_payload:
            n = s.recv_into(view[total_read:], total_payload - total_read)
            if n == 0:
                return
            total_read += n

        byte_offset = 4  # reserve first 4 bytes for length

        for batch in range(batch_size):
            start = batch * payload_size
            end = (batch + 1) * payload_size
            data = read_buffer[start:end]

            # Parse JSON
            match parser_number:
                case 0:
                    message = parse_json(data, 'q')
                case 1:
                    message = parse_simplejson(data, 'q')
                case 2:
                    message = parse_orjson(data, 'q')
                case 3:
                    message = parse_ujson(data, 'q')
                case 4:
                    message = parse_msgspec(data)

            # Write message length (u16 LE) + message bytes
            size_bytes = struct.pack('<H', len(message))
            write_buffer[byte_offset:byte_offset + 2] = size_bytes
            byte_offset += 2

            msg_bytes = message.encode('utf-8')
            write_buffer[byte_offset:byte_offset + len(msg_bytes)] = msg_bytes
            byte_offset += len(msg_bytes)

        # Write payload size into first 4 bytes
        write_buffer[0:4] = struct.pack('<I', byte_offset - 4)

        # Send back result
        s.sendall(write_buffer[:byte_offset])


if __name__ == '__main__':
    main()
