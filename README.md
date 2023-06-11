# Lambda Shell
This is a simple REPL shell for [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
I wrote it mostly for playing around a little bit with the lambda calculus.
Some parts of it are wildly inefficient, but it is fine for education purposes.

The shell can evaluate lambda expressions, assignments and macrso.
It is not fully tested yet, but you can help with that :).

## Features
* normal (leftmost-outermost) and applicative (leftmost-innermost) reduction strategies
* capture-avoiding substitution
* simple constructs provided via builtins
* macros for processing lambda terms
