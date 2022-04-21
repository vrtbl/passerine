# aspen
Passerine's package manager.

## Getting Started
To install the `aspen` command, run this in the shell of your choice:

```zsh
bash <(curl -sSf https://www.passerine.io/install.sh)
```

This requires git and a recent version of Cargo to work.
You can inspect the contents of `install.sh` first if you want,
we're not trying to play any tricks on you.

2. Start a new Passerine project using Aspen:
```bash
aspen new example
```
3. This will create a project named `example` in the current directory.
   Open this project and look through it in your editor of choice.
4. To run the project:
```bash
cd example
aspen run
```

> `aspen run` and most other commands optionally take a path to the project root.

### Commands

> NOTE: Not all commands are implemented ATM.
> Commands in **bold** are partially or wholly implemented.

| Command    | Result                                                    |
| ---------- | --------------------------------------------------------- |
| `update`   | Updates the Passerine toolchain.                          |
| **`new`**  | Creates a new Passerine package.                          |
| `publish`  | Publishes package to the registries in `Aspen.toml`.      |
| `pull`     | Pulls fresh packages from the registries in `Aspen.toml`. |
| `add`      | Adds a dependency to `Aspen.toml`.                        |
| **`run`**  | Builds and runs the corresponding Passerine package.      |
| **`repl`** | Opens a fresh repl session.                               |
| `test`     | Builds and runs the package's tests.                      |
| `bench`    | Builds and runs the package's benchmarks.                 |
| `doc`      | Builds the package's documentation.                       |
| `debug`    | Builds and runs the package in interactive debug mode.    |

An optional path to the project root may be provided.
