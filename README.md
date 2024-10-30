# S Language

Simple barebones programming language used in a logic and computability course
in uni. Decided to implement an interpreter as a weekend project for fun and
procrastination. No relation to the shader language.

## CLI usage

To run a program simply pass the file containing the program code as an
argument:

```
slang program.s
```

To instead print the Gödel number associated with a program (as a series of
prime exponents), use the `-p` option:

```
slang -p program.s
```

## Language specification

A _program_ in S Language ("slang") is a finite series of instructions. The
language is a simple state machine, where the state encompasses the value of all
variables and the program counter.

### Variables

All variables in slang are of unsigned integer type. Variables can be classified
as input, auxiliary, or output. There are infinite input variables
`x1, x2, x3...`, as well as auxiliary variables `z1, z2, z3...`, and a single
output variable `y` (in practice, the number of input and auxiliary variables is
finite, but it's addressed with a 64-bit integer so you shouldn't ever run out).

All variables follow the naming convention above.

All auxiliary variables and the output variable are initialized to `0`. Input
variables may be set when running a program. Any unset input variables are
initialized to `0`.

### Labels

Labels are used for jump instructions, and take the form `Ax, Bx, Cx, Dx, Ex`
where `x` is a positive integer greater than or equal to one.

### Instructions

The language has only three instructions, with one alternative syntax provided
for convenience:

| Instruction        | Function                                                                                                             |
|--------------------|----------------------------------------------------------------------------------------------------------------------|
| `v <- v + 1`       | Increment the value of a variable by one.                                                                            |
| `v <- v - 1`       | Decrement the value of a variable by one. If it is zero, the value remains unchanged.                                |
| `if v != 0 goto L` | Conditional jump. Move the program counter to the instruction marked with label `L` if the value of `v` is not zero. |

Where `v` is a variable name and `L` is a label. A jump to an undefined label
terminates execution immediately. Labels **must** be unique; a label
redefinition will result in a parser error.

Any instruction may be preceded by a label in brackets:

```
[A1]    x1 <- x1 - 1
        if x1 != 0 goto A1
```

Some additional "meta" instructions are provided for utility, that do not alter
the execution state:

| Instruction | Function                                                |
|-------------|---------------------------------------------------------|
| `nop`       | Does nothing. Useful for jumping out of macros.         |
| `print v`   | Prints the value of a variable to stdout for debugging. |
| `state`     | Prints the entire state of execution to stdout.         |

Leading and trailing whitespace is ignored. It is recommended to align
instructions for readability.

### Comments

Any line starting with `#` is interpreted as a comment and ignored:

```
# Loop until x1 is zero
[A1]    x1 <- x1 - 1
        if x1 != 0 goto A1
```

## Macros

To make slang somewhat more powerful and convenient, macros can be defined using
the `@def` directive. Note that macros are _not_ subroutines and do not exist at
runtime, rather all macros are expanded when parsing a program.

A macro declaration consists of a `@def` directive followed by a _pattern_. The
pattern is matched literally, replacing any named tokens in curly brackets. It
is immediately followed by a macro definition, the series of instructions the
macro expands to, terminated by the `@end` directive:

```
# Unconditional jump
@def goto {label}
    $a <- $a + 1
    if $a != 0 goto label
```

### Automatic variables

Inside macro definitions, _automatic variables_ can be used. This is a utility
provided to prevent variable conflict when using macros. An automatic variable
is defined by a dollar sign `$` followed by a variable name consisting of
alphanumeric characters (upper and lowercase) and underscores. The variable `$a`
in the above example is an automatic variable.

When a macro is expanded, each distinct automatic variable in its definition
will be replaced by the first unused auxiliary variable `zi`, ensuring its value
is not accidentally overwritten elsewhere, and that using the macro doesn't
unintentionally overwrite unrelated variables.

### Automatic labels

Similarly, if a macro containing a label is used more than once in a program, a
way to differentiate each one is needed to prevent conflicting labels.

For this purpose, _automatic labels_ can be used within macro definitions by
prefixing a label name with `%` in both its definition and use:

```
# Assign zero to a variable
@def {v} <- 0
[%A1]   v <- v - 1
        if v != 0 goto %A1
@end
```

Automatic labels will be replaced with available labels on expansion, analogous
to automatic variables.

### Nested macros

Macros can be used within other macros to compose more complex programs:

```
# Assign the value of v2 to v1, preserving the former
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
```

Note that _recursive_ macros are not allowed, as they will result in an infinite
expansion. The same applies to circular references.

### Conflicting definitions

If two macro patterns match an expression, whichever was declared first will
match. For example, the expression `x1 <- 0` will match the last two examples,
but since the zero-assign macro is defined first, it will match.

As a general rule, more restrictive patterns should be defined first.

Instructions will always be matched before macros, so a macro whose pattern
matches an instruction will never be expanded.

## Prologue

The three macros used as examples above are defined in the _prologue_ loaded
before any program, and are available to use. A list of all macros defined in
the prologue follows:

| Pattern                       | Function                                               |
|-------------------------------|--------------------------------------------------------|
| `goto {label}`                | Unconditional jump.                                    |
| `if {v} = 0 goto {label}`     | Jump if `v` is zero.                                   |
| `if {v1} < {v2} goto {label}` | Compare two variables and jump if `v1 < v2`.           |
| `{v} <- 0`                    | Assign zero to a variable.                             |
| `{v1} <- {v2}`                | Assign the value of `v2` to `v1`. `v2` is left as is.  |
| `{v} <- {a} + {b}`            | Assign the sum of variables `a` and `b` to `v`.        |
| `{v} <- {a} - {b}`            | Assign the difference of variables `a` and `b` to `v`. |
| `{v} <- {a} * {b}`            | Assign the product of variables `a` and `b` to `v`.    |
| `{v} <- {a} / {b}`            | Assign the quotient of variables `a` and `b` to `v`.   |