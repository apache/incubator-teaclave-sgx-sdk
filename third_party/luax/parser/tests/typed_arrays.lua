local arr = typedarray("u8", 1000000)
local i = 0

while i < 1000000 do
    arr[i] = i % 256
    i = i + 1
end
