        state
        y <- x1
        z1 <- x2
[C1]    if z1 != 0 goto B1
        state
        goto E1
[B1]    z1 <- z1 - 1
        print z1
        print y
        y <- y + 1
        goto C1
