# Macros

Macros allow you to modify Lambda terms at interpretation time.
For example you might want to print out a term, resolve its named terms or reduce/normalize it.
Macros are only evaluated once when the term is initially parsed, after that tey are just replaced with the terms they return.

## Available Macros:
* `!cnormalize` - like `!normalize` but shows number of reductions performed
* `!debug` - prints out the argument term
* `!normalize` - reduce the term until it cannot be reduced further (this is the so-called normal form)
* `!reduce` - execute one reduction step on the argument term
* `!resolve` - resolve named terms and church numerals
* `!time` - time the macro execution of term inside
* `!vnormalize`/`!vreduce` - like `!normalize` and `!reduce`, but prints the reduction steps

## Examples
```
[λ] one := \f . \x . f x
one := \f . \x . f x

[λ] !resolve (one y)
(\f . \x . f x) y
```

```
[λ] one := \f . \x . f x
one := \f . \x . f x

[λ] !vnormalize (one (\y . y y) z)
one (\y . y y) z
(\x . (\y . y y) x) z
(\x . x x) z
z z
```
