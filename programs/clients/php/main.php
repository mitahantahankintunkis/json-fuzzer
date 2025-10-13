<?php

const KEY_NOT_FOUND = "KEY_NOT_FOUND";
const PARSE_ERROR   = "PARSE_ERROR";

function read_all($socket, int $n) {
	$ret = "";

	while (strlen($ret) < $n) {
		$bytes = fread($socket, $n - strlen($ret));

		if ($bytes === false || strlen($bytes) === 0) {
			return $ret;
		}

		$ret .= $bytes;
	}

	return $ret;
}

$parse_std = function(string $data, string $key): string {
    $parsed = json_decode($data, true);

    if ($parsed === null && json_last_error() !== JSON_ERROR_NONE) {
        return PARSE_ERROR;
    }

    if (array_key_exists($key, $parsed)) {
        return strval($parsed[$key]);
    }

    return KEY_NOT_FOUND;
};

// Entry point
$parser_number = $argc === 2 ? intval($argv[1]) : 0;
/* $parser_fn = parse_std; */

switch ($parser_number) {
    case 0:
        $name = "php_std";
		$parser_fn = $parse_std;
        break;
    default:
        exit(1);
}

while (true) {
	/* $socket = @stream_socket_client("tcp://127.0.0.1:5000", $errno, $errstr, -1); */
	$socket = @stream_socket_client("unix:///tmp/fuzzer.sock", $errno, $errstr, -1);

    if ($socket) {
        stream_set_blocking($socket, true);
        break;
    }

    usleep(100000);
}

$name_buffer = str_pad($name, 64, "\0", STR_PAD_RIGHT);
fwrite($socket, $name_buffer);

$read_buffer  = "";
$write_buffer = "";

while (true) {
    $header = read_all($socket, 10);

    if ($header === false || strlen($header) < 10) {
        exit(0);
    }

    /* $buffer_size = unpack("V", substr($header, 0, 4))[1]; // little-endian u32 */
    /* $payload_size = unpack("v", substr($header, 4, 2))[1]; // little-endian u16 */
    /* $batch_size   = unpack("v", substr($header, 6, 2))[1]; // little-endian u16 */
    $buffer_size  = unpack("V", $header, 0)[1];
    $payload_size = unpack("v", $header, 4)[1];
    $batch_size   = unpack("V", $header, 6)[1];

    $read_length = $payload_size * $batch_size;
    $read_buffer = read_all($socket, $read_length);

    if ($read_buffer === false || strlen($read_buffer) < $read_length) {
		echo "PHP Client: Could not read body\n";
        exit(0);
    }

	if (strlen($write_buffer) < $buffer_size * 4) {
		$write_buffer = str_repeat("\0", $buffer_size * 4);
	}

    $byte_offset = 4;

    for ($batch = 0; $batch < $batch_size; ++$batch) {
        $data = substr($read_buffer, $batch * $payload_size, $payload_size);

		$parsed = $parser_fn($data, "q");

        $buffer_size = pack("v", strlen($parsed));
		for ($i = 0; $i < 2; ++$i) {
			$write_buffer[$byte_offset + $i] = $buffer_size[$i];
		}
        $byte_offset += 2;

		for ($i = 0; $i < strlen($parsed); ++$i) {
			$write_buffer[$byte_offset + $i] = $parsed[$i];
		}

        $byte_offset += strlen($parsed);
    }

    $prefix = pack("V", $byte_offset - 4);

	for ($i = 0; $i < 4; ++$i) {
		$write_buffer[$i] = $prefix[$i];
	}

    fwrite($socket, $write_buffer, $byte_offset);
}
