require 'socket'
require 'json'

PARSE_ERROR = "PARSE_ERROR".b
KEY_NOT_FOUND = "KEY_NOT_FOUND".b

# JSON parser equivalent
def std_parser(encoded)
    begin
        data = JSON.parse(encoded, symbolize_names: true)
    rescue
        return PARSE_ERROR
    end

    begin
        if !data.key?(:q) || data[:q].nil?
            return KEY_NOT_FOUND
        end
    rescue
        return PARSE_ERROR
    end

    data[:q].to_s.b
end

# Main
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
                  "ruby_std"
              else
                  exit(1)
              end

# Connect to server (retry loop)
sock = nil
loop do
    begin
        # sock = TCPSocket.new("localhost", 5000)
        sock = UNIXSocket.new("/tmp/fuzzer.sock")
        break
    rescue
        sleep(0.1)
    end
end

at_exit { sock.close if sock }

# Send parser name (padded to 64 bytes)
name_buffer = "\x00" * 64
name_buffer[0, parser_name.bytesize] = parser_name
written = sock.write(name_buffer)

if written != name_buffer.bytesize
    puts "Ruby: Could not write all name bytes"
    exit(1)
end

read_buffer = nil
write_buffer = nil

loop do
    # Read 8-byte header
    header = sock.read(10)
    break if header.nil?

    buffer_size, payload_size, batch_size = header.unpack("VvV") # little-endian: uint32, uint16, uint16

    read_buffer = "\x00" * (payload_size * batch_size) if read_buffer.nil? || read_buffer.bytesize != payload_size * batch_size
    write_buffer = "\x00" * (buffer_size << 2) if write_buffer.nil? || write_buffer.bytesize != (buffer_size << 2)

    # Read payload
    offset = 0
    while offset < read_buffer.bytesize
        chunk = sock.read(read_buffer.bytesize - offset)
        if chunk.nil?
            exit(0)
        end
        read_buffer[offset, chunk.bytesize] = chunk
        offset += chunk.bytesize
    end

    # Process batch
    out_offset = 4
    batch_size.times do |i|
        payload = read_buffer[i * payload_size, payload_size]

        message = case parser_number
                  when 0
                      std_parser(payload)
                  else
                      exit(1)
                  end

        # Write message length (uint16 little-endian) + message
        write_buffer[out_offset, 2] = [message.bytesize].pack("v")
        out_offset += 2
        write_buffer[out_offset, message.bytesize] = message
        out_offset += message.bytesize
    end

    # Write total size at start (uint32 little-endian)
    write_buffer[0, 4] = [out_offset - 4].pack("V")

    # Send buffer
    sent_offset = 0
    while sent_offset < out_offset
        sent = sock.write(write_buffer[sent_offset...out_offset])
        if sent <= 0
            puts "Ruby: Failed to send"
            exit(1)
        end
        sent_offset += sent
    end
end
