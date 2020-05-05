use std::fmt::{Display, Formatter, Result};

use crate::utils::annotation::Ann;

#[derive(Debug)]
pub enum CompilerError<'a> {
    Syntax(&'a str, Ann<'a>), // Message, Annotation
    Trace(&'a str, &'a str, Vec<Ann<'a>>), // Kind, Message, Annotation
}

impl Display for CompilerError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            CompilerError::Syntax(m, a) => {
                Display::fmt(a, f);
                writeln!(f, "Encountered a Static Error: {}", m)
            },

            CompilerError::Trace(l, m, v) => {
                writeln!(f, "Traceback, most recent call last");

                for a in v.iter() {
                    Display::fmt(a, f);
                }

                writeln!(f, "Runtime {}: {}", l, m)
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pipeline::source::Source;

    #[test]
    fn error() {
        // This is just a demo to check formatting
        // might not coincide with an actual Passerine error
        let source = Source::source("x = \"Hello, world\" -> y + 1");
        let error = CompilerError::Syntax(
            "Unexpected token '\"Hello, world!\"'",
            Ann::new(&source, 4, 14),
        );

        let target = "Line 1:5
  |
1 | x = \"Hello, world\" -> y + 1
  |     ^^^^^^^^^^^^^^
Encountered a SyntaxError: Unexpected token '\"Hello, world!\"'";

        let result = format!("{}", error);
        assert_eq!(result, target);
    }

    #[test]
    fn traceback() {
        // TODO: this method of checking source code is ugly

        let source = Source::source("incr = x -> x + 1
dub_incr = z -> (incr x) + (incr x)
forever = a -> a = a + (dub_incr a)
forever RandomLabel
");
        let target = "Traceback, most recent call last
Line 1:13
  |
1 | incr = x -> x + 1
  |             ^^^^^
Line 2:17
  |
2 | dub_incr = z -> (incr x) + (incr x)
  |                 ^^^^^^^^
Line 3:24
  |
3 | forever = a -> a = a + (dub_incr a)
  |                        ^^^^^^^^^^^^
Line 4:1
  |
4 | forever RandomLabel
  | ^^^^^^^^^^^^^^^^^^^
Runtime TypeError: Can't add Label to Label";

        let traceback = CompilerError::Trace(
            "TypeError",
            "Can't add Label to Label",
            vec![
                (Ann::new(&source, 12, 5)),
                (Ann::new(&source, 34, 8)),
                (Ann::new(&source, 77, 12)),
                (Ann::new(&source, 90, 19)),
            ]
        );

        let result = format!("{}", traceback);
        assert_eq!(result, target);
    }
}
