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

Here's an annotated, yet otherwise index-card-sized subset:

```
-- NOTE: not finalized
-- Assignment (=), Macro (~>)
for = var sequence do ~> {
    -- Function (->)
    block = var -> do

    -- Fiber (|>)
    loop = s |> {
        -- Function call (f x ...)
        block (next s)

        -- Run fiber (!)
        when (not (done s)) (loop s)!
    }

    loop (iter sequence)
}

-- Macro usage (f a b c ...)
for x 1..5 {
    print x
}
```

## Getting Started
If you just want to see what Passerine can do:

> NOTE: Passerine is in an early stage of development.
It is not even near turing-complete at this stage, so don't @ me (yet).

1. Clone this git repository.
2. Build Passerine with cargo.
3. Run the CLI on one of the examples: `cargo run -- examples/hello_world.p`.

If you'd like to contribute:

1. Read `CONTRIBUTING.md`.
2. Get up Passerine using the above instructions
3. Open issues and pull requests disclosing bugs or implementing improvements

> Fun Note: I didn't have access to wifi when I wrote the compiler until version 0.5.0.
As such, this compiler was developed completely from memory, without access to the internet for any sort of anything - no Stack Overflow, no Rust documentation, etc.
If you see any obvious errata in the compiler design, open an issue or create a pull request.

# Roadmap
Version, ∆ Done, Milestone, -- Date

0. ∆ Start project
    1. ∆ Lexer
    2. ∆ Parser
    3. ∆ Bytecode generator
    4. ∆ VM
        1. ∆ Local Variables -- Today
        2. Block Statements
    5. Unary datatypes
        1. Numbers
        2. Strings -- This weekend (28M)
    6. Functions
        1. Closures
        2. Tail Recursion
    7. Algebraic Datatypes (Tuple, Union, Struct, Map)
        1. Pattern Matching -- Next weekend (4A)
    7. Fibers
        1. Error Handling -- Weekend after (11A)
    8. CLI
    9. Standard Library
        1. Numeric Tower
    10. Clean up and optimize
1. First Stable Release -- Before May (2M)
    1. Tests
    2. Documentation Generation
    3. Package Manager
2. Macros + Backwards-Compatibility Breaking-Changes (BCBC)
3. Parallelism
4. TBD
