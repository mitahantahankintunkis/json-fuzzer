const net = require('net');
const JSON5 = require('json5');

const KEY_NOT_FOUND = 'KEY_NOT_FOUND';
const PARSE_ERROR = 'PARSE_ERROR';
const SOCKET_PATH = '/tmp/fuzzer.sock';


function parseJson(data, key) {
	try {
		// Remove null bytes and parse
		const cleanData = data.toString('utf8').replace(/\0+$/, '');
		const parsed = JSON.parse(cleanData);

		if (Object.prototype.hasOwnProperty.call(parsed, key)) {
			return JSON.stringify(parsed[key]);
		} else {
			return KEY_NOT_FOUND;
		}
	} catch (err) {
		return PARSE_ERROR;
	}
}


function parseJson5(data, key) {
	try {
		// Remove null bytes and parse
		const cleanData = data.toString('utf8').replace(/\0+$/, '');
		const parsed = JSON5.parse(cleanData);

		if (Object.prototype.hasOwnProperty.call(parsed, key)) {
			// Use JSON.stringify to clean up the results
			return JSON.stringify(parsed[key]);
			// return JSON5.stringify(parsed[key], {
			// 	quote: '"',
			// });
		} else {
			return KEY_NOT_FOUND;
		}
	} catch (err) {
		return PARSE_ERROR;
	}
}


let parserNumber = 0;
if (process.argv.length === 3) {
	parserNumber = parseInt(process.argv[2], 10);
}

let name = Buffer.alloc(65, 0);
let parserFn;

switch (parserNumber) {
	case 0:
		name.write('js_v8', 0, 'utf8');
		parserFn = parseJson;
		break;
	case 1:
		name.write('js_json5', 0, 'utf8');
		parserFn = parseJson5;
		break;
	default:
		process.exit(1);
}

const client = new net.Socket();

const connect = () => {
	client.connect(SOCKET_PATH);
};

client.on('connect', () => {
	client.write(name);
});

client.on('error', (err) => {
	if (err.code === 'ENOENT' || err.code === 'ECONNREFUSED') {
		setTimeout(connect, 100);
	}
});

let internalBuffer = Buffer.alloc(0);

client.on('data', (chunk) => {
	// Concatenate new data to our processing buffer
	internalBuffer = Buffer.concat([internalBuffer, chunk]);

	if (internalBuffer.length < 9) {
		return;
	}

	const inputBufferSize = internalBuffer.readUInt32LE(0);
	const keyLen = internalBuffer.readUInt32LE(5);

	const totalExpectedLen = 9 + keyLen + inputBufferSize;

	// Read all
	if (internalBuffer.length < totalExpectedLen) {
		return;
	}

	const key = internalBuffer.subarray(9, 9 + keyLen).toString('utf8');
	const dataRegion = internalBuffer.subarray(9 + keyLen, 9 + keyLen + totalExpectedLen);

	// Prepare Write Buffer (initial size estimate)
	let writeBuffer = Buffer.alloc(Math.max(inputBufferSize * 4, 100000) + 4);
	let writeOffset = 4;
	let readOffset = 0;

	while (readOffset < inputBufferSize) {
		const jsonSize = dataRegion.readUInt16LE(readOffset);
		readOffset += 2;

		const jsonData = dataRegion.subarray(readOffset, readOffset + jsonSize);
		readOffset += jsonSize;

		const start = process.hrtime.bigint();
		const message = parserFn(jsonData, key);
		const end = process.hrtime.bigint();

		const ns = Math.floor(Number(end - start) / 10);

		// Write timing
		writeBuffer.writeUInt32LE(ns, writeOffset);
		writeOffset += 4;

		let msgBytes = Buffer.from(message, 'utf8');
		if (msgBytes.length > 0xFFFF) {
			msgBytes = msgBytes.subarray(0, 0xFFFF);
		}

		// Write message size and content
		writeBuffer.writeUInt16LE(msgBytes.length, writeOffset);
		writeOffset += 2;

		msgBytes.copy(writeBuffer, writeOffset);
		writeOffset += msgBytes.length;
	}

	// Write the total length header
	writeBuffer.writeUInt32LE(writeOffset - 4, 0);

	if (!client.write(writeBuffer.subarray(0, writeOffset))) {
		console.error("JS: Could not send entire buffer");
	}

	// internalBuffer = internalBuffer.slice(totalExpectedLen);
	internalBuffer = Buffer.alloc(0);
});

connect();
