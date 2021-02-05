# Foreign Functional Interface

Currently, the VM keeps a table of pointers to functions with the signature:

```
Fn(Data) -> Result<Data, String>
```

This is a FFI function.

FFI functions are bound at compile time.

When?

Current compilation steps:

- lex
    - Source -> Vec<Token>
- parse
    - Vec<Token> -> AST
- desugar
    - AST -> CST
- gen
    - CST -> Lambda

wrap it in a Closure, and you can run it!

So, probably would need to be passed in at the gen step
Like a FFI module.

Let's implement it!

---

I need to clean up the stylistic conventions in this codebase.

if assignments occur on multiple lines, align them at the `=`

Same goes for match statements and `=>`

If there is a return in a match statement, put it in a block with a semicolon: `{ retun x; }`
If a function has a return, use a trailing semicolon.

Maybe just have rustfmt do the work?
