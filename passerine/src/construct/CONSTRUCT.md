# Constructs

The token tree.

## Grouping 

These tokens must be balanced at read time.

### Block
A list of forms between curlies `{ ... }`, separated by `SEP` (semicolons and/or newlines):

```
{
    this is a form
    this is (also a) form
    this is a +
        single form
    last form
}
```

### List
A single form between squares `[ ... ]`, potentially containing the `PAIR` operator `,`:

```
[1, 2, 4, 7]
```

Converts the normally-parsed tuple into a homogenous growable list.

### Form
A list of tokens, potentially split across multiple lines. Usually inferred, but optionally grouped in parenthesis `( ... )`.

```
this is a single line form

this form + has some - operators

split
    + across multiple
    - lines

postfix works - 
    as well

if (grouping is + present) {
    this if expression is a single form
} else {
    split across multiple lines
    with nested sub blocks
}

(
    single
    form
    multiple
    lines
)
```

## Leafs

Atoms that are not nested.

### Label
An identifier representing a type, must start with a capital letter. `UpperCamelCase` by convention.

```
X
String
HtmlRenderer
```

### Iden
An identifier, containing alphanumeric characters and underscores, not starting with numbers. `snake_case` by convention:

```
_
this
banana_pants
hello7
```

### Op
An operator, a series of ascii punctuation characters not reserved for other constructs, like grouping:

```
+
-
++
!=
&&&&&&&&&&&
*&^
```

### Lit
A literal, such as a string, a number, etc.

```
3
27.5
0xFF
"Hello"
()
```