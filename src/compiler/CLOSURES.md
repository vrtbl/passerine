# Thinking Through Closures
Passerine, as a language, needs closures.
Implementing closures can be a bit tricky,
and I've been having a bit of trouble with it.
I hope to lay out how closures will work in Passerine,
And then use this as an implementation guide.

While compiling a function, the compiler
may come across undeclared references.
This is not much of a surprise, consider this:

```
number = 420.69
capture = w -> {
    number
}
```

Inside the `capture` function, `number` is not defined - rather,
it exists in the enclosing scope.
The compiler needs to do two things:

1. Lift `number` off the stack and onto the heap
2. Store a reference to `number` in `capture`.

For this to work, here's what the compiler does:

1. In the enclosing compiler, the `Heap` `number` opcode is emitted
   *if* `number` is not already on the heap.
   `number` is then marked as having been moved to the heap.
2. In the current compiler, the captured variable, along with its location
   (frame + relative stack position) are added to the `lambda`'s `captured` vec.
3. Whenever the compiler emits a `Save`/`Load` for a variable,
   it emits `SaveCap`/`LoadCap` instead.

When the VM encounters a lambda definition:

1. It wraps the lambda bytecode object in a closure.
2. It uses the captured vec to store references to captured variables.
3. It pushes the new closure object onto the stack.
