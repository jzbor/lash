# Lambda Shell
This is a simple REPL shell for [untyped lambda expressions](https://en.wikipedia.org/wiki/Lambda_calculus).
I wrote it mostly for playing around a little bit with the lambda calculus.
Some parts of it are wildly inefficient, but it is fine for education purposes.

The shell can evaluate lambda expressions or execute [commands](#commands).
It is not fully tested yet, but you can help with that :).

## TODOs:
* [X] `:hist` command
* [X] variable assignments
* [X] printing in default syntax
* [X] `:steps` command
* [X] `:store` command
* [X] `:variables` command
* [X] `:builtins` command
* [ ] cli arguments
* [ ] error handling
* [ ] add license
* [ ] consume leading/trailing spaces + check for left input in wrapper parser
* [ ] limit history size
* [ ] handle files/scripts
* [ ] pure parser can't handle applications properly

## Commands
* `:echo <msg>`: output message to stdout
* `:info`: get useful information on the last evaluated lambda expression

## Syntax
```
line        := command | lambda
command     := :keyword args
```

### Default Lambda Syntax
```
lambda      := abstraction | application
abstraction := \variable-list . lambda
application := group group*
group       := variable | (lambda)
```

### Pure Lambda Syntax (currently broken and not available)
```
lambda      := variable | abstraction | application
abstraction := (\variable-name . lambda)
application := (lambda lambda)
```

