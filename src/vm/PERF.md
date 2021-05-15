# Some performance notes
(Cargo Flamegraph is your friend - this doc is formatted chat history from discord)

ran some flamegraphs today.

anway, tldr notes for future me:

- call, closure, and ffi_call ops take the most time. this is mainly because of extra clone and drops (vaporization would help fix this)
- stamp is really slow. I've actually removed it in the records branch.

This isn't a bench on the most recent version of the interpreter, btw. I should actually try that now. ok, most recent version, some more notes.

- stack is grown, and shrunk, this takes time. Maybe start by allocating larger stack for VM at beginning.
- pop_data takes a while, will have to look into that
- closure is significantly faster. I notice now that in some ops I allocate a vec for a short period of time - there are faster ways to do this sans allocation.
- call still takes a while, mainly seems to be from allocating a boxed function. Will have to look into this; this shouldn't incur an allocation.
- don't use a loop to clear stack, I think there's a dedicated op for this in the stdlib, will have to check.

I'm benchmarking some fibonacci code fwiw:

```
syntax 'if x y 'else z {
    (magic "if" (x, () -> y, () -> z)) ()
}

syntax a 'is 'greater 'than b {
    magic "greater" (a, b)
}

fib = n -> if (2 is greater than n) {
    n
} else {
    fib (n - 1) + fib (n - 2)
}

print (fib 20)
```

I think the thing that needs to be done the most is moving control flow to dedicated vm instructions rather than through FFI. FFI was a nice way to show what is possible with the macro system, but isn't scalable.

I tried with some big files, and I guess important thing to note is that compilation is pretty fast. I need to revisit the lexer however.
