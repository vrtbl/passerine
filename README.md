[![Passerine Logotype](https://raw.githubusercontent.com/vrtbl/passerine/master/Logotype.png)](https://passerine.io)

[![Rust Build Status](https://github.com/vrtbl/passerine/workflows/Rust/badge.svg)](https://github.com/vrtbl/passerine/actions)
[![Crates.io](https://img.shields.io/crates/v/passerine.svg)](https://crates.io/crates/passerine)
[![Docs.rs](https://docs.rs/passerine/badge.svg)](https://docs.rs/passerine)

Welcome to Passerine!
Passerine is a small, concise, extensible programming language, powered by a VM written in Rust.
It was inspired by Scheme, OCaml, Rust, and Wren.
Here's a small taste:

```
-- defining a macro
syntax 'for 'each var 'in list do {
    body = var -> do

    loop = remaining -> {
        match remaining {
            () -> (),
            [head & tail] -> {
                body head
                remaining tail . loop
        }
    }

    loop list
}

-- using said macro
for each number in (reverse [1, 2, 3]) {
    print (number + "...")
}

print "Liftoff!"
```

## Getting Started
If you just want to see what Passerine can do:

> NOTE: Passerine is in an early stage of development.
It's just barely reached the stage of theoretically being Turing-complete,
but don't @ me yet.

1. Clone this git repository.
2. Build Passerine with cargo.
3. Run the tests.

If you'd like to contribute:

1. Get Passerine.
2. Start developing.
   `CONTRIBUTING.md` has general guidelines and discusses project structure

## Roadmap
See the [Project Roadmap](https://github.com/vrtbl/passerine/projects/1).
