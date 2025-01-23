package main

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"time"
)

func main() {
	// Connect to the server
	conn, err := net.Dial("tcp", "localhost:5000")

	if err != nil {
		fmt.Println(err)
		return
	}

	defer conn.Close()

	// Read header
	header := make([]byte, 8)
	byte_offset := 0

	for {
		received_bytes, err := conn.Read(header[byte_offset:])

		if err != nil {
			fmt.Println(err)
			return
		}

		byte_offset += received_bytes

		if byte_offset >= len(header) {
			break
		}
	}

	name_buffer := make([]byte, 32)

	for i := range len(name_buffer) {
		name_buffer[i] = 0
	}

	copy(name_buffer, []byte("go-std"))
	written_bytes, err := conn.Write(name_buffer)

	if err != nil {
		fmt.Println(err)
		return
	}

	if written_bytes != len(name_buffer) {
		fmt.Println("Could not write all name bytes")
		return
	}

	buffer_size := binary.LittleEndian.Uint32(header[0:4])
	payload_size := binary.LittleEndian.Uint16(header[4:6])
	batch_size := binary.LittleEndian.Uint16(header[6:8])

	read_buffer := make([]byte, int(payload_size)*int(batch_size))
	write_buffer := make([]byte, buffer_size)

	// fmt.Printf("Header: %v   (%d, %d, %d)\n", header, buffer_size, payload_size, batch_size)

	for {
		byte_offset = 0

		conn.SetDeadline(time.Now().Add(time.Second * 60))

		for {
			received_bytes, err := conn.Read(read_buffer[byte_offset:])
			byte_offset += received_bytes

			if err != nil {
				if err != io.EOF {
					fmt.Println(err)
				}

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
			var decoded map[string]interface{}
			json.Unmarshal(payload, &decoded)

			val, ok := decoded["q"]
			var message []byte

			if ok {
				message = []byte(fmt.Sprint(val))

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

			} else {
				message = []byte("PARSE_ERROR")
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

// use std::io::prelude::*;
// use std::net::TcpStream;
//
// fn main() -> std::io::Result<()> {
//     let mut stream = TcpStream::connect("127.0.0.1:5000")?;
//     stream.set_nodelay(true)?;
//
//     let mut header: [u8; 8] = [0; 8];
//
//     stream
//         .read_exact(&mut header)
//         .expect("Could not read buffer size");
//
//     let buffer_size = u32::from_le_bytes(header[0..4].try_into().unwrap());
//     let payload_size = u16::from_le_bytes(header[4..6].try_into().unwrap());
//     let batch_size = u16::from_le_bytes(header[6..8].try_into().unwrap());
//
//     let mut read_buffer: Box<Vec<u8>> =
//         Box::new(vec![0; usize::from(batch_size) * usize::from(payload_size)]);
//     let mut write_buffer: Box<Vec<u8>> = Box::new(vec![0; buffer_size.try_into().unwrap()]);
//     // read_buffer.resize(buffer_size * batch_size, 0);
//     // write_buffer.resize(128 * batch_size, 0);
//
//     loop {
//         match stream.read_exact(&mut read_buffer) {
//             Ok(()) => {
//                 let size_buffer: &[u8] = &payload_size.to_le_bytes();
//                 let mut byte_offset: usize = 4;
//
//                 // stream.write_all(&size_buffer).expect("Write error");
//                 for batch in 0..usize::from(batch_size) {
//                     let data: &[u8] = &read_buffer[(batch * usize::from(payload_size))
//                         ..((batch + 1) * usize::from(payload_size))];
//
//                     write_buffer[byte_offset..(byte_offset + 2)].copy_from_slice(&size_buffer);
//                     byte_offset += 2;
//
//                     write_buffer[byte_offset..(byte_offset + data.len())].copy_from_slice(&data);
//                     byte_offset += data.len();
//                     // println!("size: {:0X?}   data: {:0X?}", &size_buffer, &data);
//                     // for i in 0..128 {
//                     //     if i < buffer_size {
//                     //         write_buffer[batch * 128 + i] = read_buffer[batch * buffer_size + i];
//                     //     } else {
//                     //         write_buffer[batch * 128 + i] = 0;
//                     //     }
//                     // }
//                 }
//
//                 write_buffer[0..4]
//                     .copy_from_slice(&u32::try_from(byte_offset - 4).unwrap().to_le_bytes());
//
//                 stream
//                     // .write_all(&write_buffer)
//                     .write_all(&write_buffer[0..byte_offset])
//                     .expect("Write error");
//             }
//             Err(_e) => {
//                 println!("Connection closed");
//                 return Ok(());
//             }
//         }
//         // match stream.read_exact(&mut read_buffer) {
//         //     Ok(()) => {
//         //         // stream.write_all(&size_buffer).expect("Write error");
//         //         stream.write_all(&read_buffer).expect("Write error");
//         //     }
//         //     Err(_e) => {
//         //         println!("Connection closed");
//         //         return Ok(());
//         //     }
//         // }
//     }
// }
