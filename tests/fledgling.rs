///! Snippet tests for the passerine compiler pipeline as a whole.

use std::{
    fs,
    path::PathBuf,
    collections::HashMap,
    rc::Rc,
};

use passerine::*;

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
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Lex,
    Parse,
    Desugar,
    Hoist,
    Gen,
    Run,
}

impl Action {
    pub fn parse(action: &str) -> Action {
        match action {
            l if l == "lex"     => Action::Lex,
            p if p == "parse"   => Action::Parse,
            d if d == "desugar" => Action::Desugar,
            d if d == "hoist"   => Action::Hoist,
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
                    use passerine::construct::ast::AST;
                    let ast: Spanned<AST> = ThinModule::thin(Source::source(result))
                        .lower().expect("Could not lex expectation")
                        .lower().expect("Could not parse expectation")
                        .repr;

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

fn snippet_outcome(source: Rc<Source>, strat: &TestStrat) -> Outcome {
    let outcome = |t| if t { Outcome::Success } else { Outcome::Syntax };
    let module = ThinModule::thin(source);

    let tokens = module.lower();
    if strat.action == Action::Lex     { return outcome(  tokens.is_ok()); }
    let ast = tokens.and_then(Lower::lower);
    if strat.action == Action::Parse   { return outcome(     ast.is_ok()); }
    let cst = ast.and_then(Lower::lower);
    if strat.action == Action::Desugar { return outcome(     cst.is_ok()); }
    let sst = cst.and_then(Lower::lower);
    if strat.action == Action::Hoist   { return outcome(     sst.is_ok()); }
    let bytecode = sst.and_then(Lower::lower);
    if strat.action == Action::Gen     { return outcome(bytecode.is_ok()); }

    let lambda = match bytecode {
        Ok(l) => l,
        Err(_) => return Outcome::Syntax,
    };

    let mut vm = VM::init(Closure::wrap(lambda));

    let run_outcome = match vm.run() {
        Ok(()) => {
            if let Some(expected) = &strat.expect {
                let top = vm.stack.pop_data();
                if expected != &top {
                    println!("Top: {}", top);
                    println!("Expected: {}", expected);
                    panic!("Top stack data does not match")
                }
            }
            return Outcome::Success;
        },
        Err(_) => Outcome::Trace,
    };

    return run_outcome;
}

fn test_snippet(source: Rc<Source>, strat: &TestStrat) {
    let outcome = snippet_outcome(source, strat);
    if outcome != strat.outcome {
        println!("expected outcome {:?}", strat.outcome);
        println!("actual outcome {:?}", outcome);
        panic!("test failed, outcomes are not the same");
    }
}

fn snippets(dir: &str) {
    let paths = fs::read_dir(dir)
        .expect("You must be in the base passerine directory, snippets in ./tests/snippets");

    let mut to_run: Vec<PathBuf> = vec![];
    for path in paths { to_run.push(path.expect("Could not read path").path()) }

    let mut counter = 0;
    println!("\nRunning {} snippet test(s)...", to_run.len());

    // TODO: subdirectories of tests
    while let Some(path) = to_run.pop() {
        println!("test {}: {}...", counter, path.display());

        let source = Source::path(&path).expect("Could not get snippet source");
        let test_strat = TestStrat::snippet(&source);

        test_snippet(source, &test_strat);
        counter += 1;
    }

    println!("All tests passed!\n");
}

#[test]
fn test_snippets() {
    snippets("./tests/snippets")
}
