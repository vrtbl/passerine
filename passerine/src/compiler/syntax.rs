use std::fmt;

use crate::common::span::Span;

/// Represents a note attached to a Syntax error,
/// i.e. a location in source code with an optional
/// specific hint or tip corresponding this this specific location
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
        Note {
            span: span.clone(),
            hint: Some(hint.to_string()),
        }
    }
}

/// Represents a static error (syntax, semantics, etc.) found at compile time.
/// Ideally, each note included should have a distinct `Span` and hint.
/// Usually, one `Note` per error is enough.
#[derive(Debug, PartialEq, Eq)]
pub struct Syntax {
    pub reason: String,
    pub notes: Vec<Note>,
}

impl Syntax {
    /// Creates a new static error with a single note that does not have a hint.
    pub fn error(reason: &str, span: &Span) -> Syntax {
        Syntax::error_with_note(
            reason,
            Note {
                span: span.clone(),
                hint: None,
            },
        )
    }

    /// Creates a new static error with a single note that may or may not have a
    /// hint.
    pub fn error_with_note(reason: &str, note: Note) -> Syntax {
        Syntax {
            reason: reason.to_string(),
            notes: vec![note],
        }
    }

    /// Creates a syntax error without a note. This syntax error will not
    /// contain any location information, so only use it if you plan to add
    /// additional notes with [`add_note`] later.
    pub fn error_no_note(reason: &str) -> Syntax {
        Syntax {
            reason: reason.to_string(),
            notes: vec![],
        }
    }

    /// Extend a syntax error by adding another note to the error.
    pub fn add_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }
}

impl fmt::Display for Syntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for note in self.notes.iter() {
            let formatted = note.span.format();

            if let Some(ref hint) = note.hint {
                if formatted.is_multiline() {
                    writeln!(f, "{}", formatted)?;
                    writeln!(f, "{} |- note: {} ", formatted.gutter_padding(), hint)?;
                    writeln!(f, "{} |", " ".repeat(formatted.gutter_padding()))?;
                } else {
                    writeln!(
                        f,
                        "In {}:{}:{}",
                        formatted.path, formatted.start, formatted.start_col
                    )?;
                    writeln!(f, "{} |", " ".repeat(formatted.gutter_padding()))?;
                    writeln!(f, "{} | {}", formatted.start + 1, formatted.lines[0])?;
                    writeln!(
                        f,
                        "{} | {}{} note: {}",
                        " ".repeat(formatted.gutter_padding()),
                        " ".repeat(formatted.start_col),
                        "^".repeat(formatted.carrots().unwrap()),
                        hint,
                    )?;
                    writeln!(f, "{} |", " ".repeat(formatted.gutter_padding()))?;
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
    use std::rc::Rc;

    use super::*;
    use crate::common::source::Source;

    #[test]
    fn error() {
        // This is just a demo to check formatting
        // might not coincide with an actual Passerine error
        let source = Rc::new(Source::source("x = \"Hello, world\" -> y + 1"));
        let error = Syntax::error(
            "Unexpected token '\"Hello, world!\"'",
            &Span::new(&source, 4, 14),
        );

        let target = r#"In ./source:1:5
  |
1 | x = "Hello, world" -> y + 1
  |     ^^^^^^^^^^^^^^
Syntax Error: Unexpected token '"Hello, world!"'"#;

        let result = format!("{}", error);
        assert_eq!(result, target);
    }
}
