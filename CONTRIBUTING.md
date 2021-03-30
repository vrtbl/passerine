# Contributing to Passerine
If you're reading this, thanks for taking a look! If you would like to contribute to Passerine, we have some simple rules to follow.

## Code of Conduct
If you find any bugs or would like to request any features, please open an issue. Once the issue is open, feel free to put in a pull request to fix the issue. Small corrections, like fixing spelling errors, or updating broken urls are appreciated. All pull requests must have adequate testing and review to assure proper function before being accepted.

We encourage an open, friendly, and supportive environment around the development of Passerine. If you disagree with someone for any reason, discuss the issue and express you opinions, don't attack the person. Discrimination of any kind against any person is not permitted. If you detract from this project's collaborative environment, you'll be prevented from participating in the future development of this project until you prove you can behave yourself adequately. Please use sound reasoning to support your suggestions - don't rely on arguments based on 'years of experience,' supposed skill, job title, etc. to get your points across.

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

# Integration Tests
If you notice any unsound behavior, like an internal compile error or an incorrect,

1. Reduce the behavior to the minimum required amount of passerine code that causes that error
2. Open an issue explaining what led to the error, what you expected to happen, and the minimum reproducible example.
3. Optionally, add the snippet to `tests/snippets` and test that the test fails (by running `cargo test snippets`).

## What is a test snippet?
A test snippet is some Passerine code that tests a specific outcome. Here's a simple test snippet:

```passerine
-- action: run
-- outcome: success
-- expect: "Banana"

print "Hello, World!"

"Banana"
```

A test snippet starts with a series of comments, each one forming a key-value pair. The `action` is what the compiler should do with the snippet:

- `lex`
- `parse`
- `desugar`
- `hoist`
- `compile`
- `run`

The outcome specifies the specific result:

- No errors are raised: `success`
- A syntax error is raised: `syntax`
- A runtime error is raised: `trace`

Optionally, if the action is `run` an `outcome` may be specified. This treats the snippet like a function body, and compares the returned value with the expected value.

Whenever you add a feature, add snippet tests that demonstrate how this feature should (and should not) work.

## Test Snippet Anvil
If you're working on a new feature and want to work on new test snippets, put them in `tests/snippets_anvil/`, and test them with:

```
cargo test anvil -- --nocapture
```

Files in `snippets_anvil` are `.gitignore`d, so make sure to move them to `snippets` before committing!
