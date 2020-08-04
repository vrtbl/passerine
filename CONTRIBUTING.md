# Contributing to Passerine
If you're reading this, thanks for taking a look!
If you would like to contribute to Passerine, we have some simple rules to follow.
Contributions are made through pull requests, unless you are given write access to the repository.\*

## Code of Conduct
If you find any bugs or would like to request any features, please open an issue.
Once the issue is open, feel free to put in a pull request to fix the issue.
Small corrections, like fixing spelling errors, or updating broken urls are appreciated.
(Note that feature requests, even if implemented, might not be accepted if they don't align with the goals of the project.)

All pull requests must have adequate testing and peer-review to assure proper function before being accepted.
If your code does not compile, or introduces other obvious bugs, your pull request will not be accepted.
More likely than not, it will take more that one commit to get a pull request in.
Don't get discouraged if your fist solution doesn't solve the problem, try again!

We encourage an open, friendly, and supportive environment around the development of Passerine.
If you disagree with someone for any reason, discuss the issue and express you opinions, don't attack the person.
Discrimination of any kind against any person is not permitted.
If you detract from this project's collaborative environment, you'll be prevented from participating in the future development of this project until you prove you can behave yourself adequately.
Please provide arguments based on anecdotes and reasoning to support your suggestions - don't rely on arguments based on 'years of experience,' supposed skill, job title, etc. to get your points across.

# General Guidelines
Readable code with clear behavior works better than illegible optimized code.
For things that are very performance-oriented, annotations describing what, how, and why are essential.

Each commit should denote exactly one thing, whether it be a bug fix, or a feature addition.
Try not to do both at the same time - it makes code harder to review.
Once the codebase is stable, new features should be:

1. First opened as an issue and discussed.
2. Forked and developed in a new branch.
3. Put in as a pull request.
4. Tested and reviewed for bugs, which are fixed.
5. If everything looks good, it will then be merged.

After a while, we plan to implement some sort of RFC process.
But, given the small toy-ish status of Passerine, this is unlikely to happen without much support.

Each feature will be given a minor release, which should be tagged.
If Passerine garners more popularity, we'll move towards a nightly + rolling release beta.
We're also about at the stage where we're looking for core team members.
If you're interested, please contribute.
When you write well-written long-lasting code (read: lines of code in current release ✕ how long each line has been there), and demonstrate an open, can-do attitude, we'll reach out to you.

## Passerine-Specific Guidelines for Getting Started
> Note: this project is in the rapid initial stages of development, and as such, breaking changes or radical changes in project structure may occur. After the 1.0.0 release, this behavior will stabilize.

Passerine strives to implement a modern compiler pipeline. 
Passerine is currently broken up into three small projects: 

- The core compiler, which resides in this repository.
- The command line interface and the package repository, [Aspen](https://github.com/vrtbl/aspen).

### Core Compiler
The core compiler is made of a series of 'pipes,' which transform one type of data into another.
Programs start as source files, after which they are then lexed into tokens, parsed into ASTs, compiled to bytecode, then run on the VM.

The pipes themselves (i.e., the lexer, the parser, and the bytecode generator), can be found in `src/compiler`.
The datastructures associated with these pipes can be found in `src/common`.

Note that pipes can only reference datastructures in pipeline - the pipeline should never reference anything found in pipes.
Additionally, pipes should *not* rely on other pipes to get their job done.
We follow the philosophy of Unix\*\*: Each pipe should do one thing, and do it well.

Both the VM and Compiler can, however, rely on datastructures in `src/common`.
For instance, all datastructures are annotated with `Spans`, which point to which part of the source file each each structure represents.
This includes utilities for printing errors, building the Passerine standard library, annotating source files, and passing data through multiple pipes sequentially, and so on.

The powerhouse of this operation, the VM and its associated datastructures, can be found in `src/VM`. The VM is expected to only rely on bytecode and a constants table. Pipes should not to rely on the VM.

As for testing, each non-trivial file is expected to have unit tests at the bottom of it.
As a rule of thumb, at least a third of a file should be dedicated to testing.

The easiest way to contribute is to find errors and report them.
If you run into an error in the compiler itself, it would be appreciated if you could go the extra mile and write some test cases that show the error.

#### Adding Features
Due to the structure of this project, it's fairly easy to implement new features.
If you've got a new feature in mind, roughly follow these steps:

1. Fork the repo, start a new branch.
2. If the feature requires a syntactic change to the language, edit `grammar.ebnf`.
3. Implement lexing for any new tokens introduced.
4. Implement parsing to lex those tokens into the correct AST.
5. Determine the behavior of the feature - if it requires a new opcode, implement said opcode.
6. Implement bytecode generation.
7. Run all tests - if passing, bump version, submit a PR.

Remember to write adequate tests during each step where applicable.
A brilliant feature with 0 tests is like a Saturn V built with duct-tape.
Impressive, but bound to explode.

### CLI and Package Repository
> NOTE: this phase of the project has not wholly started yet.

The VM should be manipulated solely through the utilities provided by core compiler.

Passerine supports modules (local, un-versioned packages) and packages. All packages and their versions to be used by a Passerine project are defined in a configuration file in the project's root.

## Footnotes
> \* This is a great responsibility. Even if you have write access, it is suggested you always open a pull request. If you're patching an urgent non-breaking-change, pushing to master is acceptable.

> \*\* However, we do not believe everything is a file.
