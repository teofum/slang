        y <- y + 1
        z1 <- z1 + 1
[A1]    if x1 != 0 goto C1
        if z1 != 0 goto E1
[C1]    x1 <- x1 - 1
        if y != 0 goto B1
        y <- y + 1
        if z1 != 0 goto A1
[B1]    y <- y - 1
        if z1 != 0 goto A1