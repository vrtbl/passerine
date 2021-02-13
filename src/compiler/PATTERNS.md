# Pattern Matching for fun ~~and for profit!~~

> This is a big file where I'm planning stuff.
> I'll delete it before this branch is merged into master.

What's the best way to implement pattern matching from a backend /runtime perspective?That's a tricky question. I'll go into some more depth here. Currently, the language I'm developing has support for tuples - so you can do something like:
```
(a, b) = (b, a)
```
This becomes the following AST (simplified):

```
AST::Assign {
    pattern: Pattern(Pattern::Tuple [a, b])
    expression: AST(AST::Tuple [b, a])
}
```

The AST is compiled down to a fairly flexible bytecode format.
so the expression would be:

```
Load b
Load a
Tuple 2
-- a tuple is on the top of the stack
-- ???
```

Note that I'm not looking for some optimized solution relating to tuples, just pattern matching in general.

Well, let's think we're essentially doing two assignments, but these assignments have to be atomic.
Each datatype has specific accessors to it.
So what we're doing is essentially a series of checks accesses and assignments.
first, we need to implement an operator to access the tuple:

```
Load b
Load a
Tuple 2
-- a tuple is on the top of the stack
UnTuple 0
-- b is not on top of the stack
```

then we need to save that value:
```
Load b
Load a
Tuple 2
-- a tuple is on the top of the stack
UnTuple 0
-- b is now on top of the stack
Save a
-- b is saved into a
UnTuple 1
-- a is not on top of the stack
Save b
-- a is saved into b
```

this will work.

What about labels?
Labels require a kind.

```
Number n = Number 7.0
```

this becomes the following AST:

```
AST::Assign {
    pattern: Pattern(Pattern::Label(Number, n))
    expression: AST(AST::Label(Number, 7.0))
}
```

so here's the expression bytecode:

```
Con 7.0
Con Kind(Number)
Label
-- label is now on top of stack
-- ???
```

now, the label needs to be unpacked.
I could just add:

```
-- ...
-- label is now on top of stack
UnLabelBad
-- 7.0 is now on top of stack
```

which accesses the data regardless of whether it's correct.
but this is bad. i.e.

```
-- assume this is valid, and un_label is practically UnLabelBad
x = un_label_bad (Number 7.0)
```

this would un-label anything. this is no bueno (right now)

So we'll make it so the kind *has* to be present:

```
-- ...
-- label is now on top of stack
Con Kind(Number)
UnLabel
-- it is checked that the topmost value is the label (Number _)
-- 7.0 is not on top of the stack
```

this is a good solution, let's implement it!

we'll be using this as a simple test:
```
packed_seven = Number 7.0
Number seven = packed_seven

print seven
```

- Patterns need to be passed through to the CST
- when the CST is compiling patterns it's essentially creating a set of destructors and assignments.

Variables need to be declared onto the stack before assignment. this is because variables, when undefined, are saved onto the topmost item. but if there are no new variables declared, one of two bad things could happen:

```
-- this is the stack
a: 7.0
b: 3.0
(7.0, 3.0)
-- we de-structure this tuple into (c, d)
```

load the first element:

```
a: 7.0
b: 3.0
(7.0, 3.0)
7.0
```

then save (this is bad!):

possibility one:
```
a: 7.0
b: 3.0
(7.0, 3.0)
c: 7.0
```

the tuple gets randomly trapped in the region of the stack dedicated to variables.

possibility two:

```
a: 7.0
b: 3.0
c: (7.0, 3.0)
7.0
```

c is declared as a tuple rather than as 7.0

possibility three:

```
a: 7.0
b: 3.0
c: 7.0
```

the tuple is overwritten, making the assignment to d impossible.

**here's what needs to happen**

first, the uninitialized locations are declared:

```
a: 7.0
b: 3.0
c: _
d: _
```

second, the tuple is loaded, the first element is extracted:

```
a: 7.0
b: 3.0
c: _
d: _
(7.0, 3.0)
7.0
```

third, topmost value is saved normally into c:

```
a: 7.0
b: 3.0
c: 7.0
d: _
(7.0, 3.0)
```

fourth, second element in tuple is loaded:

```
a: 7.0
b: 3.0
c: 7.0
d: _
(7.0, 3.0)
3.0
```

fifth, saved into d:

```
a: 7.0
b: 3.0
c: 7.0
d: 3.0
(7.0, 3.0)
```

tuple is popped and replaced by unit:

```
a: 7.0
b: 3.0
c: 7.0
d: 3.0
()
```

this does make some unnecessary copies, but this can be fixed later on.

---

## Addendum: floating expressions:

TODO:
right now, passerine supports breaking on infix operators, i.e.
```
hello =
    "hi"
```
but the infix operator must be on the same line, so the following is not valid:
```
hello
    = "hi"
```
I'd like for the second case to be supported
Additionally, within parenthesis, everything should be treated as a call.
so:
```
(
    foo
    bar
)
```
is the same as:
```
foo bar
```

hello = goodbye

-{
    Tokens:
    Symbol(hello)
    Assign
    Goodbye
}-

hello =
    goodbye

-{
    Tokens:
    Symbol(hello)
    Assign
    Sep
    Goodbye
}-

When we're checking for an infix symbol:

- If the next non-sep symbol is an infix operator, we skip to that operator
- If the next non-sep symbol is not an infix operator, we parse as a call
- After we parse an infix operator, we skip the next Sep

This has been implemented

# TODOs:
- [ ] using macros from other macros; nesting macros
- [ ] variable hoisting
- [ ] pattern error messages
- [X] calls in patterns are ignored?
- [ ] add step to deal with hoisting, etc.
