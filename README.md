# Coastal: C wrapper generator for Rust

This is an early work in progress.

Coastal is a ~~sea~~ C wrapper generator.
It parses Rust functions and types using macros, generating Rust wrapper functions and a C header file for you.

Goals:

- Minimise the amount of code needed for a C wrapper.
You won't need to write `extern "C"` functions yourself.

- Be safe. You won't need to write unsafe code yourself.

- Be extensible, and flexible with that extensibility.

- Support plain C with optional C++ extensions, similar to cbindgen.
