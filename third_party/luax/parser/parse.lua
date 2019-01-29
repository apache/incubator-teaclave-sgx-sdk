
local parser = require "lua-parser.parser"
local pp = require "lua-parser.pp"
local cjson = require "cjson"

local code = io.read("*all")

local ast, error_msg = parser.parse(code, "input.lua")
if not ast then
    print(error_msg)
    os.exit(1)
end

print(cjson.encode(ast))
