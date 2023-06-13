# Lambda Shell
This is a simple REPL shell for [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
I wrote it mostly for playing around a little bit with the lambda calculus.
Some parts of it are wildly inefficient, but it is fine for education purposes.

The shell can evaluate lambda expressions, assignments and macrso.
It is not fully tested yet, but you can help with that :).

# Example

There are builtin terms `SUCC` and `NIL` to create natural numbers using church numerals:
```
[λ] one := SUCC NIL
one := SUCC NIL

[λ] two := SUCC (SUCC NIL)
two := SUCC (SUCC NIL)

[λ] !normalize (ADD one two)
\f . \x . f (f (f x))
```

You can select different reduction strategies at runtime:
```
[λ] !vnormalize (AND TRUE FALSE)
AND TRUE FALSE
(\q . TRUE q TRUE) FALSE
(\q . (\y . q) TRUE) FALSE
(\q . q) FALSE
FALSE

[λ] #set strategy normal
#set strategy normal

[λ] !vnormalize (AND TRUE FALSE)
AND TRUE FALSE
(\q . TRUE q TRUE) FALSE
TRUE FALSE TRUE
(\y . FALSE) TRUE
FALSE
```

## Features
* normal (leftmost-outermost) and applicative (leftmost-innermost) reduction strategies
* capture-avoiding substitution
* simple constructs provided via builtins
* macros for processing lambda terms
