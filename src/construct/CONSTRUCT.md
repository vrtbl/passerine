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

if (grouping is present) {
    this is a single form split
} else {
    across multiple lines
}

(
    single
    form
    multiple
    lines
)
```

## Leafs


Label
Iden
Op
Lit