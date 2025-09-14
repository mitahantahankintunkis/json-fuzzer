#!/bin/env lua

local socket = require("socket")
local cjson = require("cjson.safe") -- use safe to avoid hard crashes
-- local bit32 = require("bit32")
local string = string
local table = table

local KEY_NOT_FOUND = "KEY_NOT_FOUND"
local PARSE_ERROR = "PARSE_ERROR"

-- Parse JSON and extract key
local function parse_json(data, key)
	local str = data:gsub("%z+$", "") -- strip trailing \0
	local parsed, err = cjson.decode(str)
	if not parsed then
		return PARSE_ERROR
	end
	if parsed[key] ~= nil then
		return tostring(parsed[key])
	else
		return KEY_NOT_FOUND
	end
end

local function main()
	local parser_number = 0
	if #arg == 1 then
		parser_number = tonumber(arg[1]) or 0
	end

	local name
	if parser_number == 0 then
		name = "lua_cjson"
	else
		os.exit(1)
	end

	-- Connect to server
	local s
	repeat
		s = socket.tcp()
		s:setoption("tcp-nodelay", true)
		local ok, err = s:connect("127.0.0.1", 5000)
		if not ok then
			socket.sleep(0.1)
		end
	until ok

	-- Send name padded to 64 bytes
	local name_buffer = name .. string.rep("\0", 64 - #name)
	s:send(name_buffer)

	local read_buffer = ""
	local write_buffer = ""

	while true do
		-- Read header (8 bytes)
		local header, err = s:receive(8)
		if not header then
			return
		end

		-- Unpack: <IHH (little-endian: u32, u16, u16)
		local b1, b2, b3, b4, h1, h2, h3, h4 = header:byte(1, 8)
		local buffer_size = b1 + b2 * 256 + b3 * 65536 + b4 * 16777216
		local payload_size = h1 + h2 * 256
		local batch_size = h3 + h4 * 256

		local total_payload = batch_size * payload_size

		-- Read JSON payloads
		local payload, err2, partial = s:receive(total_payload)
		if not payload then
			return
		end

		local byte_offset = 5 -- Lua strings are 1-based; reserve first 4 bytes
		local parts = {}
		table.insert(parts, "\0\0\0\0") -- reserve 4 bytes

		for batch = 0, batch_size - 1 do
			local start = batch * payload_size + 1
			local stop = start + payload_size - 1
			local data = payload:sub(start, stop)

			local message
			if parser_number == 0 then
				message = parse_json(data, "q")
			end

			-- message length (u16 LE)
			local len = #message
			local size_bytes = string.char(len % 256, math.floor(len / 256))
			table.insert(parts, size_bytes)
			table.insert(parts, message)
			byte_offset = byte_offset + 2 + len
		end

		-- Write payload size into first 4 bytes
		local size = byte_offset - 5
		local size_bytes = string.char(
			size % 256,
			math.floor(size / 256) % 256,
			math.floor(size / 65536) % 256,
			math.floor(size / 16777216) % 256
		)
		parts[1] = size_bytes

		local result = table.concat(parts)
		s:send(result)
	end
end

main()
