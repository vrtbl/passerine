what is wrong?
um, so...

here's the deal, we have this code:

```
w = "hello"

loop = ()
loop = y -> {
    x -> {
        y w
        loop y ()
    }
}

loop (x -> print x) ()
```

it's supposed to print "hello" forever.
but it doesn't

why?

well, w is a captured variable

first, loop inner looks to capture some variables - here's the ast:

```
Block [
    Call { y, w }
    Call { Call { loop, y } () }
]
```

and here's the order it's traversed in:

```
Block
Call
w
y
Call
()
Call
y
loop
```

and here's the order the variables are traversed in:

```
w
y
y -- duplicate
loop
```

and here is the depth of each variable:

```
-- innermost depth is 2
w: 0
y: 1
y: 1 -- duplicate
loop: 0
```

what does this mean?

each scope has a list of variables it captures.

```
0: c[]
1: c[]
2: c[]
```

each scope also refers to the local variables it captures:
each these arrays refers to positions in the array before it

```
0:
1: u[]
2: u[]
```

let's walk through this, together:

```
w
y
y -- duplicate
loop
```

first, w is seen in scope 2, it is added to 0s captured array:

```
0: c[w, ]
1: c[]
2: c[]
---
0:
1: u[]
2: u[]
```

and propogated upwards:

```
0: c[w, ]
1: c[w, ]
2: c[w, ]
---
0:
1: u[w, ]
2: u[w, ]
```

then, y is seen in scope 1 and propogated upwards:

```
0: c[w, ]
1: c[w, y, ]
2: c[w, y, ]
---
0:
1: u[w, y, ]
2: u[w, y, ]
```

then y is seen again in scope 2 - it already exists, so nothing changes.

then, loop is seen in scope two and found in scope 1 then propogated upwards:

```
0: c[w, loop]
1: c[w, y, loop]
2: c[w, y, loop]
---
0:
1: u[w, loop]
2: u[w, y, loop]
```

here's the issue - let's look at layer 1:

```
1: c[w, y, loop]
1: u[w, loop]
```

the upvalue array defines values that are captured in enclosing scopes
this is correct.

So, here w and loop are both captured when the lambda is constructed.

```
1: c[w, y, loop]
1: u[w, loop]
1: r[w, loop]
```

y is then added in later when the capture instruction is encountered:

```
1: c[w, y, loop]
1: u[w, loop]
1: r[w, loop, y]
```

note that the compiled code expects `c`, but the running code will have `r`.

this will effectively swap the position of y and loop, in this case:

```
1: c[w, y, loop]
1: r[w, loop, y]
```

this is bad because y and loop are *not* in any regard the same.

so, how do we fix this?

well, the first solution that comes to mind it to preallocate values we know to be empty.
in essence, when the closure is encountered, we do this:

```
1: c[w, y, loop]
1: u[w, loop]
1: r[w, _, loop]
```

and then when y is encountered, we slot it in:

```
1: c[w, y, loop]
1: u[w, loop]
1: r[w, y, loop]
```

this way, c and r are equal:

```
1: c[w, y, loop]
1: r[w, y, loop]
```

the next solution is to push all nonlocal variables onto c before the locals, like so:

```
1: u[w, loop]
1: n[y]
1: c[w, loop, y]
```

then, when the code is loaded into the vm, the order of the locals will reflect the order they are captured.

but there is there a simpler solution?

why are we even capturing locals in the first place?

well, when locals are referenced from a non-local scope, they need to be put on the heap for reasons I won't get into here.

each stack frame essentially has a list of captured variables.

this is what r is.

when a new scope is entered, i.e. a new closure is constructed, it needs to reference some captured variables.

the way we currently do this is by moving all local variables into stack frames, then builting a chain of references through stack frames until we reach the scope that uses it.

in essence, the upvalue array points at the captures array of the scope above it:

```
0:
0: c[w, loop] *
     ^  ^
     |  |
1: u[w, loop]
     ^    ^
     |     \
1: c[w, y, loop] *
     ^  ^  ^
     |  |  |
2: u[w, y, loop]
     ^  ^  ^
     |  |  |
2: c[w, y, loop] *
```

the reason we do this is to prevent redundant heaping from closures declared in the same scope.

so what other solutions exist?

none that I can think of.

so let's go with second solution, this one:

> the next solution is to push all nonlocal variables onto c before the locals, like so:
>
> ```
> 1: u[w, loop]
> 1: n[y]
> 1: c[w, loop, y]
> ```
>
> then, when the code is loaded into the vm, the order of the locals will reflect the order they are captured.

under this solution, no bent paths can exist.
so our above diagram becomes this:

```
0:
0: c[w, loop] *
     ^  ^
     |  |
1: u[w, loop]
     ^  ^
     |  |
1: c[w, loop, y] *
     ^  ^     ^
     |  |     |
2: u[w, loop, y]
     ^  ^     ^
     |  |     |
2: c[w, loop, y] *

```

we'll think of a reasonable solution to this in a bit.

*One dinner later*

I think I've got it.

When we walk the tree, we have no idea when we'll know which variables are captured.

essentially, within each scope, we have captured locals and captured non-locals.

let's make this division apparent:

```
0:
0: n[] l[w, loop]
         ^  ^
         |  |
1:     u[w, loop]
         ^  ^
         |  |
1:     n[w, loop] l[y]
         ^  ^       ^
         |  |       |
2:     u[w, loop,   y]
         ^  ^       ^
         |  |       |
2:     n[w, loop,   y] l[]
```

so c becomes n and n.

> c is compiler.captures
> u is compiler.lambda.upvalues
>
> we'll change c to
> compiler.cap_locals
> and
> compiler.cap_non_locals

note that ordered r is just n + l

how is this built up? let's try running through the compiler.

```
w
y
y -- duplicate
loop
```

starting with:


```
0:
0: l[]
1: u[]
1: n[] l[]
2: u[]
2: n[] l[]
```

first, w is seen, and is local in layer 0:

```
0:
0: l[w]
1: u[]
1: n[] l[]
2: u[]
2: n[] l[]
```

this is then propagated up the layers:

```
0:
0: l[w]
1: u[w]
1: n[w] l[]
2: u[w]
2: n[w] l[]
```

then, y is seen and found to the local in the second scope:

```
0:
0: l[w]
1: u[w]
1: n[w] l[y]
2: u[w]
2: n[w] l[]
```

at this point we don't know where y will be on the second layer upvalue.

so what do we do?

we have three options, all of them involve touching the upvalue array in some way.

we can make the upvalue array contain two distinct types, which are then flattened:

> ```
> 0:
> 0: l[w, loop]
> 1: u[l_w, l_loop]
> 1: n[w, loop] l[y]
> 2: u[n_w, l_y, n_loop]
> 2: n[w, loop, y] l[]
> ```
> flattened:
> 0:
> 0: l[w, loop]
> 1: u[w, loop]
> 1: n[w, loop] l[y]
> 2: u[w, loop, y]
> 2: n[w, loop, y] l[]
> ```

wait.
let's return to something earlier.

> the reason we do this is to prevent redundant heaping from closures declared in the same scope.
>
> so what other solutions exist?

what if we just keep track of what locals have been heaped.
so if we need to access them again, we just don't emit the opcode::heap instruction.

and when we need to capture some upvalues, instead of referring to the previous frame's upvalue scope,
we refer to the positions of the locals in the current scope,
and the positions of the nonlocals in the upvalues scope.

Here's what I'm getting at.

```
0: l[] n[]
1: l[w, loop] n[]
2: l[y] n[w, loop]
```

that's a simpler solution. let's implement it.
