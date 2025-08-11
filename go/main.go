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
	"github.com/minio/simdjson-go"
	// "github.com/ohler55/ojg/oj"
	"github.com/sugawarayuuta/sonnet"
	"github.com/tidwall/gjson"
	"github.com/valyala/fastjson"
)

var PARSE_ERROR = []byte("PARSE_ERROR")
var KEY_NOT_FOUND = []byte("KEY_NOT_FOUND")

type query struct {
	q *int
}

// implement gojay.UnmarshalerJSONObject
func (q *query) UnmarshalJSONObject(dec *gojay.Decoder, key string) error {
	switch key {
	case "q":
		return dec.IntNull(&q.q)
	}
	return nil
}

func (q *query) NKeys() int {
	return 3
}

type std_query struct {
	Q *int `json:"q"`
}

var std_decoded iter_query

func std_parser(encoded []byte) []byte {
	std_decoded.Q = nil

	// var decoded map[string]interface{}
	// err := std_json.Unmarshal(encoded, &decoded)
	// fmt.Println("Valid: ", std_json.Valid(encoded))
	// for k, m := range decoded {
	// 	fmt.Println(k, " ", m)
	// }

	err := std_json.Unmarshal(encoded, &std_decoded)

	if err != nil {
		return PARSE_ERROR
	}

	if std_decoded.Q == nil {
		return KEY_NOT_FOUND
	}

	return []byte(fmt.Sprint(*std_decoded.Q))

	// val, ok := decoded["q"]
	//
	// if !ok {
	// 	return KEY_NOT_FOUND
	// }
	// return []byte(fmt.Sprint(val))
}

func francoispqt_gojay_parser(encoded []byte) []byte {
	// decoded := query(make(map[string]interface{}))
	// var decoded map[string]interface{}
	// err := gojay.Unmarshal(payload, &decoded)
	decoded := &query{}
	err := gojay.UnmarshalJSONObject(encoded, decoded)

	if err != nil {
		return PARSE_ERROR
	}

	if decoded.q == nil {
		return KEY_NOT_FOUND
	}
	return []byte(fmt.Sprint(*decoded.q))
	// val, ok := decoded["q"]
	// fmt.Println(len(decoded))
	//
	// if ok {
	// message = []byte(fmt.Sprint(val))
	// } else {
	// 	message = []byte("KEY_NOT_FOUND")
	// }
}

type iter_query struct {
	Q *int `json:"q"`
}

var json_iterator_decoded iter_query

func json_iterator_parser(encoded []byte) []byte {
	// json_iterator_decoded.Q = nil

	// var decoded map[string]interface{}
	var json = jsoniter.ConfigCompatibleWithStandardLibrary
	err := json.Unmarshal(encoded, &json_iterator_decoded)

	if err != nil {
		return PARSE_ERROR
	}

	if json_iterator_decoded.Q == nil {
		return KEY_NOT_FOUND
	}

	return []byte(fmt.Sprint(*json_iterator_decoded.Q))

	// val, ok := decoded["q"]
	//
	// if !ok {
	// 	return KEY_NOT_FOUND
	// }

	// return []byte(fmt.Sprint(val))
}

func tidwall_gjson_parser(encoded []byte) []byte {
	value := gjson.Get(string(encoded[:]), "q")

	if !value.Exists() || value.Type != gjson.Number {
		return KEY_NOT_FOUND
	}

	fmt.Println("Valid: ", gjson.Valid(string(encoded)))

	return []byte(value.String())
}

func buger_jsonparser_parser(encoded []byte) []byte {
	value, err := jsonparser.GetInt(encoded, "q")
	// value := gjson.Get(string(payload[:]), "q")

	if err != nil {
		// fmt.Println(string(encoded), " ", string(PARSE_ERROR))
		return PARSE_ERROR
	}

	// fmt.Println(string(encoded), " ", value)

	return []byte(fmt.Sprint(value))

	// var decoded map[string]interface{}
	// var json = jsoniter.ConfigCompatibleWithStandardLibrary
	// err := json.Unmarshal(payload, &decoded)
	//
	// if err == nil {
	// 	val, ok := decoded["q"]
	//
	// 	if ok {
	// 		message = []byte(fmt.Sprint(val))
	// 	} else {
	// 		message = []byte("KEY_NOT_FOUND")
	// 	}
	// } else {
	// 	message = []byte("PARSE_ERROR")
	// }
}

