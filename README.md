![Passerine Logotype](https://raw.githubusercontent.com/vrtbl/passerine/master/Logotype.png)

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
            [head & tail] -> { body head; remaining tail }
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

| Version   | Milestone                   | Status (P, W, S) | Stable Goal |
|-----------|-----------------------------|------------------|-------------|
| **0**     | Start Project               | Stable           |             |
| **0.1**   | Lexer                       | Stable           |             |
| **0.2**   | Parser                      | Stable           |             |
| **0.3**   | Bytecode Generator          | Stable           |             |
| **0.4**   | VM                          | Stable           | 2020-04-25✓ |
| **0.4.1** | Local Variables             | Stable           |             |
| **0.4.2** | Block Expressions           | Stable           |             |
| **0.5**   | Unary Datatypes             | Stable           | 2020-05-02✓ |
| **0.5.1** | Nan Tagging                 | Stable           |             |
| **0.5.2** | Numbers                     | Stable           |             |
| **0.5.3** | Strings                     | Stable           |             |
| 0.6       | Functions                   | WIP              | 2020-08-08  |
| **0.6.1** | Block Scope                 | Stable           |             |
| 0.6.2     | Closures                    | WIP              |             |
| 0.6.3     | Operators                   | WIP              |             |
| 0.7       | Alg. Structs.               | Planning         | 2020-05-15  |
| 0.7.1     | Tuple                       | Planning         |             |
| 0.7.2     | Union                       | Planning         |             |
| 0.7.3     | Record                      | Planning         |             |
| 0.7.4     | Map                         | Planning         |             |
| 0.8       | Hygenic Macros              | Planning         | 2020-05-29  |
| 0.8.1     | Pattern Matching            | Planning         |             |
| 0.8.2     | Types and Traits            | Planning         |             |
| 0.9.0     | Fibers                      |                  |             |
| 0.9.1     | Error Handling              | WIP              |             |
| 0.10      | CLI                         |                  |             |
| 0.11      | Standard Library            |                  |             |
| 0.11.1    | FFI                         |                  |             |
| 0.11.1    | I/O                         |                  |             |
| 0.11.2    | Math & Numeric Tower        |                  |             |
| 0.11.3    | Random                      |                  |             |
| 0.11.4    | Time                        |                  |             |
| 0.11.5    | Networking                  |                  |             |
| 0.12      | Clean up for Stable Release |                  |             |
| 0.12.1    | Website                     | Planning         |             |
| 1.0.0     | First Stable Release        |                  |             |
| 1.1.0     | Automated Tests             |                  |             |
| 1.2.0     | Documentation Generation    |                  |             |
| 1.3.0     | Package Manager             |                  |             |
| 2.0.0     | Parallelism                 |                  |             |
| ...       | TBD                         |                  |             |
