<p align="center">
    <a href="https://passerine.io">
        <img src="https://raw.githubusercontent.com/vrtbl/passerine/master/Logotype.png">
    </a>
</p>
<h3 align="center">The Passerine Programming Language</h3>
<p align="center">Made with ♡ by Isaac Clayton and the Passerine Community — a cornerstone of the Veritable Computation Initiative.</p>
<p align="center">
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
    <br>
</p>

**Welcome to Passerine!**
Passerine is a small, concise, extensible programming language,
powered by a VM written in Rust.

Here's a small taste:

```plain
-- a syntactic macro that introduces a scoped let construct
syntax 'let variable 'be vaule 'in body {
    -- using the classical let-as-evaluated-lambda trick
    -- `<p> -> <e>` defines a function
    -- `<l> <e>` calls a function
    (variable -> body) value
}

-- `<p> = <e>` is assignment
x = false

-- using the let macro defined above
let x be true in {
    -- prints `true`
    print x
}

-- outer scope not affected
-- prints `false`
print x
```

## Getting Started
Passerine is still very much so a work in progress.
We've done a lot, but there's still a so much more to do!

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

### Who started this?
This is first project of The Veritable Computation Initiative.
Our goal is to improve the tools developers use to write software.

So far, all of Passerine has been developed by
[Isaac Clayton](https://github.com/slightknack).

## Roadmap
See the [Project Roadmap](https://github.com/vrtbl/passerine/projects/1) to get a feel for what's currently under development.
