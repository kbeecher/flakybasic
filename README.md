# FlakyBASIC

A Rust-based implementation of a [Tiny BASIC](https://en.wikipedia.org/wiki/Tiny_BASIC) interpreter.

This is a project for me to learn Rust programming. I'm a beginner to the language and wanted something a bit more challenging than 'Hello, World' or even a simple todo tracker.

Therefore, don't expect this to be a particularly good implementation. It's likely got bugs and inefficiencies, and uses non-idiomatic Rust practices. We've all got to start somewhere.

## Version History

### v0.3.0

Enhancements:

* Added a `for`-`next`[-`step`] loop.
* The `print` command now takes an arbitrary number of arguments.

### v0.2.0

Added the ability to `load` and `save` programs. Programs are saved in human-readable text format.

### v0.1.0

A basic working version. Supported only the fundamental keywords:

* `rem`
* `print`
* `let`
* `if`-`then`
* `goto`
* `gosub`-`return`
* `input`
* `list`
* `run`
* `end`

Notable omissions and differences from the original TinyBASIC (some addressed in later versions):

* ~~No saving or loading of programs~~.
* ~~`print` accepts only one argument.~~
* `input` accepts only one argument.
* `goto` and `gosub` accept only numbers.
* No `clear` keyword.
* ~~No loop construct.~~
* No functions.
* ~~Integers only, no floating point values.~~
* `let` keyword not optional