var simdjson_iter *simdjson.Iter
var simdjson_obj *simdjson.Object
var simdjson_parsed simdjson.ParsedJson
var simdjson_element simdjson.Element

func minio_simdjson_parser(encoded []byte) []byte {
	parsed, err := simdjson.Parse(encoded, &simdjson_parsed)

	if err != nil {
		return PARSE_ERROR
	}

	iter := parsed.Iter()
	// elem, err := iter.FindElement(nil, "q")
	//
	// if err != nil {
	// 	return KEY_NOT_FOUND
	// }
	//
	// ret, err := elem.Iter.String()
	//
	// if err != nil {
	// 	return PARSE_ERROR
	// }

	typ := iter.Advance()

	switch typ {
	case simdjson.TypeRoot:
		if typ, simdjson_iter, err = iter.Root(simdjson_iter); err != nil {
			return PARSE_ERROR
		}

		if typ == simdjson.TypeObject {
			if simdjson_obj, err = simdjson_iter.Object(simdjson_obj); err != nil {
				return PARSE_ERROR
			}

			e := simdjson_obj.FindKey("q", &simdjson_element)
			if e != nil && simdjson_element.Type == simdjson.TypeInt {
				v, _ := simdjson_element.Iter.Int()
				return []byte(fmt.Sprint(v))
			}
		}

	default:
		return PARSE_ERROR
	}

	return PARSE_ERROR
	// fmt.Println(ret)
	//
	// return ret
}

// func ohohler55_ojg_parser(encoded []byte) string {
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

func valyala_fastjson_parser(encoded []byte) []byte {
	// return fmt.Sprint(fastjson.GetInt(encoded, "foo", "0"))
	decoded, err := fastjson_parser.ParseBytes(encoded)

	if err != nil {
		return PARSE_ERROR
	}

	if !decoded.Exists("q") {
		return KEY_NOT_FOUND
	}

	return []byte(fmt.Sprint(decoded.GetInt("q")))
}

