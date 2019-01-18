function map(v)
    if v == 1 then
        return 10
    elseif v == 2 then
        return 20
    elseif v >= 100 then
        return -1
    else
        return 0
    end
end

function map2(v)
    if v == 1 then
        return 11
    elseif v == 2 then
        return 21
    elseif v >= 100 then
        return -10
    end

    return 0
end

local v = 0

while v < 100 do
    assert(map(5) == 0) -- 0
    assert(map(100) == -1) -- -1
    assert(map(101) == -1) -- -1
    assert(map(1) == 10) -- 10
    assert(map(2) == 20) -- 20

    assert(map2(1) == 11) -- 11
    assert(map2(2) == 21) -- 21
    assert(map2(5) == 0) -- 0
    assert(map2(200) == -10) -- -10

    v = v + 1
end
