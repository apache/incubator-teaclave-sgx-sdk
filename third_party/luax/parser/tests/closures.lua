function generate(a)
    return function(b)
        return a + b
    end
end

local f = generate(5)
assert(f(3) == 8)