func sugawarayuuta_sonnet_parser(encoded []byte) []byte {
	var decoded map[string]interface{}
	err := sonnet.Unmarshal(encoded, &decoded)

	if err != nil {
		return PARSE_ERROR
	}

	val, ok := decoded["q"]

	if !ok {
		return KEY_NOT_FOUND
	}

	return []byte(fmt.Sprint(val))
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

	switch parser_number {
	case 0:
		parser_name = "go-std"
	case 1:
		parser_name = "go-francoispqt-gojay"
	case 2:
		parser_name = "go-json-iterator"
	case 3:
		parser_name = "go-tidwall-gjson"
	case 4:
		parser_name = "go-buger-jsonparser"
	case 5:
		parser_name = "go-minio-simdjson"
	// case 6:
	// 	parser_name = "go-ohler55-ojg"
	case 6:
		parser_name = "go-valyala-fastjson"
	case 7:
		parser_name = "go-sugawarayuuta-sonnet"

	default:
		fmt.Println("Invalid parser number")
		os.Exit(1)
	}

	// Connect to the server
	var conn net.Conn

	for {
		c, err := net.Dial("tcp", "localhost:5000")
		// conn, err := net.Dial("tcp", "[::1]:5000")

		if err != nil {
			time.Sleep(time.Millisecond * 100)
			continue
			// fmt.Println(err)
			// return
		}

		conn = c
		break
	}

	defer conn.Close()

	name_buffer := make([]byte, 64)

	for i := range len(name_buffer) {
		name_buffer[i] = 0
	}

	copy(name_buffer, []byte(parser_name))
	written_bytes, err := conn.Write(name_buffer)

	if err != nil {
		fmt.Println(err)
		return
	}

	if written_bytes != len(name_buffer) {
		fmt.Println("Could not write all name bytes")
		return
	}

	var read_buffer []byte = nil
	var write_buffer []byte = nil

	// fmt.Printf("Header: %v   (%d, %d, %d)\n", header, buffer_size, payload_size, batch_size)

	for {
		// conn.SetDeadline(time.Now().Add(time.Second * 60))

		// Read header
		header := make([]byte, 8)
		byte_offset := 0

		for {
			received_bytes, err := conn.Read(header[byte_offset:])

			if err != nil {
				if err != io.EOF {
					fmt.Println(err)
				}
				return
			}

			byte_offset += received_bytes

			if byte_offset >= len(header) {
				break
			}
		}

		buffer_size := binary.LittleEndian.Uint32(header[0:4])
		payload_size := binary.LittleEndian.Uint16(header[4:6])
		batch_size := binary.LittleEndian.Uint16(header[6:8])

		if read_buffer == nil || len(read_buffer) != int(payload_size)*int(batch_size) {
			read_buffer = make([]byte, int(payload_size)*int(batch_size))
		}

		if write_buffer == nil || len(write_buffer) != int(buffer_size) {
			write_buffer = make([]byte, int(buffer_size))
		}

		byte_offset = 0

		for {
			received_bytes, err := conn.Read(read_buffer[byte_offset:])
			byte_offset += received_bytes

			if err != nil {
				fmt.Println(err)

				return
			}

			if byte_offset >= len(read_buffer) {
				break
			}
		}

		byte_offset = 4

		for i := 0; i < int(batch_size); i++ {
			// fmt.Printf("%d %d  %d\n", (i * int(batch_size)), ((i + 1) * int(payload_size)), len(read_buffer))
			payload := read_buffer[(i * int(payload_size)):((i + 1) * int(payload_size))]
			// fmt.Println(payload, string(payload[:]))

			var message []byte

			switch parser_number {
			case 0:
				message = std_parser(payload)

			case 1:
				message = francoispqt_gojay_parser(payload)

			case 2:
				message = json_iterator_parser(payload)

			case 3:
				message = tidwall_gjson_parser(payload)

			case 4:
				message = buger_jsonparser_parser(payload)

			case 5:
				message = minio_simdjson_parser(payload)

			// case 6:
			// 	message = []byte(ohohler55_ojg_parser(payload))

			case 6:
				message = valyala_fastjson_parser(payload)

			case 7:
				message = sugawarayuuta_sonnet_parser(payload)

			default:
				fmt.Println("Invalid parser number")
				os.Exit(1)
			}

			binary.LittleEndian.PutUint16(write_buffer[byte_offset:], uint16(len(message)))
			byte_offset += 2

			copy(write_buffer[byte_offset:], message)
			byte_offset += len(message)
		}

		binary.LittleEndian.PutUint32(write_buffer[0:4], uint32(byte_offset-4))
		send_offset := 0

		// Send buffer
		for {
			sent_bytes, err := conn.Write(write_buffer[send_offset:byte_offset])
			send_offset += sent_bytes

			if err != nil {
				fmt.Println(err)
				return
			}

			if sent_bytes >= byte_offset {
				break
			}
		}
		// copy(write_buffer[byte_offset:], message)
	}
}

// switch val.(type) {
// case int32:
//
// 	binary.LittleEndian.PutUint16(write_buffer[byte_offset:], uint16(4))
// 	byte_offset += 2
//
// 	casted_val, ok := val.(int32)
//
// 	if ok {
// 		binary.LittleEndian.PutUint32(write_buffer[byte_offset:], uint32(casted_val))
// 	}
//
// 	byte_offset += 4
//
// case int64:
// 	binary.LittleEndian.PutUint16(write_buffer[byte_offset:], uint16(8))
// 	byte_offset += 2
//
// 	casted_val, ok := val.(int32)
//
// 	if ok {
// 		binary.LittleEndian.PutUint32(write_buffer[byte_offset:], uint32(casted_val))
// 	}
//
// 	byte_offset += 8
// }
// fmt.Fprintf(w, "%v", val)
