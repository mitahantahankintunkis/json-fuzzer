require 'socket'
require 'json'

PARSE_ERROR = 'PARSE_ERROR'.b
KEY_NOT_FOUND = 'KEY_NOT_FOUND'.b

def std_parser(encoded, key)
  data = JSON.parse(encoded)

  return KEY_NOT_FOUND if !data.key?(key) || data[key].nil?

  JSON.generate(data[key])
  # data[key].to_s.b
rescue StandardError
  PARSE_ERROR
end

def recv_all(sock, buf, n)
  offset = 0
  while offset < n
    chunk = sock.read(n - offset)
    exit(0) if chunk.nil?
    buf[offset, chunk.bytesize] = chunk
    offset += chunk.bytesize
  end
end

parser_number = 0

if ARGV.length > 0
  begin
    parser_number = Integer(ARGV[0])
  rescue ArgumentError
    exit(1)
  end
end

parser_name = case parser_number
              when 0
                'ruby_std'
              else
                exit(1)
              end

# Connect to server (retry loop)
sock = nil
loop do
  # sock = TCPSocket.new("localhost", 5000)
  sock = UNIXSocket.new('/tmp/fuzzer.sock')
  break
rescue StandardError
  sleep(0.1)
end

at_exit { sock.close if sock }

# Send parser name (padded to 64 bytes)
info_buf = "\x00" * 65
info_buf[0, parser_name.bytesize] = parser_name
written = sock.write(info_buf)

if written != info_buf.bytesize
  puts 'Ruby: Could not write all name bytes'
  exit(1)
end

read_buffer = nil
write_buffer = nil

loop do
  # Read 8-byte header
  header = sock.read(9)
  break if header.nil?

  input_buffer_size, _, key_len = header.unpack('VBV')

  max_len = input_buffer_size
  max_len = key_len if max_len < key_len

  read_buffer = "\x00" * max_len if read_buffer.nil? || read_buffer.bytesize < max_len

  if write_buffer.nil? || write_buffer.bytesize < input_buffer_size << 2
    write_buffer = "\x00" * (input_buffer_size << 2)
  end

  recv_all(sock, read_buffer, key_len)
  key = read_buffer[0, key_len]

  recv_all(sock, read_buffer, input_buffer_size)

  read_offset = 0
  write_offset = 4

  while read_offset < input_buffer_size
    json_size = read_buffer[read_offset, 2].unpack1('v')
    read_offset += 2

    json = read_buffer[read_offset, json_size]
    read_offset += json_size

    start_time = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    message = case parser_number
              when 0
                std_parser(json, key)
              else
                exit(1)
              end
    end_time = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    ns = (end_time - start_time) * 100_000_000

    write_buffer[write_offset, 4] = [ns].pack('V')
    write_offset += 4

    write_buffer[write_offset, 2] = [message.bytesize].pack('v')
    write_offset += 2

    write_buffer[write_offset, message.bytesize] = message
    write_offset += message.bytesize
  end

  # Write total size at start (uint32 little-endian)
  write_buffer[0, 4] = [write_offset - 4].pack('V')

  # Send buffer
  sent_offset = 0
  while sent_offset < write_offset
    sent = sock.write(write_buffer[sent_offset...write_offset])
    if sent <= 0
      puts 'Ruby: Failed to send'
      exit(1)
    end
    sent_offset += sent
  end
end
