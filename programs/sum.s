    state
    y <- x1
    state
    z1 <- x2
    state
[C] print z1
    if z1 != 0 goto B
    print z1
    goto END
[B] z1 <- z1 - 1
    print z1
    print y
    y <- y + 1
    print z1
    print y
    goto C
