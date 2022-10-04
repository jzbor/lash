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

## Example
```
[λ] CONS
\h . \t . \c . \n . c h (t c n)

[λ] list = CONS $2 (CONS $3 (CONS $1 NIL))
[λ] list
(\h . \t . \c . \n . c h (t c n)) (\f . \x . f (f x)) ((\h . \t . \c . \n . c h (t c n)) (\f . \x . f (f (f x))) ((\h . \t . \c . \n . c h (t c n)) (\f . \x . f x) (\c . \n . n)))

[λ] :mode normalize
Changed mode

[N] list
\c . \n . c (\f . \x . f (f x)) (c (\f . \x . f (f (f x))) (c (\f . \x . f x) n))

[N] result1 = list ADD $0
[N] result1
\f . \x . f (f (f (f (f (f x)))))

[N] ADD $2 $4
\f . \x . f (f (f (f (f (f x)))))

[N] :store result2
Added variable mapping for 'result2'

[N] :eq result1 result2
true
```

## Commands
* `:: <comment>`: comment (gets ignored)
* `:alpha <name1> <name2>`: checks if the terms are alpha equivalent
* `:builtins` list builtin terms
* `:debruijn [term]`: print DeBruijn index form of the (last) term
* `:echo <msg>`: output message to stdout
* `:eq <name1> <name2>`: checks if the terms normal forms are alpha equivalent
* `:hist` show history
* `:info`: get useful information on the last evaluated lambda expression
* `:mode <normalize | reduce | validate>`: switch mode
* `:normal [term]` normalize (last) term
* `:normalize <variable>` normalize variable (store result)
* `:print <name>`: print assigned value for variable or builtin
* `:reduced [term]` perform one reduction on (last) term
* `:reduce <variable>` perform one reduction on variable (store result)
* `:steps` print reduction steps of last term
* `:store <name>` store last term into a variable
* `:strategy <normal | applicative>`: switch reduction strategy
* `:vars` list variables

## Syntax
```
line        := command | lambda
command     := :keyword args
assignment  := variable-name = lambda
variable    := [a-zA-Z0-9_'-]*
numeral     := $[1-9][0-9]*
lambda-sign := '\' | 'λ'
```

### Lambda Syntax
```
lambda      := abstraction | application
abstraction := lambda-sign variable-list . lambda
application := group group*
group       := variable | numeral | (lambda)
```

