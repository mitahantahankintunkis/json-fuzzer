package main

import (
	"encoding/binary"
	std_json "encoding/json"
	"fmt"
	"io"
	"net"
	"os"
	"strconv"
	"time"

	"github.com/buger/jsonparser"
	"github.com/francoispqt/gojay"
	"github.com/json-iterator/go"
	// "github.com/minio/simdjson-go"
	// "github.com/ohler55/ojg/oj"
	"github.com/sugawarayuuta/sonnet"
	"github.com/tidwall/gjson"
	"github.com/valyala/fastjson"
)

var PARSE_ERROR = []byte("PARSE_ERROR")
var KEY_NOT_FOUND = []byte("KEY_NOT_FOUND")
var CRITICAL_ERROR = []byte("CRITICAL_ERROR")

type query struct {
	q *float64
}

// implement gojay.UnmarshalerJSONObject
func (q *query) UnmarshalJSONObject(dec *gojay.Decoder, key string) error {
	switch key {
	case "q":
		return dec.Float64Null(&q.q)
	}
	return nil
}

func (q *query) NKeys() int {
	return 3
}

type std_query struct {
	Q *float64 `json:"q"`
}

var std_decoded std_query

func std_parser(encoded []byte, key string) []byte {
	if key == "q" {
		std_decoded.Q = nil
		err := std_json.Unmarshal(encoded, &std_decoded)

		if err == nil {
			if std_decoded.Q != nil {
				val, err := std_json.Marshal(std_decoded.Q)

				if err == nil {
					return val
				}
			}
		}
	}

	var decoded map[string]any
	err := std_json.Unmarshal(encoded, &decoded)

	if err != nil {
		return PARSE_ERROR
	}
	parsed, ok := decoded[key]

	if !ok {
		return KEY_NOT_FOUND
	}

	val, err := std_json.Marshal(parsed)
	if err != nil {
		return PARSE_ERROR
	}

	return val
}

func francoispqt_gojay_parser(encoded []byte, key string) []byte {
	// if key == "q" {
	decoded := &query{}
	err := gojay.UnmarshalJSONObject(encoded, decoded)

	if err != nil {
		return PARSE_ERROR
	}

	if decoded.q == nil {
		return KEY_NOT_FOUND
	}

	// Gojay marshal does not work for some reason
	val, err := std_json.Marshal(decoded.q)

	if err != nil {
		return PARSE_ERROR
	}
	return val

	// }

	// var decoded map[string]any
	// err := gojay.Unmarshal(encoded, &decoded)
	//
	// if err != nil {
	// 	return PARSE_ERROR
	// }
	//
	// parsed, ok := decoded[key]
	//
	// if !ok {
	// 	return KEY_NOT_FOUND
	// }
	//
	// val, err := gojay.Marshal(parsed)
	//
	// if err != nil {
	// 	return PARSE_ERROR
	// }
	//
	// return val
}

var json_iterator_decoded std_query

func json_iterator_parser(encoded []byte, key string) []byte {
	if key == "q" {
		json_iterator_decoded.Q = nil
		var json = jsoniter.ConfigCompatibleWithStandardLibrary
		err := json.Unmarshal(encoded, &json_iterator_decoded)

		if err == nil && json_iterator_decoded.Q != nil {
			val, err := json.Marshal(json_iterator_decoded.Q)

			if err == nil {
				return val
			}
		}
	}

	var decoded map[string]any
	var json = jsoniter.ConfigCompatibleWithStandardLibrary
	err := json.Unmarshal(encoded, &decoded)
	// err := std_json.Unmarshal(encoded, &decoded)

	if err != nil {
		return PARSE_ERROR
	}

	parsed, ok := decoded[key]

	if !ok {
		return KEY_NOT_FOUND
	}

	val, err := json.Marshal(parsed)

	if err != nil {
		return PARSE_ERROR
	}

	return val
}

func tidwall_gjson_parser(encoded []byte, key string) []byte {
	value := gjson.Get(string(encoded[:]), key)

	if !value.Exists() {
		return KEY_NOT_FOUND
	}

	return []byte(value.String())
}

// GJSON parser with optional validation
func tidwall_gjson_safe_parser(encoded []byte, key string) []byte {
	json := string(encoded)

	if !gjson.Valid(json) {
		return PARSE_ERROR
	}

	value := gjson.Get(json, key)

	if !value.Exists() {
		return KEY_NOT_FOUND
	}

	return []byte(value.String())
}

