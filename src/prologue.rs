pub const PROLOGUE: &'static str = r"
@def goto {label}
    $a <- $a + 1
    if $a != 0 goto label
@end

@def if {v} = 0 goto {label}
        if v != 0 goto %E
        goto label
[%E]    nop
@end

@def if {v1} < {v2} goto {label}
        $a <- v2 - v1
        if $a != 0 goto label
@end

@def {v} <- 0
[%L]    v <- v - 1
        if v != 0 goto %L
@end

@def {v1} <- {v2}
        v1 <- 0
[%A]    if v2 != 0 goto %B
        goto %C
[%B]    v2 <- v2 - 1
        v1 <- v1 + 1
        $a <- $a + 1
        goto %A
[%C]    if $a != 0 goto %D
        goto %E
[%D]    $a <- $a - 1
        v2 <- v2 + 1
        goto %C
[%E]    nop
@end

@def {v} <- {a} + {b}
        v <- a
        $t <- b
[%C]    if $t != 0 goto %B
        goto %E
[%B]    $t <- $t - 1
        v <- v + 1
        goto %C
[%E]    nop
@end

@def {v} <- {a} - {b}
        v <- a
        $t <- b
[%C]    if $t != 0 goto %B
        goto %E
[%B]    $t <- $t - 1
        v <- v - 1
        if v != 0 goto %C
[%E]    nop
@end

@def {v} <- {a} * {b}
        v <- 0
        $t <- b
[%B]    if $t != 0 goto %A
        goto %E
[%A]    $t <- $t - 1
        $u <- a + v
        v <- $u
        goto %B
[%E]    nop
@end

@def {v} <- {a} / {b}
        v <- 0
        $t <- a
[%C]    $u <- b - $t
        if $u != 0 goto %E
        $w <- $t - b
        $t <- $w
        v <- v + 1
        goto %C
[%E]    nop
@end

# Alt syntax macros
@def inc {v}
        v <- v + 1
@end

@def dec {v}
        v <- v - 1
@end

@def jnz {v} {label}
        if v != 0 goto label
@end

@def jze {v} {label}
        if v = 0 goto label
@end

@def jlt {v1} {v2} {label}
        if v1 < v2 goto label
@end

@def mov {v1} {v2}
        v1 <- v2
@end
";
