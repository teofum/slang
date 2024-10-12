    y <- 0
    z1 <- x1
[C] z3 <- x2 - z1
    if z3 != 0 goto END
    z2 <- z1 - x2
    z1 <- z2
    y <- y + 1
    goto C