func buger_jsonparser_parser(encoded []byte, key string) []byte {
	if value, err := jsonparser.GetInt(encoded, key); err == nil {
		return fmt.Append(nil, value)
	}

	if value, err := jsonparser.GetFloat(encoded, key); err == nil {
		return fmt.Append(nil, value)
	}

	if value, err := jsonparser.GetBoolean(encoded, key); err == nil {
		return fmt.Append(nil, value)
	}

	if value, err := jsonparser.GetString(encoded, key); err == nil {
		return fmt.Append(nil, "\"", value, "\"")
	}

	return PARSE_ERROR
}

// var simdjson_iter *simdjson.Iter
// var simdjson_obj *simdjson.Object
// var simdjson_parsed simdjson.ParsedJson
// var simdjson_element simdjson.Element

// func minio_simdjson_parser(encoded []byte, key string) []byte {
// 	parsed, err := simdjson.Parse(encoded, &simdjson_parsed)
//
// 	if err != nil {
// 		return PARSE_ERROR
// 	}
//
// 	iter := parsed.Iter()
//
// 	typ := iter.Advance()
//
// 	switch typ {
// 	case simdjson.TypeRoot:
// 		if typ, simdjson_iter, err = iter.Root(simdjson_iter); err != nil {
// 			return PARSE_ERROR
// 		}
//
// 		if typ == simdjson.TypeObject {
// 			if simdjson_obj, err = simdjson_iter.Object(simdjson_obj); err != nil {
// 				return PARSE_ERROR
// 			}
//
// 			e := simdjson_obj.FindKey("q", &simdjson_element)
// 			if e != nil && simdjson_element.Type == simdjson.TypeInt {
// 				v, _ := simdjson_element.Iter.Int()
// 				return []byte(fmt.Sprint(v))
// 			}
// 		}
//
// 	default:
// 		return PARSE_ERROR
// 	}
//
// 	return PARSE_ERROR
// 	// fmt.Println(ret)
// 	//
// 	// return ret
// }

// func ohohler55_ojg_parser(encoded []byte, key string) string {
// 	var decoded map[string]interface{}
//
// 	err := oj.Unmarshal(encoded, &decoded)
//
// 	if err != nil {
// 		return "PARSE_ERROR"
// 	}
//
// 	val, ok := decoded["q"]
//
// 	if !ok {
// 		return "KEY_NOT_FOUND"
// 	}
//
// 	return fmt.Sprint(val)
// }

var fastjson_parser fastjson.Parser

func valyala_fastjson_parser(encoded []byte, key string) []byte {
	decoded, err := fastjson_parser.ParseBytes(encoded)

	if err != nil {
		return PARSE_ERROR
	}

	if !decoded.Exists(key) {
		return KEY_NOT_FOUND
	}

	value := decoded.Get(key)

	if val, err := value.Int64(); err == nil {
		return fmt.Append(nil, val)
	}

	if val, err := value.Float64(); err == nil {
		return fmt.Append(nil, val)
	}

	if val, err := value.Bool(); err == nil {
		return fmt.Append(nil, val)
	}

	return fmt.Append(nil, decoded.GetInt(key))
}

func sugawarayuuta_sonnet_parser(encoded []byte, key string) []byte {
	var decoded map[string]any
	err := sonnet.Unmarshal(encoded, &decoded)

	if err != nil {
		return PARSE_ERROR
	}

	val, ok := decoded[key]

	if !ok {
		return KEY_NOT_FOUND
	}

	ret, err := sonnet.Marshal(val)
	if err != nil {
		return PARSE_ERROR
	}
	return ret
	// return []byte(fmt.Sprint(val))
}

func recv_all(conn net.Conn, buf []byte, n int) error {
	byte_offset := 0

	for {
		received_bytes, err := conn.Read(buf[byte_offset:n])

		if err != nil {
			return err
		}

		byte_offset += received_bytes

		if byte_offset >= n {
			break
		}
	}

	return nil
}

func safe_parse(json []byte, key string, parser_fn func([]byte, string) []byte) (output []byte, dur int64) {
	defer func() {
		if r := recover(); r != nil {
			output = CRITICAL_ERROR
			dur = 0
		}
	}()

	start := time.Now()
	message := parser_fn(json, key)
	elapsed := time.Since(start)
	ns := elapsed.Nanoseconds()

	return message, ns
}

