local i = 0
while i < 10 do
    i = i + 1
    if i == 5 then
        break
    end
end

assert(i == 5)

repeat
    i = i + 1
    if i == 10 then
        break
    end
until i == 20

assert(i == 10)
