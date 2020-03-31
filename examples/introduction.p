-{
    # A complete overview of Passerine, in Passerine
    Passerine intends to be a fast, extensible, modern programming language.
    Passerine is structurally typed, and has support for traits and the like.
}-

-{
    ## Part I: Datatypes
    Passerine supports all the standard datatypes you'd expect.
    First off, the atomic datatypes:
}-

-- Numeric Tower
69    -- Naturals
-420  -- Integers
2/3   -- Rationals
6.25  -- Reals
2+3i  -- Imaginaries

-- Booleans
true
false -- all values but true are false

-- Strings
"Hello, World!" -- Strings
"Really? 😮"    -- Unicode Support
"\b00\bFF"      -- Insert bytes

-{
    Passerine also supports algebraic datatypes, such as:
}-

-- Tuples
(1, "Hello", true) -- sized collections of different datatypes

-- Lists
[1, 2, 3, 4, 5] -- variable length collections of the same datatype

-- Unions
2D {x, y} ~ 3D {x, y, z} ~ ND [d..] -- Tagged enumerations of different datatypes -- Tagged yay/nay?

-- Structures
{x, y, z} -- Named fields that hold values

-- Maps
{"Heck": true, 0: (1, False)} -- Dictionaries, a catch-all hodgepodge -- Include?

-{
    ## PART II: Patterns
    A pattern de-structures a datatype into bindings.
    Patterns mirror the datatypes they de-structure, but with symbols capturing the values in the corresponding places.
    `..` and `_` are used to collect and/or discard extra leftover values.
}-

-- For example, the pattern:
{x: (a, b, [h, t..]), ..}

-- matches
{y: 0, x: (1, "Hello", [true, false, true])}

-- and would produce the bindings:
a = 1
b = "Hello"
h = true
t = [false, true]

-- Additionally, patterns can contain guards: conditions that must be met for the pattern to match.
-- For example, the pattern:
x | x > 0

-- matches all values `x` where `x` is greater than `0`.
-- To check multiple conditions, use `and`.
(x, y) | x > 0 and y > 0

-{
    ## Part III: Assignment
    An assignment produces a binding in the current scope, or updates an older binding by the same name.
    A binding maps a name (symbol) to a value - it's a variable.
}-

-- An assignment takes the form:
p = e

-- where p is a pattern and e is an expression.
-{
    To evaluate an assignment:
    1. Evaluate expression `e` into value `v`.
    2. Match value `v` against the pattern `p`, producing a set of bindings `b`.
    3. For each binding in `b`, update old binding in current scope or produce new one.
    4. Evaluates to Unit: `()`.
}-

-- For example:
(a, b) = (1 + 2, 3 * 5)

-- 1. Evaluate `e`:
(a, b) = (3, 15)

-- 2. Produce bindings:
a = 3
b = 15

-- 3. Update...

-- 4. Evaluate to unit:
()

-{
    # Part IV: Lambda
    A lambda expression produces an anonymous function that transforms one value into another.
    Lambda expressions are used to create closures, and in combination with assignment, named functions.
}-

-- A lambda expression takes the form:
p -> e0

-- Where `p` is a pattern, and `e` is an expression, and produces a new anonymous function.
-- To evaluate an anonymous function, given:
(p -> e0) e1

-- Where `(p -> e0)` is an anonymous function, and `e1` is an expression:
-- 1. Evaluate `e1` to the value `v0`.
-- 2. Match value `v0` against the pattern `p`, producing a set of bindings `b`.
-- 3. Evaluate the expression `e0` in a new scope with bindings `b`, producing a new value `v1`.
-- 4. Evaluates to `v1`.

-{
    # Part IV: Blocks
    Blocks evaluate multiple expressions in a row, and take on the value of the last expression.
}-

-- For instance:

{
    x = 5
    y = x + 2
    x * y + 2
}

-- would evaluate to:
37

-- Blocks are very useful in combination with functions:

perms = seq -> {
    match (len sequence) [
        x | x <= 1 -> yield seq
        x -> {
            [head, tail..] = seq
            for perm (perms tail) {
                for i 0..((len perm)) {
                    yield perm[..i] + head + perm[i..]
                }
            }
        }
    ]
}


# Macros
# Fibers
