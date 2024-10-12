pub const PROLOGUE: &'static str = r"
@def goto {label}
    $a <- $a + 1
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
";
