local state = 0

function get_one()
    state = 1
    return 1
end

function get_two()
    state = 2
    return 2
end

assert(get_one() == 1 or get_two() == 1 or 0 == 1)
assert(state == 1)

assert((get_one() == 1 and get_two() == 1) == false)
assert((get_one() == 2 and get_two() == 2) == false)
assert((get_one() == 1 or get_two() == 1) == true)
assert((get_one() == 2 or get_two() == 2) == true)
assert((get_one() == 1 and get_two() == 2) == true)
assert((get_one() == 2 or get_two() == 1) == false)
