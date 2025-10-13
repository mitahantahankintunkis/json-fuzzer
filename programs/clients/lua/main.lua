#!/bin/env lua

local socket = require("socket")
socket.unix = require("socket.unix")
local cjson = require("cjson.safe")
--local simdjson = require("resty.simdjson")
-- local bit32 = require("bit32")
local string = string
local table = table

local KEY_NOT_FOUND = "KEY_NOT_FOUND"
local PARSE_ERROR = "PARSE_ERROR"

-- local simdjson_parser = simdjson.new()
--
-- local function parse_simdjson(data, key)
-- 	local parsed = simdjson_parser:decode(data)
--
-- 	if not parsed then
-- 		return PARSE_ERROR
-- 	end
-- 	if parsed[key] ~= nil then
-- 		return tostring(parsed[key])
-- 	else
-- 		return KEY_NOT_FOUND
-- 	end
-- end

local function parse_cjson(data, key)
	local str = data:gsub("%z+$", "")
	local parsed, _ = cjson.decode(str)
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

	local parsers = {
		{ "lua_cjson", parse_cjson },
		--{ "lua_simdjson", parse_simdjson },
	}

	if parser_number >= #parsers then
		os.exit(1)
	end

	local name, parser_fn = unpack(parsers[1])

	-- Connect to server
	local s
	repeat
		s = socket.unix()
		local ok, _ = s:connect("/tmp/fuzzer.sock")
		if not ok then
			socket.sleep(0.1)
		end
	until ok

	-- Send name padded to 64 bytes
	local name_buffer = name .. string.rep("\0", 64 - #name)
	s:send(name_buffer)

	while true do
		-- Read header (8 bytes)
		local header, _ = s:receive(10)
		if not header then
			return
		end

		-- Unpack: <IHH (little-endian: u32, u16, u32)
		local h1, h2, h3, h4, h5, h6 = header:byte(5, 10)
		local payload_size = h1 + h2 * 256
		local batch_size = h3 + h4 * 256 + h5 * 65536 + h6 * 16777216

		local total_payload = batch_size * payload_size

		-- Read JSON payloads
		local payload, _, _ = s:receive(total_payload)
		if not payload then
			return
		end

		local byte_offset = 5
		local parts = {}
		table.insert(parts, "\0\0\0\0") -- reserve 4 bytes

		for batch = 0, batch_size - 1 do
			local start = batch * payload_size + 1
			local stop = start + payload_size - 1
			local data = payload:sub(start, stop)

			local message = parser_fn(data, "q")

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
