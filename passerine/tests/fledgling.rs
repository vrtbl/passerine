///! Snippet tests for the passerine compiler pipeline as a whole.
use std::{collections::HashMap, fs, path::PathBuf, rc::Rc};

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
            s if s == "syntax" => Outcome::Syntax,
            t if t == "trace" => Outcome::Trace,
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
    Read,
    Desugar,
    Hoist,
    Gen,
    Run,
}

impl Action {
    pub fn parse(action: &str) -> Action {
        match action {
            l if l == "lex" => Action::Lex,
            p if p == "parse" => Action::Parse,
            r if r == "read" => Action::Read,
            d if d == "desugar" => Action::Desugar,
            d if d == "hoist" => Action::Hoist,
            g if g == "gen" => Action::Gen,
            r if r == "run" => Action::Gen, // TODO: actually run code!
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
    action: Action,
    /// The expected outcome.
    outcome: Outcome,
    /// Optional data to check against.
    /// Should only be used with Action::Run
    expect: Option<Data>,
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
                a if a == "action" => action = Some(Action::parse(result)),
                e if e == "expect" => {
                    // TODO: implement expectations.
                    // println!("Warning, expectations are not implemented");
                    // expect =
                    // {
                    //     let (ast, _symbols) =
                    //         compiler::parse(Source::source(result))
                    //             .expect("Could not parse result field");

                    //     use construct::tree::{
                    //         Base,
                    //         AST,
                    //     };

                    //     if let AST::Base(Base::Block(ref b)) = ast.item {
                    //         if let AST::Base(Base::Lit(ref l)) =
                    //             b.get(0).expect("Expected an
                    // expression").item         {
                    //             assert_eq!(l, )
                    //         }
                    //     }

                    //     dbg!(ast);
                    // }
                    // todo!();
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
            if line.len() <= 1 || &line[0..1] != "#" {
                break;
            };

            let spliced = line[1..].trim().split(":").collect::<Vec<&str>>();
            if spliced.len() <= 1 {
                panic!("Missing colon in test strat heading")
            }

            let strat = spliced[0];
            let result = spliced[1..].join(":");
            if heading
                .insert(strat.trim().to_string(), result.trim().to_string())
                .is_some()
            {
                panic!("Key present twice in test strat heading");
            }
        }

        return TestStrat::heading(heading);
    }
}

fn outcome<T>(t: Result<T, Syntax>) -> Outcome {
    if let Err(e) = t {
        eprintln!("{}", e);
        Outcome::Syntax
    } else {
        Outcome::Success
    }
}

fn snippet_outcome(source: Rc<Source>, strat: &TestStrat) -> Outcome {
    let result = match strat.action {
        Action::Lex => return outcome(compiler::lex(source)),
        Action::Parse => return outcome(compiler::parse(source)),
        Action::Read => return outcome(compiler::read(source)),
        Action::Desugar => return outcome(compiler::desugar(source)),
        Action::Hoist => return outcome(compiler::hoist(source)),
        Action::Gen => return outcome(compiler::gen(source)),
        Action::Run => compiler::gen(source),
    };

    let lambda = match result {
        Ok(l) => l,
        Err(_) => return Outcome::Syntax,
    };

    println!("{}", &lambda);
    println!(
        "{}",
        if let Data::Lambda(s) = &lambda.constants[0] {
            s
        } else {
            todo!()
        }
    );

    let mut Fiber = Fiber::init(Closure::wrap(lambda));

    let run_outcome = match Fiber.run() {
        Ok(()) => {
            if let Some(expected) = &strat.expect {
                let top = Fiber.stack.pop_data();
                if expected != &top {
                    println!("Top: {}", top);
                    println!("Expected: {}", expected);
                    panic!("Top stack data does not match")
                }
            }
            return Outcome::Success;
        },
        Err(e) => {
            eprintln!("{}", e);
            Outcome::Trace
        },
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
    let paths = fs::read_dir(dir).expect(
        "You must be in the base passerine directory, snippets in ./tests/snippets",
    );

    let mut to_run: Vec<PathBuf> = vec![];
    for path in paths {
        to_run.push(path.expect("Could not read path").path())
    }

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