func main() {
	parser_number := 0

	if len(os.Args) > 1 {
		number, err := strconv.Atoi(os.Args[1])

		if err != nil {
			panic("Could not parse parser number")
		}

		parser_number = number
	}

	var parser_name string
	parser_fn := std_parser

	switch parser_number {
	case 0:
		parser_name = "go_std"
		parser_fn = std_parser
	case 1:
		parser_name = "go_gojay"
		parser_fn = francoispqt_gojay_parser
	case 2:
		parser_name = "go_json_iterator"
		parser_fn = json_iterator_parser
	case 3:
		parser_name = "go_gjson"
		parser_fn = tidwall_gjson_parser
	case 4:
		parser_name = "go_gjson_safe"
		parser_fn = tidwall_gjson_safe_parser
	case 5:
		parser_name = "go_jsonparser"
		parser_fn = buger_jsonparser_parser
	// case 6:
	// 	parser_name = "go_minio_simdjson"
	// case 7:
	// 	parser_name = "go-ohler55-ojg"
	case 6:
		parser_name = "go_fastjson"
		parser_fn = valyala_fastjson_parser
	case 7:
		parser_name = "go_sonnet"
		parser_fn = sugawarayuuta_sonnet_parser

	default:
		os.Exit(1)
	}

	// Connect to the server
	var conn net.Conn

	for {
		c, err := net.Dial("unix", "/tmp/fuzzer.sock")
		// c, err := net.Dial("tcp", "localhost:5000")

		if err != nil {
			time.Sleep(time.Millisecond * 100)
			continue
		}

		conn = c
		break
	}

	defer conn.Close()

	info_buf := make([]byte, 65)

	for i := range len(info_buf) {
		info_buf[i] = 0
	}

	copy(info_buf, []byte(parser_name))
	info_buf[64] = 0
	written_bytes, err := conn.Write(info_buf)

	if err != nil {
		fmt.Println(err)
		return
	}

	if written_bytes != len(info_buf) {
		fmt.Println("Could not write all name bytes")
		return
	}

	var read_buffer []byte = nil
	var write_buffer []byte = nil
	header := make([]byte, 9)

	for {
		// Read header
		err := recv_all(conn, header, 9)

		if err != nil {
			if err != io.EOF {
				fmt.Println(err)
			}
			return
		}

		input_buffer_size := int(binary.LittleEndian.Uint32(header[0:4]))
		key_len := int(binary.LittleEndian.Uint32(header[5:9]))
		max_len := max(key_len, input_buffer_size)

		if read_buffer == nil || len(read_buffer) < max_len {
			read_buffer = make([]byte, max_len)
		}

		write_buffer_size := int(max(2048, input_buffer_size<<2))
		if write_buffer == nil || len(write_buffer) < write_buffer_size {
			write_buffer = make([]byte, write_buffer_size)
		}

		recv_all(conn, read_buffer, key_len)
		key := string(read_buffer[0:key_len])

		recv_all(conn, read_buffer, input_buffer_size)

		write_offset := 4

		for read_offset := 0; read_offset < input_buffer_size; {
			json_size := int(binary.LittleEndian.Uint16(read_buffer[read_offset : read_offset+2]))
			read_offset += 2

			// fmt.Println(input_buffer_size, json_size, read_offset)
			json := read_buffer[read_offset : read_offset+json_size]
			read_offset += json_size

			message, ns := safe_parse(json, key, parser_fn)

			binary.LittleEndian.PutUint32(write_buffer[write_offset:], uint32(ns/10))
			write_offset += 4

			binary.LittleEndian.PutUint16(write_buffer[write_offset:], uint16(len(message)))
			write_offset += 2

			copy(write_buffer[write_offset:], message)
			write_offset += len(message)
		}

		binary.LittleEndian.PutUint32(write_buffer[0:4], uint32(write_offset-4))
		send_offset := 0

		// Send buffer
		for {
			sent_bytes, err := conn.Write(write_buffer[send_offset:write_offset])
			send_offset += sent_bytes

			if err != nil {
				fmt.Println(err)
				return
			}

			if sent_bytes >= write_offset {
				break
			}
		}
	}
}
