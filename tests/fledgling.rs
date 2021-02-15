///! Snippet tests for the passerine compiler pipeline as a whole.

use std::{
    fs,
    path::PathBuf,
    collections::HashMap,
    rc::Rc,
};

use passerine::{
    common::{
        source::Source,
        data::Data,
        closure::Closure,
    },
    compiler::{
        lex, parse, desugar, gen,
        ast::AST,
    },
    vm::vm::VM,
};

/// Represents specific success/failure modes of a snippet test.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Syntax,
    Trace,
}

impl Outcome {
    pub fn parse(outcome: &str) -> Outcome {
        match outcome {
            s if s == "success" => Outcome::Success,
            s if s == "syntax"  => Outcome::Syntax,
            t if t == "trace"   => Outcome::Trace,
            invalid => {
                println!("invalid: '{}'", invalid);
                panic!("invalid outcome in strat heading");
            },
        }
    }
}

/// Represents what part of the compiler a snippet tests.
#[derive(Debug)]
pub enum Action {
    Lex,
    Parse,
    Desugar,
    Gen,
    Run,
}

impl Action {
    pub fn parse(action: &str) -> Action {
        match action {
            l if l == "lex"     => Action::Lex,
            p if p == "parse"   => Action::Parse,
            d if d == "desugar" => Action::Desugar,
            g if g == "gen"     => Action::Gen,
            r if r == "run"     => Action::Run,
            invalid => {
                println!("invalid: '{}'", invalid);
                panic!("invalid action in strat heading");
            },
        }
    }
}

/// Represents a test strategy for executing a snippet,
/// Found at the top of each file.
#[derive(Debug)]
pub struct TestStrat {
    /// How to run the test.
    action:  Action,
    /// The expected outcome.
    outcome: Outcome,
    /// Optional data to check against.
    /// Should only be used with Action::Run
    expect:  Option<Data>
}

impl TestStrat {
    /// Uses a heading to construct a test strat
    pub fn heading(heading: HashMap<String, String>) -> TestStrat {
        let mut outcome = None;
        let mut action = None;
        let mut expect = None;

        for (strat, result) in heading.iter() {
            match strat {
                o if o == "outcome" => outcome = Some(Outcome::parse(result)),
                a if a == "action"  => action  = Some(Action::parse(result)),
                e if e == "expect"  => expect  = {
                    let tokens = lex(Source::source(result)).expect("Could not lex expectation");
                    let ast    = parse(tokens).expect("Could not parse expectation");

                    if let AST::Block(b) = ast.item {
                        if let AST::Data(d) = &b[0].item {
                            Some(d.clone())
                        } else { panic!("expected data in block") }
                    } else { panic!("expected block in ast") }
                },
                invalid => {
                    println!("invalid: '{}'", invalid);
                    panic!("invalid strat in strat heading");
                },
            }
        }

        TestStrat {
            outcome: outcome.expect("no outcome provided"),
            action: action.expect("no action provided"),
            expect,
        }
    }

    /// Parses the Test Strat from a given snippet.
    pub fn snippet(source: &Rc<Source>) -> TestStrat {
        let mut heading = HashMap::new();
        let lines = source.contents.lines();

        // build up a list of key-value pairs
        for line in lines {
            if line.len() <= 2 || &line[0..2] != "--" { break };

            let spliced = line[2..].trim().split(":").collect::<Vec<&str>>();
            if spliced.len() <= 1 { panic!("Missing colon in test strat heading") }

            let strat = spliced[0];
            let result = spliced[1..].join(":");
            if heading.insert(strat.trim().to_string(), result.trim().to_string()).is_some() {
                panic!("Key present twice in test strat heading");
            }
        }

        return TestStrat::heading(heading);
    }
}

fn test_snippet(source: Rc<Source>, strat: TestStrat) {
    let actual_outcome: Outcome = match strat.action {
        Action::Lex => if lex(source)
            .is_ok() { Outcome::Success } else { Outcome::Syntax },

        Action::Parse => if lex(source)
            .and_then(parse)
            .is_ok() { Outcome::Success } else { Outcome::Syntax },

        Action::Desugar => if lex(source)
            .and_then(parse)
            .and_then(desugar)
            .is_ok() { Outcome::Success } else { Outcome::Syntax },

        Action::Gen => if lex(source)
            .and_then(parse)
            .and_then(desugar)
            .and_then(gen)
            .is_ok() { Outcome::Success } else { Outcome::Syntax },

        Action::Run => {
            match lex(source)
                .and_then(parse)
                .and_then(desugar)
                .and_then(gen)
            {
                Ok(lambda) => {
                    let mut vm = VM::init(Closure::wrap(lambda));

                    match vm.run() {
                        Ok(()) => {
                            if let Some(expected) = &strat.expect {
                                let top = vm.stack.pop_data();
                                if expected != &top {
                                    println!("Top: {}", top);
                                    println!("Expected: {}", expected);
                                    panic!("Top stack data does not match")
                                }
                            }
                            Outcome::Success
                        },
                        Err(_) => Outcome::Trace
                    }
                }
                Err(e) => {
                    println!("{}", e);
                    Outcome::Syntax
                }
            }
        }
    };

    if actual_outcome != strat.outcome {
        println!("expected outcome {:?}", strat.outcome);
        println!("actual outcome {:?}", actual_outcome);
        panic!("test failed, outcomes are not the same");
    }
}

#[test]
fn test_snippets() {
    let paths = fs::read_dir("./tests/snippets")
        .expect("You must be in the base passerine directory, snippets in ./tests/snippets");

    let mut to_run: Vec<PathBuf> = vec![];
    for path in paths { to_run.push(path.expect("Could not read path").path()) }

    let mut counter = 0;
    println!("\nRunning {} snippet test(s)...", to_run.len());

    // TODO: subdirectories of tests
    while let Some(path) = to_run.pop() {
        println!("test {}: {}...", counter, path.display());

        let source = Source::path(path).expect("Could not get snippet source");
        let test_strat = TestStrat::snippet(&source);

        test_snippet(source, test_strat);
        counter += 1;
    }

    println!("All tests passed!\n");
}
