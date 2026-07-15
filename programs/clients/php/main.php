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

    if (is_array($parsed) && array_key_exists($key, $parsed)) {
        return @json_encode($parsed[$key]);
    }

    return KEY_NOT_FOUND;
};

// Entry point
$parser_number = $argc === 2 ? intval($argv[1]) : 0;

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
		stream_set_timeout($socket, 999999);
        break;
    }

    usleep(100000);
}

$info_buf = str_pad($name, 65, "\0", STR_PAD_RIGHT);
fwrite($socket, $info_buf);

$read_buffer  = "";
$write_buffer = "";

while (true) {
    $header = read_all($socket, 9);

    if ($header === false || strlen($header) < 9) {
        exit(0);
    }

    $input_buffer_size = unpack("V", $header, 0)[1];
    $key_len = unpack("V", $header, 5)[1];

    $key = read_all($socket, $key_len);
    $read_buffer = read_all($socket, $input_buffer_size);

    if (strlen($read_buffer) < $input_buffer_size) {
		echo "PHP Client: Could not read body\n";
        exit(0);
    }

	if (strlen($write_buffer) < $input_buffer_size * 4) {
		$write_buffer = str_repeat("\0", $input_buffer_size * 4);
	}

	$read_offset = 0;
    $write_offset = 4;

	while ($read_offset < $input_buffer_size) {
		$json_size = unpack("v", $read_buffer, $read_offset)[1];
		$read_offset += 2;
        $data = substr($read_buffer, $read_offset, $json_size);
		$read_offset += $json_size;

		$start = hrtime(true);
		$parsed = $parser_fn($data, $key);
		$end = hrtime(true);
		$ns = ($end - $start) / 10;

        $ns_bytes = pack("V", $ns);
		for ($i = 0; $i < 4; ++$i) {
			$write_buffer[$write_offset + $i] = $ns_bytes[$i];
		}

        $write_offset += 4;

        $buffer_size = pack("v", strlen($parsed));
		for ($i = 0; $i < 2; ++$i) {
			$write_buffer[$write_offset + $i] = $buffer_size[$i];
		}

        $write_offset += 2;

		for ($i = 0; $i < strlen($parsed); ++$i) {
			$write_buffer[$write_offset + $i] = $parsed[$i];
		}

        $write_offset += strlen($parsed);
    }

    $prefix = pack("V", $write_offset - 4);

	for ($i = 0; $i < 4; ++$i) {
		$write_buffer[$i] = $prefix[$i];
	}

    fwrite($socket, $write_buffer, $write_offset);
}
