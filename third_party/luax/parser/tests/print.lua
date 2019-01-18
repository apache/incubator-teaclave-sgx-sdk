function f()
    print("f() called")
    return "OK"
end

local a = 1
local b = a * 2
local b = b ^ 4
print(b .. " " .. a .. " " .. f())
