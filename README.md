# Lambda Shell
This is a simple REPL shell for [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
I wrote it mostly for playing around a little bit with the lambda calculus.
Some parts of it are wildly inefficient, but it is fine for education purposes.

The shell can evaluate lambda expressions or execute [commands](#commands).
It is not fully tested yet, but you can help with that :).

## Features
* normal (leftmost-outermost) and applicative (leftmost-innermost) reduction strategies
* capture-avoiding substitution
* simple constructs provided via builtins
* church numerals
* commands for processing lambda terms

## Commands
* `:: <comment>`: comment (gets ignored)
* `:alpha <name1> <name2>`: checks if the terms are alpha equivalent
* `:builtins` list builtin terms
* `:debruijn [term]`: print DeBruijn index form of the (last) term
* `:echo <msg>`: output message to stdout
* `:eq <name1> <name2>`: checks if the terms normal forms are alpha equivalent
* `:hist` show history
* `:info`: get useful information on the last evaluated lambda expression
* `:normal <term>` normalize term
* `:print <name>`: print assigned value for variable or builtin
* `:reduce <term>` perform one reduction on term
* `:steps` print reduction steps of last term
* `:store <name>` store last term into a variable
* `:vars` list variables

## Syntax
```
line        := command | lambda
command     := :keyword args
assignment  := variable-name = lambda
variable    := [a-zA-Z0-9_'-]*
numeral     := $[1-9][0-9]*
```

### Lambda Syntax
```
lambda      := abstraction | application
abstraction := \variable-list . lambda
application := group group*
group       := variable | numeral | (lambda)
```

