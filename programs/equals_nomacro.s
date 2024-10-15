# Set up unconditional jump
    z1 <- z1 + 1

[C] if x1 != 0 goto A
# x1 = 0
    if x2 != 0 goto END
# x1 = x2 = 0
    y <- y + 1
    if z1 != 0 goto END

# x1 != 0
[A] if x2 != 0 goto B
# x1 != 0, x2 = 0
    if z1 != 0 goto END

# x1 != 0, x2 != 0, decrement both
[B] x1 <- x1 - 1
    x2 <- x2 - 1
    if z1 != 0 goto C
