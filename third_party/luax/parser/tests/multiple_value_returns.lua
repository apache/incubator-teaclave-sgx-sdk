function rotate(a, b)
    return b, a
end

local left, right = rotate(1, 2)
assert(left == 2)
assert(right == 1)