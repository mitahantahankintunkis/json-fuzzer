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
	-- local str = data:gsub("%z+$", "")
	-- print(str, key)
	local parsed, _ = cjson.decode(data)
	if not parsed then
		return PARSE_ERROR
	end
	if type(parsed) == "table" and parsed[key] ~= nil then
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
		local header, _ = s:receive(9)
		if not header then
			return
		end

		local h1, h2, h3, h4 = header:byte(1, 4)
		local input_buffer_size = h1 + h2 * 256 + h3 * 65536 + h4 * 16777216
		h1, h2, h3, h4 = header:byte(6, 9)
		local key_len = h1 + h2 * 256 + h3 * 65536 + h4 * 16777216
		local key, _, _ = s:receive(key_len)

		local read_buffer, _, _ = s:receive(input_buffer_size)
		if not read_buffer then
			return
		end

		local read_offset = 1
		local write_offset = 5
		local parts = {}
		table.insert(parts, "\0\0\0\0")

		while read_offset <= input_buffer_size do
			local s1, s2 = read_buffer:byte(read_offset, read_offset + 1)
			local json_size = s1 + s2 * 256
			read_offset = read_offset + 2

			local json = string.sub(read_buffer, read_offset, read_offset + json_size - 1)
			read_offset = read_offset + json_size

			local start_time = os.clock()
			local message = parser_fn(json, key)
			local end_time = os.clock()
			local micros = math.floor((end_time - start_time) * 1000000)

			local micros_bytes = string.char(
				micros % 256,
				math.floor(micros / 256) % 256,
				math.floor(micros / 65536) % 256,
				math.floor(micros / 16777216) % 256
			)
			table.insert(parts, micros_bytes)
			write_offset = write_offset + 4

			local len = #message
			local size_bytes = string.char(len % 256, math.floor(len / 256))
			table.insert(parts, size_bytes)
			table.insert(parts, message)
			write_offset = write_offset + 2 + len
		end

		local size = write_offset - 5
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
