        y <- 0
        z1 <- x1
[C1]    z3 <- x2 - z1
        if z3 != 0 goto E1
        z2 <- z1 - x2
        z1 <- z2
        y <- y + 1
        goto C1