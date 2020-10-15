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
Passerine is still very much so a work in progress.
We've done a lot more, but there's still a lot left.

For those pioneers out there,
The best way to get a feel for Passerine is to install [Aspen](https://github.com/vrtbl/aspen),
Passerine's CLI and package manager.
We're working on an installation script for the passerine development toolchain;
until then, we suggest just cloning this repository and aspen,
and reading through [the documentation](https://docs.rs/passerine).

## Contributing
Contributors welcome!
Read our [Contribution Guidelines](https://github.com/vrtbl/passerine/blob/master/CONTRIBUTING.md)
and join the [Discord server](https://discord.gg/yMhUyhw)
to get started.

## Who's behind this
This is maiden project of The Veritable Computation Foundation.

> TODO: expand...

## Roadmap
See the [Project Roadmap](https://github.com/vrtbl/passerine/projects/1).
