// TODO: preserve values using module system.

// use std::io::{self, Read};
// use std::io::Write;
//
// use passerine::{
//     common::{closure::Closure, source::Source, lambda::Lambda},
//     compiler::{lex, parse, desugar, gen, hoist, syntax::Syntax},
//     vm::vm::VM,
// };

pub fn repl() {
    println!("Repl is WIP.");
    return;

    // println!("Hit ^C to quit.\n");

    // loop {
    //     let mut to_eval = String::new();
    //
    //     let lambda: Result<Lambda, Syntax> = 'read: loop {
    //         print!("| ");
    //         std::io::stdout().flush().unwrap();
    //
    //         let mut exit = false;
    //         match io::stdin().read_line(&mut to_eval) {
    //             Ok(l) if l <= 1 => { exit = true; },
    //             _ => ()
    //         }
    //
    //         let source = Source::source(&to_eval);
    //         let compiled = lex(source)
    //             .and_then(parse)
    //             .and_then(desugar)
    //             .and_then(hoist)
    //             .and_then(gen);
    //
    //         match compiled {
    //             Ok(lambda) => { break 'read Ok(lambda); },
    //             Err(e)     => if exit { break 'read Err(e); },
    //         };
    //     };
    //
    //     let lambda = match lambda {
    //         Ok(l)  => l,
    //         Err(e) => { println!("\n{}\n", e); continue; },
    //     };
    //
    //     let mut vm = VM::init(Closure::wrap(lambda));
    //     match vm.run() {
    //         Ok(()) => {
    //             let data = vm.stack.pop_data();
    //             println!("= {}\n", data)
    //         },
    //         Err(e) => {
    //             println!("\n{}\n", e);
    //         },
    //     }
    // }
}
