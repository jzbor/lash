# Strategies

When reducing Lambda terms there are a few different strategies we can pick.

## Applicative aka leftmost-innermost
This strategy always replaces inner terms first:
```
[λ] @set strategy applicative
@set strategy applicative

[λ] !reduce ((\x . (\y . y y) x) z)
(\x . x x) z
```

## Normal aka leftmost-outermost
This strategy always replaces outer terms first:
```
[λ] @set strategy normal
@set strategy normal

[λ] !reduce ((\x . (\y . y y) x) z)
(\y . y y) z
```
