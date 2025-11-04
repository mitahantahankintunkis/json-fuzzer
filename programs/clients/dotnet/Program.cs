using System.Net.Sockets;
using System.Text;
using System.Text.Json;

class Program
{
    const string KEY_NOT_FOUND = "KEY_NOT_FOUND";
    const string PARSE_ERROR = "PARSE_ERROR";

    static int Main(string[] args)
    {
        int parserNumber = 0;
        if (args.Length == 1)
        {
            if (!int.TryParse(args[0], out parserNumber)) parserNumber = 0;
        }

        var parsers = new (string name, Func<string, string, string> fn)[] {
            ("dotnet_std", ParseStd),
        };

        if (parserNumber >= parsers.Length) return 1;

        var (name, parserFn) = parsers[parserNumber];

        // Connect repeatedly to Unix domain socket /tmp/fuzzer.sock
        var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        var endpoint = new UnixDomainSocketEndPoint("/tmp/fuzzer.sock");
        while (true)
        {
            try
            {
                socket.Connect(endpoint);
                if (socket.Connected) break;
            }
            catch
            {
                socket.Dispose();
                Thread.Sleep(1000);
            }
        }

        var readBuffer = new byte[1 << 16];
        var writeBuffer = new byte[1 << 16];

        using (socket)
        using (var networkStream = new NetworkStream(socket, ownsSocket: false))
        {
            // Send name padded to 64 bytes with NULs
            var nameBytes = new byte[64];
            var name_bytes = Encoding.UTF8.GetBytes(name);
            Array.Copy(name_bytes, nameBytes, Math.Min(name_bytes.Length, 64));
            networkStream.Write(nameBytes, 0, nameBytes.Length);
            networkStream.Flush();

            var headerBuf = new byte[9];

            while (true)
            {
                try
                {
                    networkStream.ReadExactly(headerBuf, 0, 9);
                }
                catch (EndOfStreamException)
                {
                    return 0;
                }

                uint input_buffer_size = BitConverter.ToUInt32(headerBuf, 0);
                uint key_len = BitConverter.ToUInt32(headerBuf, 5);
                // uint bufferSize = BitConverter.ToUInt32(headerBuf, 0);
                // uint payloadSize = BitConverter.ToUInt16(headerBuf, 4);
                // uint batchSize = BitConverter.ToUInt32(headerBuf, 6);

                uint max_len = Math.Max(key_len, input_buffer_size);
                // uint totalPayload = batchSize * payloadSize;

                if (readBuffer.Length < max_len)
                {
                    readBuffer = new byte[max_len];
                }

                if (writeBuffer.Length < input_buffer_size << 2)
                {
                    writeBuffer = new byte[input_buffer_size << 2];
                }

                networkStream.ReadExactly(readBuffer, 0, (int)key_len);
                string key = Encoding.UTF8.GetString(readBuffer, 0, (int)key_len);

                networkStream.ReadExactly(readBuffer, 0, (int)input_buffer_size);
                int writeOffset = 4;
                int readOffset = 0;
                var watch = System.Diagnostics.Stopwatch.StartNew();

                while (readOffset < input_buffer_size)
                {
                    int json_size = BitConverter.ToUInt16(readBuffer, readOffset);
                    readOffset += 2;

                    string data = Encoding.UTF8.GetString(readBuffer, readOffset, json_size);
                    readOffset += json_size;

                    watch.Reset();
                    string message = parserFn(data, key);
                    UInt32 elapsed = (UInt32)watch.Elapsed.TotalMicroseconds;

                    var micros_bytes = BitConverter.GetBytes(elapsed);
                    if (BitConverter.IsLittleEndian) Array.Reverse(micros_bytes);
                    Array.Copy(micros_bytes, 0, writeBuffer, writeOffset, 4);
                    writeOffset += 4;

                    var msgBytes = Encoding.UTF8.GetBytes(message);
                    if (msgBytes.Length > ushort.MaxValue)
                    {
                        Console.Out.Write("Dotnet client: Parsed message too big");
                        return 1;
                    }

                    ushort len = (ushort)msgBytes.Length;
                    writeBuffer[writeOffset + 0] = (byte)(len & 0xFF);
                    writeBuffer[writeOffset + 1] = (byte)((len >> 8) & 0xFF);
                    writeOffset += 2;

                    Array.Copy(msgBytes, 0, writeBuffer, writeOffset, msgBytes.Length);
                    writeOffset += msgBytes.Length;
                }

                // Prepend 4-byte little-endian size
                int size = writeOffset - 4;
                writeBuffer[0] = (byte)(size & 0xFF);
                writeBuffer[1] = (byte)((size >> 8) & 0xFF);
                writeBuffer[2] = (byte)((size >> 16) & 0xFF);
                writeBuffer[3] = (byte)((size >> 24) & 0xFF);

                networkStream.Write(writeBuffer, 0, writeOffset);
                networkStream.Flush();
            }
        }
    }

    static string ParseStd(string data, string key)
    {
        try
        {
            using (JsonDocument doc = JsonDocument.Parse(data))
            {
                if (!doc.RootElement.TryGetProperty(key, out JsonElement el)) return KEY_NOT_FOUND;

                switch (el.ValueKind)
                {
                    case JsonValueKind.String:
                        // Console.Out.Write(el.GetString());
                        return el.GetString() ?? PARSE_ERROR;
                    case JsonValueKind.Number:
                        return el.GetRawText();
                    case JsonValueKind.True:
                    case JsonValueKind.False:
                        return el.GetRawText();
                    case JsonValueKind.Null:
                        return "null";
                    default:
                        return el.GetRawText();
                }
            }
        }
        catch (Exception)
        {
            // Console.Out.WriteLine(e);
            return PARSE_ERROR;
        }
    }
}

// static string ParseNewtonsoft(string data, string key)
// {
//     // try
//     // {
//     //     using (JsonDocument doc = JsonDocument.Parse(data))
//     //     {
//     //         if (!doc.RootElement.TryGetProperty(key, out JsonElement el)) return KEY_NOT_FOUND;
//     //
//     //         switch (el.ValueKind)
//     //         {
//     //             case JsonValueKind.String:
//     //                 return el.GetString() ?? PARSE_ERROR;
//     //             case JsonValueKind.Number:
//     //                 return el.GetRawText();
//     //             case JsonValueKind.True:
//     //             case JsonValueKind.False:
//     //                 return el.GetRawText();
//     //             case JsonValueKind.Null:
//     //                 return "null";
//     //             default:
//     //                 return el.GetRawText();
//     //         }
//     //     }
//     // }
//     // catch (JsonException)
//     // {
//     return PARSE_ERROR;
//     // }
// }
