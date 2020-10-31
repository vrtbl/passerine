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
It's taken inspiration from many languages,
including Scheme, OCaml, Rust, and Wren.
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

So far, all of Passerine has been developed by a
[one-man team](https://github.com/slightknack).

> It's taken quite a lot of effort to get this far.
> I've been thinking about this language for around 3 years now,
> and have been actively developing it for around the past year.
> I'm excited to continue development -
> this is largely a test of personal effort,
> which is why I've been developing it regardless of whether others
> see its value at the moment.
> I'm just a high-school student, so any interest
> means a lot to me.
> Thanks for checking out my work!
>
> — Isaac Clayton, the man of the one-man team

## Roadmap
See the [Project Roadmap](https://github.com/vrtbl/passerine/projects/1) to get a feel for what's currently under development.
