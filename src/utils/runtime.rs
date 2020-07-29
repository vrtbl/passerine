use std::fmt;
use crate::utils::span::Span;

/// Represents a static error (syntax, semantics, etc.) found at compile time
#[derive(PartialEq, Eq)]
pub struct Syntax {
    message: String,
    span:    Span,
}

impl Syntax {
    pub fn error(message: &str, span: Span) -> Syntax {
        Syntax { message: message.to_string(), span }
    }
}

impl fmt::Display for Syntax {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.span, f)?;
        writeln!(f, "Encountered a Static Error: {}", self.message)
    }
}

impl fmt::Debug for Syntax {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

/// Represents a runtime error, i.e. a traceback
#[derive(Debug, PartialEq, Eq)]
pub struct Trace {
    kind: String, // TODO: enum?
    message: String,
    spans: Vec<Span>,
}

impl Trace {
    pub fn error(kind: &str, message: &str, spans: Vec<Span>) -> Trace {
        Trace {
            kind: kind.to_string(),
            message: message.to_string(),
            spans,
        }
    }
}

impl fmt::Display for Trace {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Traceback, most recent call last:")?;

        for span in self.spans.iter() {
            fmt::Display::fmt(span, f)?;
        }

        writeln!(f, "Runtime {}: {}", self.kind, self.message)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pipeline::source::Source;
    use std::rc::Rc;

    #[test]
    fn error() {
        // This is just a demo to check formatting
        // might not coincide with an actual Passerine error
        let source = Rc::new(Source::source("x = \"Hello, world\" -> y + 1"));
        let error = Syntax::error(
            "Unexpected token '\"Hello, world!\"'",
            Span::new(&source, 4, 14),
        );

        let target = "Line 1:5
  |
1 | x = \"Hello, world\" -> y + 1
  |     ^^^^^^^^^^^^^^
Encountered a Static Error: Unexpected token '\"Hello, world!\"'
";

        let result = format!("{}", error);
        assert_eq!(result, target);
    }

    #[test]
    fn traceback() {
        // TODO: this method of checking source code is ugly

        let source = Rc::new(Source::source("incr = x -> x + 1
dub_incr = z -> (incr x) + (incr x)
forever = a -> a = a + (dub_incr a)
forever RandomLabel
"));
        let target = "Traceback, most recent call last:
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
Runtime Type Error: Can't add Label to Label
";

        let traceback = Trace::error(
            "Type Error",
            "Can't add Label to Label",
            vec![
                (Span::new(&source, 12, 5)),
                (Span::new(&source, 34, 8)),
                (Span::new(&source, 77, 12)),
                (Span::new(&source, 90, 19)),
            ]
        );

        let result = format!("{}", traceback);
        assert_eq!(result, target);
    }
}
