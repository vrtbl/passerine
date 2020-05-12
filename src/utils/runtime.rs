use std::fmt;
// use std::ops::Try;

use crate::utils::span::{ Span, Spanned };

/// This is a Result returned by functions in the passerine compiler.
/// - `Ok(_)` indicates ok, and returns a value to be used by the next step.
/// - `Syntax(message, span)` denotes a syntax or static error,
///    caught by the compiler at compile-time.
/// - `Trace(kind, message, spans)` denotes a runtime error, and has a traceback.
/// Both `Syntax(...)` and `Trace(...)` can be `Displayed`.
pub enum Result<'a, T> {
    Ok(T),
    // Not a spanned string for consistency with Trace
    // TODO: merge syntax and trace, or make syntax spanned string?
    Syntax(String, Span<'a>),
    Trace(String, String, Vec<Span<'a>>),
}

impl<'a, T> Result<'a, T> {
    pub fn syntax(message: &str, span: Span<'a>) -> Result<'a, T> {
        Result::Syntax(message.to_string(), span)
    }

    pub fn trace(kind: &str, message: &str, spans: Vec<Span<'a>>) -> Result<'a, T> {
        Result::Trace(kind.to_string(), message.to_string(), spans)
    }
}

impl<T> fmt::Display for Result<'_, T> {
    /// Prints the corrosponding annotations, and the error message.
    /// If the `Result` variant is *not* an error, this function will panic.
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Result::Ok(e) => panic!("Can't display a non-error"),
            Result::Syntax(m, a) => {
                fmt::Display::fmt(a, f);
                writeln!(f, "Encountered a Static Error: {}", m)
            },
            Result::Trace(k, m, v) => {
                writeln!(f, "Traceback, most recent call last");

                for a in v.iter() {
                    fmt::Display::fmt(a, f);
                }

                writeln!(f, "Runtime {}: {}", k, m)
            },
        }
    }
}

// TODO: make error it's own type?
// idk, man
// is this not idiomatic?
// std Result expects 1 error type, but runtime Result has two.
// should they be their own things?
// impl<'a, T> Try for Result<'a, T> {
//     type Ok    = T;
//     type Error = Result<'a>;
//
//     fn into_result(self) -> std::result::Result<Self::Ok, Self::Error> {
//         match self {
//             Result::Ok(item) => item,
//             other            => other,
//         }
//     }
//
//     fn from_error(v: Self::Error) -> Self { v }
//     fn from_ok(v: Self::Ok)       -> Self { Result::Ok(v) }
// }

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
