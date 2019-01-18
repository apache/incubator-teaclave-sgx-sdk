local a = 1
local b = 2

function f()
    local a = 0
    a = 3
    b = 5
end

f()
assert(a == 1)
assert(b == 5)
do
    local a = 2
    assert(a == 2)
end
assert(a == 1)

if a == 1 then
    local a = 2
    assert(a == 2)
else
    local a = 3
    assert(a == 3)
end

assert(a == 1)
