using System.Net.Sockets;
using System.Text;
using System.Text.Json;

class Program
{
    const string KEY_NOT_FOUND = "KEY_NOT_FOUND";
    const string PARSE_ERROR = "PARSE_ERROR";

    static int Main(string[] args)
    {
        if (!BitConverter.IsLittleEndian)
        {
            Console.Out.Write("Dotnet client: Only works on litte endian machines");
            return 1;
        }

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

            var headerBuf = new byte[10];

            while (true)
            {
                try
                {
                    networkStream.ReadExactly(headerBuf, 0, 10);
                }
                catch (EndOfStreamException)
                {
                    return 0;
                }

                uint bufferSize = BitConverter.ToUInt32(headerBuf, 0);
                uint payloadSize = BitConverter.ToUInt16(headerBuf, 4);
                uint batchSize = BitConverter.ToUInt32(headerBuf, 6);

                uint totalPayload = batchSize * payloadSize;

                if (readBuffer.Length < totalPayload)
                {
                    readBuffer = new byte[totalPayload];
                }

                if (writeBuffer.Length < bufferSize * 4)
                {
                    writeBuffer = new byte[bufferSize * 4];
                }

                // var payload = new byte[payloadSize];

                networkStream.ReadExactly(readBuffer, 0, (int)totalPayload);
                int byteOffset = 4;

                for (long batch = 0; batch < batchSize; ++batch)
                {
                    int start = (int)(batch * payloadSize);
                    // Array.Copy(readBuffer, start, payload, 0, payloadSize);
                    string data = Encoding.UTF8.GetString(readBuffer, start, (int)payloadSize);

                    string message = parserFn(data, "q");

                    // Console.Out.WriteLine(data, " ", message);

                    var msgBytes = Encoding.UTF8.GetBytes(message);
                    if (msgBytes.Length > ushort.MaxValue)
                    {
                        Console.Out.Write("Dotnet client: Parsed message too big");
                        return 1;
                    }

                    ushort len = (ushort)msgBytes.Length;
                    writeBuffer[byteOffset + 0] = (byte)(len & 0xFF);
                    writeBuffer[byteOffset + 1] = (byte)((len >> 8) & 0xFF);
                    byteOffset += 2;

                    Array.Copy(msgBytes, 0, writeBuffer, byteOffset, msgBytes.Length);
                    byteOffset += msgBytes.Length;
                }

                // Prepend 4-byte little-endian size
                int size = byteOffset - 4;
                writeBuffer[0] = (byte)(size & 0xFF);
                writeBuffer[1] = (byte)((size >> 8) & 0xFF);
                writeBuffer[2] = (byte)((size >> 16) & 0xFF);
                writeBuffer[3] = (byte)((size >> 24) & 0xFF);

                networkStream.Write(writeBuffer, 0, byteOffset);
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

    static string ParseNewtonsoft(string data, string key)
    {
        // try
        // {
        //     using (JsonDocument doc = JsonDocument.Parse(data))
        //     {
        //         if (!doc.RootElement.TryGetProperty(key, out JsonElement el)) return KEY_NOT_FOUND;
        //
        //         switch (el.ValueKind)
        //         {
        //             case JsonValueKind.String:
        //                 return el.GetString() ?? PARSE_ERROR;
        //             case JsonValueKind.Number:
        //                 return el.GetRawText();
        //             case JsonValueKind.True:
        //             case JsonValueKind.False:
        //                 return el.GetRawText();
        //             case JsonValueKind.Null:
        //                 return "null";
        //             default:
        //                 return el.GetRawText();
        //         }
        //     }
        // }
        // catch (JsonException)
        // {
        return PARSE_ERROR;
        // }
    }
}
