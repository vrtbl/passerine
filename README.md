<p align="center">
    <a href="https://passerine.io">
        <img src="https://raw.githubusercontent.com/vrtbl/passerine/master/Logotype.png">
    </a>
    <br>
    <a href="https://github.com/vrtbl/passerine/actions">
        <img src="https://github.com/vrtbl/passerine/workflows/Rust/badge.svg">
    </a>
    <a href="https://crates.io/crates/passerine">
        <img src="https://img.shields.io/crates/v/passerine.svg">
    </a>
    <a href="https://docs.rs/passerine">
        <img src="https://docs.rs/passerine/badge.svg">
    </a>
    <a href="https://discord.gg/yMhUyhw">
        <img src="https://img.shields.io/discord/651996477333438474?logo=discord">
    </a>
    <br>
</p>

**Welcome to Passerine!**
Passerine is a small, concise, extensible programming language, powered by a VM written in Rust.
It was inspired by Scheme, OCaml, Rust, and Wren.
Here's a small taste:

```
syntax 'for var 'in list do {
    body = var -> do

    loop = remaining -> {
        body remaining.head
        match remaining.tail -> [
            Some more -> loop more,
            None      -> (),
        ]
    }

    loop list
}

for number in [1, 2, 3].reverse {
    print (number + "...")
}

print "Liftoff!"
```

## Getting Started
If you just want to see what Passerine can do:

> NOTE: Passerine is in an early stage of development.
It's just barely reached the stage of theoretically being Turing-complete,
but don't @ me yet.

1. Clone this git repository.
2. Build Passerine with cargo.
3. Run the tests.

If you'd like to contribute:

1. Get Passerine.
2. Start developing.
   `CONTRIBUTING.md` has general guidelines and discusses project structure

## Roadmap
See the [Project Roadmap](https://github.com/vrtbl/passerine/projects/1).
