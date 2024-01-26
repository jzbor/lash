# Church Encodings

After [Alonzo Church](https://en.wikipedia.org/wiki/Alonzo_Church) introduced the lambda calculus he also introduced means to represent different sorts of data with it.

## Booleans

Booleans most often represent a truth value and can be either `true` or `false`.
In church encodings they are represented by a function that takes two inputs and discards either the first or second value:
```
TRUE := λx . λy . x
FALSE := λx . λy . y
```

In `lash` the standard library gives definitions for `TRUE` and `FALSE` as well as several operations such as `AND`, `OR` and `NOT`.

## Numerals

A positiv number `n` is encoded by applying a function `f` `n` times to an argument `x`:
```
0 := λf . λx . x
1 := λf . λx . f x
2 := λf . λx . f (f x)
3 := λf . λx . f (f (f x))
  .
  .
  .
```
In `lash` Church Numerals can be used by prefixing a demoniator with the `$` sign.
This has to be enabled either at interpreter invocation or [at runtime](directives.html#set-key-value).
Operations are defined in the standard library.

Example:
```
[λ] @usestd
@usestd

[λ] @set numerals true
@set numerals true

[λ] !resolve $3
\f . \x . f (f (f x))

[λ] !normalize (ADD $1 $2)
\f . \x . f (f (f x))
```

## See also
* [Wikipedia: Church encoding](https://en.wikipedia.org/wiki/Church_encoding)
