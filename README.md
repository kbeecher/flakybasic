# FlakyBASIC

A Rust-based implementation of a [Tiny BASIC](https://en.wikipedia.org/wiki/Tiny_BASIC) interpreter.

This is a project for me to learn Rust programming. I'm a beginner to the language and wanted something a bit more challenging than 'Hello, World' or even a simple todo tracker.

Therefore, don't expect this to be a particularly good implementation. It's likely got bugs and inefficiencies, and uses non-idiomatic Rust practices. We've all got to start somewhere.

## Version History

### v0.1.0

A basic working version. Supports only the fundamental keywords:

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

Some notable omissions to be filled in later versions:

* No saving or loading of programs.
* `print` and `input` accept only one argument.
* `goto` and `gosub` accept only numbers.
* No `clear` keyword.
* No loop construct.
* No functions.
* Integers only, no floating point values.