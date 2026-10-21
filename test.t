# Comprehensive sample for tiny_compiler. Comments are removed by Phase 0.
// C-style single-line comments are accepted as well.
def add(a, b):
    sum_value = a + b
    return sum_value

def mix(a, b, c):
    shifted = a << 2
    masked = b & 7
    flipped = (shifted ^ masked) | c
    return flipped

def sum_seven(a, b, c, d, e, f, g):
    total = a + b + c + d + e + f + g
    return total

def main():
    x = 5  + 10
    k = 20
    y = 3
    z = add(x, y)
    quotient = z / 2
    shifted_right = z >> 1
    negative = -y
    total = 0
    for i in 0..4:
        total = total + i
    total = total * 1
    if z > y:
        total = total + mix(z, y, 1)
    else:
        total = total - 1
    if x != y:
        total = total + (x | y)
    if x <= y:
        total = total + 100
    else:
        total = total + quotient
    if x == 5:
        total = total + shifted_right
    if x >= 0:
        total = total + 0
    if x < y:
        total = total + negative
    stack_test = sum_seven(1, 2, 3, 4, 5, 6, 7)
    print(x, y, z, quotient, shifted_right, total, stack_test)
    return 0
