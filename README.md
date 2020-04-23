# Passerine
Welcome to Passerine!
Passerine is a small, concise, extensible programming language, powered by a VM written in Rust.
It was inspired by Scheme, OCaml, and Wren.
Passerine has just about five concepts:

- Patterns
- Assignment
- Functions
- Macros
- Fibers

Here's a taste:

```
-- This is a comment
->> = data functions... -> {
    match functions [
        []    -> data
        [h|t] -> (->> (h data) t...)
    ]
}

numbers = 0..5
->> (filter even?) (map square) sum (x -> x / 2)

-- [0 2 4]
-- [0 4 16]
-- 20
-- 10
```

## Getting Started
If you just want to see what Passerine can do:

> NOTE: Passerine is in an early stage of development.
It is not even near turing-complete at this stage, so don't @ me (yet).

1. Clone this git repository.
2. Build Passerine with cargo.
3. Run the tests.

If you'd like to contribute:

1. Read `CONTRIBUTING.md`.
2. Get Passerine.
3. If you find anything, open an issue / pull request.

# Roadmap
| Version   | Milestone                   | Status (Planning, WIP, Stable) | Goal        |
|-----------|-----------------------------|--------------------------------|-------------|
| **0**     | Start Project               | WIP                            |             |
| **0.1**   | Lexer                       | Stable                         |             |
| **0.2**   | Parser                      | Stable                         |             |
| **0.3**   | Bytecode Generator          | Stable                         |             |
| **0.4**   | VM                          | Stable                         | 2020-04-25✓ |
| **0.4.1** | Local Variables             | Stable                         |             |
| **0.4.2** | Block Expressions           | Stable                         |             |
| **0.5**   | Unary Datatypes             | Stable                         | 2020-05-02✓ |
| **0.5.1** | Nan Tagging                 | Stable                         |             |
| **0.5.2** | Numbers                     | Stable                         |             |
| **0.5.3** | Strings                     | Stable                         |             |
| 0.6       | Functions                   | WIP                            | 2020-05-09  |
| 0.6.1     | Block Scope                 | Planning                       |             |
| 0.6.2     | Closures                    | Planning                       |             |
| 0.6.3     | Operators                   | Planning                       |             |
| 0.6.4     | Custom Operators            |                                |             |
| 0.7       | Alg. Structs.               |                                | 2020-05-16  |
| 0.7.1     | Tuple                       |                                |             |
| 0.7.2     | Union                       |                                |             |
| 0.7.3     | Record                      |                                |             |
| 0.7.4     | Map                         |                                |             |
| 0.7.5     | Pattern Matching            |                                |             |
| 0.8       | Fibers                      |                                | 2020-05-23  |
| 0.8.1     | Error Handling              |                                |             |
| 0.9       | CLI                         |                                |             |
| 0.10      | Standard Library            |                                |             |
| 0.10.1    | FFI                         |                                |             |
| 0.10.1    | I/O                         |                                |             |
| 0.10.2    | Math & Numeric Tower        |                                |             |
| 0.10.3    | Random                      |                                |             |
| 0.10.4    | Time                        |                                |             |
| 0.10.5    | Networking                  |                                |             |
| 0.11      | Clean up for Stable Release |                                |             |
| 0.11.1    | Website                     |                                |             |
| 1         | First Stable Release        |                                | 2020-06-09? |
| 1.1       | Automated Tests             |                                |             |
| 1.2       | Documentation Generation    |                                |             |
| 1.3       | Package Manager             |                                |             |
| 2         | Macros & (BCBC)             |                                |             |
| 3         | Parallelism                 |                                |             |
| 4.        | TBD                         |                                |             |
