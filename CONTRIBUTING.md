# Contributing to Passerine
If you're reading this, thanks for taking a look! If you would like to contribute to Passerine, we have some simple rules to follow.

## Code of Conduct
If you find any bugs or would like to request any features, please open an issue. Once the issue is open, feel free to put in a pull request to fix the issue. Small corrections, like fixing spelling errors, or updating broken urls are appreciated. All pull requests must have adequate testing and review to assure proper function before being accepted.

We encourage an open, friendly, and supportive environment around the development of Passerine. If you disagree with someone for any reason, discuss the issue and express you opinions, don't attack the person. Discrimination of any kind against any person is not permitted. If you detract from this project's collaborative environment, you'll be prevented from participating in the future development of this project until you prove you can behave yourself adequately. Please provide arguments based on anecdotes and reasoning to support your suggestions - don't rely on arguments based on 'years of experience,' supposed skill, job title, etc. to get your points across.

# General Guidelines
Readable code with clear behavior works better than illegible optimized code. For things that are very performance-oriented, annotations describing what, how, and why are essential.

Each commit should denote exactly one thing, whether it be a bug fix, or a feature addition. Try not to do both at the same time - it makes code harder to review. Once the codebase is stable, new features should be:

1. First opened as an issue and discussed.
2. Forked and developed in a new branch.
3. Put in as a pull request.
4. Tested and reviewed for bugs, which are fixed.
5. If everything looks good, it will then be merged.

After a while, we plan to implement some sort of RFC process. But, given the small toy-ish status of Passerine, this is unlikely to happen without much support.

Each feature will be given a minor release, which should be tagged. If Passerine garners more popularity, we'll move towards a nightly + rolling release beta. We're also about at the stage where we're looking for core team members. If you're interested, please contribute. When you write well-written long-lasting code (read: lines of code in current release ✕ how long each line has been there), and demonstrate an open, can-do attitude, we'll reach out to you.

## Passerine-Specific Guidelines for Getting Started
> Note: this project is in the rapid initial stages of development, and as such, breaking changes or radical changes in project structure may occur. After the 1.0.0 release, this behavior will stabilize.

Passerine strives to implement a modern compiler pipeline. Passerine is currently broken up into three small projects:

- The core compiler, which resides in this repository.
- The command line interface and the package repository, [Aspen](https://github.com/vrtbl/aspen).

> TODO: write about project structure
> TODO: integration tests
