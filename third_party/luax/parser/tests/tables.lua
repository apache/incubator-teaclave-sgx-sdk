local t1 = {1, 2}
t1["1"] = 3

assert(t1["1"] == 3)
assert(t1[1] == 1)
assert(t1[2] == 2)

local t2 = { 1, ["a"] = 2, [0] = 3, [2] = 3, 5, 7, [3] = 1 }
assert(t2["a"] == 2)
assert(t2[0] == 3)
assert(t2[2] == 5)
assert(t2[3] == 7)
