# Untyped Lambda Calculus

There are two basic notions in the Lambda calculus:
* **Abstractions**: The idea is that we define a function by specifying which variable we want to be able to substitute. A very simple example is the identity function, which takes a term as parameter and maps it to itself: `λx . x`
* **Applications**: This allows us to actually evaluate a function. We derive a new term by substituting the variable in the abstraction with the argument term: `(λ x . x x) (y z)` → `(y z) (y z)`

The Lambda Shell allows you to use either `λ` or `\` as lambda symbol for abstractions.

You are allowed to abbreviate multiple abstractions as one: `\x . \y . x y` = `\x y . x y`.

In batch mode (when processing `.lsh` files) you are required to end every statement with a semicolon (`;`).
This is not necessary in interactive mode.


## Assignments
In order to keep this readable the Lambda Shell allows you to name terms and reuse them.
To assign a name to a term the `:=` tag is used:
```
one := \f . \x . f x
true := \x . \y . y
```

These terms are referred to with their name until they get resolved explicitly or during reduction (see [macros](./macros.md)).

