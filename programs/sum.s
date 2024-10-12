    state
    y <- x1
    z1 <- x2
[C] if z1 != 0 goto B
    state
    goto END
[B] z1 <- z1 - 1
    print z1
    print y
    y <- y + 1
    goto C
