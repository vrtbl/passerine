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
This is a loose roadmap of the features
we plan to add to Passerine to reach 1.0.
**Bold** items are a work-in-progress.
*Italic* items are currently being planned.

- [X] 0: Start Project
  - [X] 0.1: Lexer
  - [X] 0.2: Parser
  - [X] 0.3: Bytecode Generator
  - [X] 0.4: Virtual Machine <details><summary>**Details**</summary>
    - [X] Local Variables
    - [X] Block Expressions
    </details>
  - [X] 0.5: Unary Datatypes <details><summary>**Details**</summary>
    - [X] NaN Tagging
    - [X] Numbers
    - [X] Strings
    </details>
  - [ ] **0.6: Functions** <details><summary>**Details**</summary>
    - [X] Block Scope
    - [ ] **Closures**
    - [ ] **Operators**
    </details>
  - [ ] 0.7: *Compound Data Types* <details><summary>**Details**</summary>
    - [ ] *Tuple*
    - [ ] *Union*
    - [ ] *Record*
    - [ ] *Map*
    </details>
  - [ ] 0.8: *Hygenic Macros* <details><summary>**Details**</summary>
    - [ ] Pattern Matching
    - [ ] Modules
    - [ ] Types and Traits (Labels)
    </details>
  - [ ] 0.9: *Fibers* <details><summary>**Details**</summary>
    - [ ] Coroutines
    - [ ] **Error Handling**
    </details>
  - [ ] 0.10: [**CLI**](https://github.io/vrtbl/aspen) <details><summary>**Details**</summary>
    Visit the [Aspen](https://github.io/vrtbl/aspen) repository to discover the status of Passerine's CLI and package manager.
    </details>
  - [ ] 0.11: Standard Library <details><summary>**Details**</summary>
    - [ ] FFI
    - [ ] I/O
    - [ ] Math
    - [ ] Random
    - [ ] Time
    - [ ] Networking
    </details>
  - [ ] 0.12 Prepare for Stable Release <details><summary>**Details**</summary>
    - [ ] **Website**
    - [ ] **Documentation**
    - [ ] Optimizations
    - [ ] Resources
    </details>
- [ ] 1: First Stable Release
    - [ ] 1.1: Automated Tests 
    - [ ] 1.2: Documentation Generation
    - [ ] 1.3: **Package Manager**
    - [ ] ...
- [ ] 2: Parallelism
- [ ] ... 
