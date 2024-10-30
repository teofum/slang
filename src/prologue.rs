pub const PROLOGUE: &'static str = r"
@def goto {label}
    $a <- $a + 1
    if $a != 0 goto label
@end

@def if {v} = 0 goto {label}
        if v != 0 goto %E1
        goto label
[%E1]   nop
@end

@def if {v1} < {v2} goto {label}
        $a <- v2 - v1
        if $a != 0 goto label
@end

@def {v} <- 0
[%A1]   v <- v - 1
        if v != 0 goto %A1
@end

@def {v1} <- {v2}
        v1 <- 0
[%A1]   if v2 != 0 goto %B1
        goto %C1
[%B1]   v2 <- v2 - 1
        v1 <- v1 + 1
        $a <- $a + 1
        goto %A1
[%C1]   if $a != 0 goto %D1
        goto %E1
[%D1]   $a <- $a - 1
        v2 <- v2 + 1
        goto %C1
[%E1]   nop
@end

@def {v} <- {a} + {b}
        v <- a
        $t <- b
[%C1]   if $t != 0 goto %B1
        goto %E1
[%B1]   $t <- $t - 1
        v <- v + 1
        goto %C1
[%E1]   nop
@end

@def {v} <- {a} - {b}
        v <- a
        $t <- b
[%C1]   if $t != 0 goto %B1
        goto %E1
[%B1]   $t <- $t - 1
        v <- v - 1
        if v != 0 goto %C1
[%E1]   nop
@end

@def {v} <- {a} * {b}
        v <- 0
        $t <- b
[%B1]   if $t != 0 goto %A1
        goto %E1
[%A1]   $t <- $t - 1
        $u <- a + v
        v <- $u
        goto %B1
[%E1]   nop
@end

@def {v} <- {a} / {b}
        v <- 0
        $t <- a
[%C1]   $u <- b - $t
        if $u != 0 goto %E1
        $w <- $t - b
        $t <- $w
        v <- v + 1
        goto %C1
[%E1]   nop
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
