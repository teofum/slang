    y <- y + 1
    z1 <- z1 + 1
[A] if x1 != 0 goto C
    if z1 != 0 goto END
[C] x1 <- x1 - 1
    if y != 0 goto B
    y <- y + 1
    if z1 != 0 goto A
[B] y <- y - 1
    if z1 != 0 goto A