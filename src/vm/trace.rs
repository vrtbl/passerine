use std::fmt;
use crate::common::span::Span;

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
    use crate::common::source::Source;
    use std::rc::Rc;

    #[test]
    fn traceback() {
        // TODO: this method of checking source code is ugly

        let source = Rc::new(Source::source("incr = x -> x + 1
dub_incr = z -> (incr x) + (incr x)
forever = a -> a = a + (dub_incr a)
forever RandomLabel
"));
        let target = "\
            Traceback, most recent call last:\n\
            Line 1:13\n  \
              |\n\
            1 | incr = x -> x + 1\n  \
              |             ^^^^^\n\
            Line 2:17\n  \
              |\n\
            2 | dub_incr = z -> (incr x) + (incr x)\n  \
              |                 ^^^^^^^^\n\
            Line 3:24\n  \
              |\n\
            3 | forever = a -> a = a + (dub_incr a)\n  \
              |                        ^^^^^^^^^^^^\n\
            Line 4:1\n  \
              |\n\
            4 | forever RandomLabel\n  \
              | ^^^^^^^^^^^^^^^^^^^\n\
            Runtime Type Error: Can't add Label to Label\n\
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
