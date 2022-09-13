# Lambda Shell
This is a simple REPL shell for [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
I wrote it mostly for playing around a little bit with the lambda calculus.
Some parts of it are wildly inefficient, but it is fine for education purposes.

The shell can evaluate lambda expressions or execute [commands](#commands).
It is not fully tested yet, but you can help with that :).

## Commands
* `:builtins` list builtin terms
* `:echo <msg>`: output message to stdout
* `:hist` show history
* `:info`: get useful information on the last evaluated lambda expression
* `:steps` print reduction steps of last term
* `:store <name>` store last term into a variable
* `:vars` list variables

## Syntax
```
line        := command | lambda
command     := :keyword args
```

### Lambda Syntax
```
lambda      := abstraction | application
abstraction := \variable-list . lambda
application := group group*
group       := variable | (lambda)
```

