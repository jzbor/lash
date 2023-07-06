# Directives
Directives are a mechanism to influence certain behaviours of the interpreter.
They do not interact with Lambda terms directly, but can change how for example macros are evaluated.

A directive always has to be on its own line and starts with an `@`:
```
@echo "hello world"
```

## Available directives
### `@echo "<string>"`
The `@echo` directives prints out the string that is passed to it to stdout.
Make sure to put the argument in parenthesis.

### `@include "<path>"`
You can include files that are then directly evaluated.
Directives in the included file might also have an effect on following code, so make sure to import files before actually setting important interpreter behavior.

### `@set <key> <value>`
Set compiler behavior with this directive.
Settings you can use are:
* `strategy normal|applicative` - changes the reduction strategy (see [Strategies](./strategies.md))

### `@usestd`
There is a **unstable** standard library, which is a collection of a few useful terms.
By running `@usestd` these get included as named terms.

You can find the terms in [`src/stdlib.rs`](https://github.com/jzbor/lash/blob/master/src/stdlib.rs).

