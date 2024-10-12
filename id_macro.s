@def goto {label}
$a <- $a + 1
if $a != 0 goto label
@end

[A] if x1 != 0 goto B
    goto END
[B] x1 <- x1 - 1
    y <- y + 1
    z1 <- z1 + 1
    goto A