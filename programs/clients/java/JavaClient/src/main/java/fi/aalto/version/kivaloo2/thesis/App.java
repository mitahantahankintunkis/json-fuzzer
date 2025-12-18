package fi.aalto.version.kivaloo2.thesis;

import java.io.EOFException;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;

// import JacksonParser.java;
// import Parser.java;

public class App {
    private static final String HOST = "127.0.0.1";
    private static final int PORT = 5000;

    public static void main(String[] args) {
        int parserNumber = 0;
        if (args.length == 1) {
            try {
                parserNumber = Integer.parseInt(args[0]);
            } catch (NumberFormatException e) {
                parserNumber = 0;
            }
        }

        Parser[] parsers = new Parser[] {
			new JacksonParser(),
        };

        if (parserNumber >= parsers.length) {
            System.exit(1);
        }

		Parser parser = parsers[parserNumber];
        // String name = parsers[parserNumber].name;
        // ParserFunction parserFn = parsers[parserNumber].fn;

        // Connect repeatedly to TCP socket HOST:PORT
        Socket socket = new Socket();
        while (true) {
            try {
                socket.connect(new InetSocketAddress(HOST, PORT));
                if (socket.isConnected()) break;
            } catch (IOException e0) {
				try { socket.close(); } catch (IOException e1) {}
                try { Thread.sleep(100); } catch (InterruptedException e1) {}
            }
        }

		try { socket.setTcpNoDelay(true); } catch (Exception e) {}

        try (Socket s = socket;
             InputStream in = s.getInputStream();
             OutputStream out = s.getOutputStream()) {

            // Send name padded to 64 bytes with NULs
            byte[] infoBuf = new byte[65];
            byte[] name_utf = parser.getName().getBytes(StandardCharsets.UTF_8);
            System.arraycopy(name_utf, 0, infoBuf, 0, Math.min(name_utf.length, 64));
            out.write(infoBuf);
            out.flush();

            byte[] headerBuf = new byte[9];
            byte[] readBuffer = new byte[1 << 16];
            byte[] writeBuffer = new byte[1 << 16];

            while (true) {
                if (!readExactly(in, headerBuf, 0, 9)) {
                    System.exit(0);
				}

                int inputBufferSize = (int)uint32FromLE(headerBuf, 0);
                int keyLen = (int)uint32FromLE(headerBuf, 5);

                int maxLen = Math.max(keyLen, inputBufferSize);
                if (readBuffer.length < maxLen) {
                    readBuffer = new byte[maxLen];
                }
                if (writeBuffer.length < (inputBufferSize << 2)) {
                    writeBuffer = new byte[inputBufferSize << 2];
                }

                // read key
                readExactly(in, readBuffer, 0, keyLen);
                String key = new String(readBuffer, 0, keyLen, StandardCharsets.UTF_8);

                // read input buffer
                readExactly(in, readBuffer, 0, inputBufferSize);

                int readOffset = 0;
                int writeOffset = 4;

                while (readOffset < inputBufferSize) {
                    int jsonSize = uint16FromLE(readBuffer, readOffset);
                    readOffset += 2;

                    // String data = new String(readBuffer, readOffset, jsonSize, StandardCharsets.UTF_8);
					long start = System.nanoTime();
                    String message = parser.parse(readBuffer, readOffset, jsonSize, key);
					long end = System.nanoTime();
					long ns = (end - start) / 10;

					// System.out.println(key + " " + new String(readBuffer, readOffset, jsonSize, StandardCharsets.UTF_8) + " " + message);

                    readOffset += jsonSize;

                    writeBuffer[writeOffset + 0] = (byte)(ns & 0xFF);
                    writeBuffer[writeOffset + 1] = (byte)((ns >> 8) & 0xFF);
                    writeBuffer[writeOffset + 2] = (byte)((ns >> 16) & 0xFF);
                    writeBuffer[writeOffset + 3] = (byte)((ns >> 24) & 0xFF);
                    writeOffset += 4;

                    byte[] msgBytes = message.getBytes(StandardCharsets.UTF_8);
                    if (msgBytes.length > 0xFFFF) {
                        System.out.print("Java client: Parsed message too large");
                        System.exit(1);
                    }

                    int len = msgBytes.length;
                    writeBuffer[writeOffset + 0] = (byte)(len & 0xFF);
                    writeBuffer[writeOffset + 1] = (byte)((len >> 8) & 0xFF);
                    writeOffset += 2;

                    System.arraycopy(msgBytes, 0, writeBuffer, writeOffset, msgBytes.length);
                    writeOffset += msgBytes.length;
                }

                int size = writeOffset - 4;
                writeBuffer[0] = (byte)(size & 0xFF);
                writeBuffer[1] = (byte)((size >> 8) & 0xFF);
                writeBuffer[2] = (byte)((size >> 16) & 0xFF);
                writeBuffer[3] = (byte)((size >> 24) & 0xFF);

                out.write(writeBuffer, 0, writeOffset);
                out.flush();
            }
        } catch (IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static boolean readExactly(InputStream in, byte[] buffer, int offset, int len) {
        int pos = 0;
        while (pos < len) {
			try {
				int got = in.read(buffer, offset + pos, len - pos);
				if (got == -1) return false;
				pos += got;
			} catch (IOException e) {
				return false;
			}
        }

		return true;
    }

    private static long uint32FromLE(byte[] buffer, int offset) {
        long b0 = (long)Byte.toUnsignedInt(buffer[offset + 0]);
        long b1 = (long)Byte.toUnsignedInt(buffer[offset + 1]);
        long b2 = (long)Byte.toUnsignedInt(buffer[offset + 2]);
        long b3 = (long)Byte.toUnsignedInt(buffer[offset + 3]);
        return (b0) | (b1 << 8) | (b2 << 16) | (b3 << 24);
    }

    private static int uint16FromLE(byte[] buffer, int offset) {
        int b0 = Byte.toUnsignedInt(buffer[offset + 0]);
        int b1 = Byte.toUnsignedInt(buffer[offset + 1]);
        return b0 | (b1 << 8);
    }
}
