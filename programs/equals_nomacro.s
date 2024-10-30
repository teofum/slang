# Set up unconditional jump
        z1 <- z1 + 1

[C1]    if x1 != 0 goto A1
# x1 = 0
        if x2 != 0 goto E1
# x1 = x2 = 0
        y <- y + 1
        if z1 != 0 goto E1

# x1 != 0
[A1]    if x2 != 0 goto B1
# x1 != 0, x2 = 0
        if z1 != 0 goto E1

# x1 != 0, x2 != 0, decrement both
[B1]    x1 <- x1 - 1
        x2 <- x2 - 1
        if z1 != 0 goto C1
