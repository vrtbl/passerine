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
    /// Creates a new traceback
    pub fn error(kind: &str, message: &str, spans: Vec<Span>) -> Trace {
        Trace {
            kind: kind.to_string(),
            message: message.to_string(),
            spans,
        }
    }

    /// Used to add context (i.e. function calls) while unwinding the stack.
    pub fn add_context(&mut self, span: Span) {
        self.spans.push(span);
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: better message?
        writeln!(f, "Traceback, most recent call last:")?;

        for span in self.spans.iter().rev() {
            fmt::Display::fmt(span, f)?;
        }

        write!(f, "Runtime {} Error: {}", self.kind, self.message)
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use crate::common::source::Source;
}
