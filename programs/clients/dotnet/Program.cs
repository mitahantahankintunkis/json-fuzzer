using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using Newtonsoft.Json;

delegate string JsonParserDelegate(ReadOnlySpan<byte> data, string key);

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

        // var parsers = new (string name, Func<ReadOnlySpan<byte>, string, string> fn)[] {
        var parsers = new (string name, JsonParserDelegate fn)[] {
            ("dotnet_std", ParseStd),
            ("dotnet_newtonsoft", ParseNewtonsoft),
            // ("dotnet_std_unoptimized", ParseStd2),
            // ("dotnet_newtonsoft_unoptimized", ParseNewtonsoft2),
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
            var infoBuf = new byte[65];
            var nameBytes = Encoding.UTF8.GetBytes(name);
            Array.Copy(nameBytes, infoBuf, Math.Min(nameBytes.Length, 64));
            networkStream.Write(infoBuf, 0, infoBuf.Length);
            networkStream.Flush();

            var headerBuf = new byte[9];

            while (true)
            {
                // var watch0 = System.Diagnostics.Stopwatch.StartNew();
                // var watch1 = System.Diagnostics.Stopwatch.StartNew();
                //
                // double t_recv = 0;
                // double t_parse = 0;
                // double t_send = 0;
                // double t_other = 0;
                //
                // watch0.Restart();
                try
                {
                    networkStream.ReadExactly(headerBuf, 0, 9);
                }
                catch (EndOfStreamException)
                {
                    return 0;
                }

                // t_recv += watch0.Elapsed.TotalNanoseconds;

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

                // watch0.Restart();
                networkStream.ReadExactly(readBuffer, 0, (int)key_len);

                string key = Encoding.UTF8.GetString(readBuffer, 0, (int)key_len);

                networkStream.ReadExactly(readBuffer, 0, (int)input_buffer_size);
                // t_recv += watch0.Elapsed.TotalNanoseconds;

                int writeOffset = 4;
                int readOffset = 0;
                var watch = System.Diagnostics.Stopwatch.StartNew();

                while (readOffset < input_buffer_size)
                {
                    // watch0.Restart();
                    int json_size = BitConverter.ToUInt16(readBuffer, readOffset);
                    readOffset += 2;

                    var data = readBuffer.AsSpan(readOffset, json_size);
                    // string data = Encoding.UTF8.GetString(readBuffer, readOffset, json_size);
                    readOffset += json_size;
                    // t_other += watch0.Elapsed.TotalNanoseconds;

                    watch.Restart();
                    string message = parserFn(data, key);
                    UInt32 elapsed = (UInt32)(watch.Elapsed.TotalNanoseconds / 10);
                    // Console.Error.Write("1: ", watch.Elapsed.TotalNanoseconds);
                    // t_parse += watch.Elapsed.TotalNanoseconds;
                    //
                    // watch0.Restart();

                    var ns_bytes = BitConverter.GetBytes(elapsed);
                    // if (BitConverter.IsLittleEndian) Array.Reverse(ns_bytes);
                    Array.Copy(ns_bytes, 0, writeBuffer, writeOffset, 4);
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

                    // t_other += watch0.Elapsed.TotalNanoseconds;
                }

                // watch0.Restart();

                // Prepend 4-byte little-endian size
                int size = writeOffset - 4;
                writeBuffer[0] = (byte)(size & 0xFF);
                writeBuffer[1] = (byte)((size >> 8) & 0xFF);
                writeBuffer[2] = (byte)((size >> 16) & 0xFF);
                writeBuffer[3] = (byte)((size >> 24) & 0xFF);

                networkStream.Write(writeBuffer, 0, writeOffset);
                networkStream.Flush();

                // t_send += watch0.Elapsed.TotalNanoseconds;
                // Console.Error.WriteLine("{0} size    {1}", name, input_buffer_size);
                // Console.Error.WriteLine("{0} t_recv  {1}", name, t_recv);
                // Console.Error.WriteLine("{0} t_parse {1}", name, t_parse);
                // Console.Error.WriteLine("{0} t_send  {1}", name, t_send);
                // Console.Error.WriteLine("{0} t_other {1}", name, t_other);
                // Console.Error.WriteLine("");
            }
        }
    }

    static string ParseStd(ReadOnlySpan<byte> data, string key)
    {
        // True for object, false for array
        var stack = new Stack<bool>();

        try
        {
            var reader = new Utf8JsonReader(data);
            var ret = PARSE_ERROR;

            while (reader.Read())
            {
                if (reader.TokenType == JsonTokenType.PropertyName && reader.ValueTextEquals(key))
                {
                    if (stack.Count != 1 || !stack.Peek())
                    {
                        continue;
                    }

                    // Move to the value
                    reader.Read();

                    if (reader.TokenType == JsonTokenType.StartObject)
                    {
                        stack.Push(true);
                    }
                    else if (reader.TokenType == JsonTokenType.StartArray)
                    {
                        stack.Push(false);
                    }
                    else if (reader.TokenType == JsonTokenType.EndObject)
                    {
                        return PARSE_ERROR;
                    }
                    else if (reader.TokenType == JsonTokenType.EndArray)
                    {
                        return PARSE_ERROR;
                    }

                    ret = reader.TokenType switch
                    {
                        JsonTokenType.String => $"\"{reader.GetString()}\"",
                        JsonTokenType.Number => reader.GetDouble().ToString(),
                        JsonTokenType.True => "true",
                        JsonTokenType.False => "false",
                        JsonTokenType.Null => "null",
                        _ => PARSE_ERROR,
                    };
                }
                else if (reader.TokenType == JsonTokenType.StartObject)
                {
                    stack.Push(true);
                }
                else if (reader.TokenType == JsonTokenType.EndObject)
                {
                    if (stack.Count == 0 || !stack.Peek())
                    {
                        return PARSE_ERROR;
                    }

                    stack.Pop();
                }
                else if (reader.TokenType == JsonTokenType.StartArray)
                {
                    stack.Push(false);
                }
                else if (reader.TokenType == JsonTokenType.EndArray)
                {
                    if (stack.Count == 0 || stack.Peek())
                    {
                        return PARSE_ERROR;
                    }

                    stack.Pop();
                }

            }

            if (stack.Count != 0) ret = PARSE_ERROR;

            return ret;
        }
        catch { return PARSE_ERROR; }
    }

    static string ParseNewtonsoft(ReadOnlySpan<byte> data, string key)
    {
        try
        {
            string str = Encoding.UTF8.GetString(data);
            string key_lower = key.ToLower();
            var sr = new StringReader(str);
            var reader = new JsonTextReader(sr);
            var ret = PARSE_ERROR;

            // True for object, false for array
            var stack = new Stack<bool>();

            while (reader.Read())
            {
                if (reader.Value != null)
                {
                    // Newtonsoft is case insensitive by default. Replicating the behavior here
                    string value = reader.Value.ToString() ?? "";
                    if (reader.TokenType == JsonToken.PropertyName && value.ToString().ToLower().Equals(key_lower))
                    {
                        if (stack.Count != 1 || !stack.Peek())
                        {
                            continue;
                        }

                        // Move to the value
                        reader.Read();

                        if (reader.TokenType == JsonToken.StartObject)
                        {
                            stack.Push(true);
                        }

                        if (reader.TokenType == JsonToken.StartArray)
                        {
                            stack.Push(false);
                        }
                        else if (reader.TokenType == JsonToken.EndObject)
                        {
                            return PARSE_ERROR;
                        }
                        else if (reader.TokenType == JsonToken.EndArray)
                        {
                            return PARSE_ERROR;
                        }

                        ret = reader.TokenType switch
                        {
                            JsonToken.String => $"\"{reader.Value.ToString()}\"",
                            JsonToken.Float => reader.Value.ToString(),
                            JsonToken.Integer => reader.Value.ToString(),
                            JsonToken.Boolean => reader.Value.Equals(true) ? "true" : "false",
                            JsonToken.Null => "null",
                            // JsonToken.Integer => reader.ReadAsDouble().ToString(),
                            // JsonToken.Boolean => reader.ReadAsBoolean() ?? false ? "true" : "false",
                            // JsonToken.Null => "null",
                            _ => PARSE_ERROR,
                        };
                    }

                }
                else
                {
                    if (reader.TokenType == JsonToken.StartObject)
                    {
                        stack.Push(true);
                    }
                    else if (reader.TokenType == JsonToken.EndObject)
                    {
                        if (stack.Count == 0 || !stack.Peek())
                        {
                            return PARSE_ERROR;
                        }

                        stack.Pop();
                    }
                    else if (reader.TokenType == JsonToken.StartArray)
                    {
                        stack.Push(false);
                    }
                    else if (reader.TokenType == JsonToken.EndArray)
                    {
                        if (stack.Count == 0 || stack.Peek())
                        {
                            return PARSE_ERROR;
                        }

                        stack.Pop();
                    }
                }
            }

            if (stack.Count != 0) ret = PARSE_ERROR;

            return ret ?? PARSE_ERROR;
        }
        catch { return PARSE_ERROR; }
    }

    static string ParseStd2(ReadOnlySpan<byte> data, string key)
    {
        try
        {
            string str = Encoding.UTF8.GetString(data);
            using (JsonDocument doc = JsonDocument.Parse(str))
            {
                if (!doc.RootElement.TryGetProperty(key, out JsonElement el)) return KEY_NOT_FOUND;

                switch (el.ValueKind)
                {
                    case JsonValueKind.String:
                        // Console.Out.Write(el.GetString());
                        var ret = el.GetString();

                        if (ret == null)
                        {
                            return PARSE_ERROR;
                        }
                        else
                        {
                            return "\"" + ret + "\"";
                        }

                    case JsonValueKind.Number:
                        return el.GetDouble().ToString();
                    // case JsonValueKind.True:
                    // case JsonValueKind.False:
                    //     return el.GetRawText();
                    // case JsonValueKind.Null:
                    //     return "null";
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

    static string ParseNewtonsoft2(ReadOnlySpan<byte> data, string key)
    {
        string str = Encoding.UTF8.GetString(data);

        try
        {
            QueryDouble? parsed = JsonConvert.DeserializeObject<QueryDouble>(str);

            if (parsed == null)
            {
                return PARSE_ERROR;
            }

            if (parsed.q == null)
            {
                return KEY_NOT_FOUND;
            }

            return parsed.q.ToString() ?? PARSE_ERROR;
        }
        catch (Exception) { }

        try
        {
            QueryString? parsed_str = JsonConvert.DeserializeObject<QueryString>(str);

            if (parsed_str == null)
            {
                return PARSE_ERROR;
            }

            if (parsed_str.q == null)
            {
                return KEY_NOT_FOUND;
            }

            return "\"" + parsed_str.q + "\"";
        }
        catch (Exception)
        {
            return PARSE_ERROR;
        }
    }
}

class QueryDouble
{
    public double? q;
}

class QueryString
{
    public string? q;
}
