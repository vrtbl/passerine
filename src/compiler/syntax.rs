use std::fmt;
use crate::common::span::Span;

/// Represents a note attached to a Syntax error,
/// i.e. a location in source code with an optional
/// specific hint or tip.
#[derive(Debug, PartialEq, Eq)]
pub struct Note {
    pub span: Span,
    pub hint: Option<String>,
}

impl Note {
    pub fn new(span: Span) -> Note {
        Note { span, hint: None }
    }

    pub fn new_with_hint(hint: &str, span: &Span) -> Note {
        Note { span: span.clone(), hint: Some(hint.to_string()) }
    }
}

/// Represents a static error (syntax, semantics, etc.) found at compile time.
/// Ideally, each note included should have a distinct `Span` and hint.
/// Usually, one `Note` for an error is enough.
#[derive(Debug, PartialEq, Eq)]
pub struct Syntax {
    pub reason: String,
    pub notes:  Vec<Note>,
}

impl Syntax {
    /// Creates a new static error, with
    pub fn error(reason: &str, span: &Span) -> Syntax {
        Syntax::error_with_note(reason, Note { span: span.clone(), hint: None })
    }

    /// Creates a new static error, but with an added hint.
    pub fn error_with_note(reason: &str, note: Note) -> Syntax {
        Syntax {
            reason: reason.to_string(),
            notes:  vec![note],
        }
    }

    pub fn add_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }
}

impl fmt::Display for Syntax {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for note in self.notes.iter() {
            let formatted = note.span.format();

            if let Some(ref hint) = note.hint {
                if formatted.is_multiline() {
                    writeln!(f, "{}", formatted)?;
                    writeln!(f, "{} ├─ note: {} ", formatted.gutter_padding(), hint)?;
                    writeln!(f, "{} │", " ".repeat(formatted.gutter_padding()))?;
                } else {
                    writeln!(f, "In {}:{}:{}", formatted.path, formatted.start, formatted.start_col)?;
                    writeln!(f, "{} │", " ".repeat(formatted.gutter_padding()))?;
                    writeln!(f, "{} │ {}", formatted.start + 1, formatted.lines[0])?;
                    writeln!(f, "{} │ {}{} note: {}",
                        " ".repeat(formatted.gutter_padding()),
                        " ".repeat(formatted.start_col),
                        "^".repeat(formatted.carrots().unwrap()),
                        hint,
                    )?;
                    writeln!(f, "{} │", " ".repeat(formatted.gutter_padding()))?;
                }
            } else {
                write!(f, "{}", formatted)?;
            }
        }
        write!(f, "Syntax Error: {}", self.reason)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::source::Source;
    use std::rc::Rc;

    #[test]
    fn error() {
        // This is just a demo to check formatting
        // might not coincide with an actual Passerine error
        let source = Rc::new(Source::source("x = \"Hello, world\" -> y + 1"));
        let error = Syntax::error(
            "Unexpected token '\"Hello, world!\"'",
            &Span::new(&source, 4, 14),
        );

        let target = "In ./source:1:5
   |
 1 | x = \"Hello, world\" -> y + 1
   |     ^^^^^^^^^^^^^^
   |
Syntax Error: Unexpected token '\"Hello, world!\"'\
";

        let result = format!("{}", error);
        assert_eq!(result, target);
    }
}
