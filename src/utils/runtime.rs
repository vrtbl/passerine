use std::fmt;
use crate::utils::span::{ Span, Spanned };

/// Represents a static error (syntax, semantics, etc.) found at compile time
pub struct Syntax<'a> {
    message: String,
    span:    Span<'a>,
}

impl<'a> Syntax<'a> {
    pub fn error(message: &str, span: Span<'a>) -> Syntax<'a> {
        Syntax { message: message.to_string(), span }
    }
}

impl fmt::Display for Syntax<'_> {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.span, f);
        writeln!(f, "Encountered a Static Error: {}", self.message)
    }
}

/// Represents a runtime error, i.e. a traceback
pub struct Trace<'a> {
    kind: String, // TODO: enum?
    message: String,
    spans: Vec<Span<'a>>,
}

impl<'a> Trace<'a> {
    pub fn error(kind: &str, message: &str, spans: Vec<Span<'a>>) -> Trace<'a> {
        Trace {
            kind: kind.to_string(),
            message: message.to_string(),
            spans,
        }
    }
}

impl fmt::Display for Trace<'_> {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Traceback, most recent call last:");

        for span in self.spans.iter() {
            fmt::Display::fmt(span, f);
        }

        writeln!(f, "Runtime {}: {}", self.kind, self.message)
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
        let error = Result::syntax(
            "Unexpected token '\"Hello, world!\"'",
            Span::new(&source, 4, 14),
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

        let traceback = Result::trace(
            "TypeError",
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
