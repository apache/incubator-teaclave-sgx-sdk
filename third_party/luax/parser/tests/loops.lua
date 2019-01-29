for i = 1, 10 do
    local a = i
end

for i = 1, 10, 1 do
    local a = i
end

local a = {1, 2}
for i, v in ipairs(a) do
    local t = v
end

local i = 0
while i < 10 do
    i = i + 1
end

repeat
    i = i - 1
until i == 0