use std::fmt::{Formatter, Display, Result};
use std::usize;
use crate::pipeline::source::Source;

/// A `Span` refers to a section of a source,
/// much like a `&str`, but with a reference to a `Source` rather than a `String`.
/// A `Span` is  meant to be paired with other datastructures,
/// to be used during error reporting.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Span<'a> {
    pub source: &'a Source,
    pub offset: usize,
    pub length: usize,
}

impl<'a> Span<'a> {
    /// Create a new `Span` from an offset with a length.
    /// All `Span`s have access to the `Source` from whence they came,
    /// So they can't be misinterpreted or miscombined.
    pub fn new(source: &'a Source, offset: usize, length: usize) -> Span<'a> {
        return Span { source, offset, length };
    }

    /// A `Span` that points at a specific point in the source.
    pub fn point(source: &'a Source, offset: usize) -> Span<'a> {
        // NOTE: maybe it should be 0?
        return Span { source, offset, length: 1 }
    }

    /// Create a new empty `Span`.
    /// An empty `Span` has only a source,
    /// if combined with another `Span`, the resulting `Span` will just be the other.
    pub fn empty() -> Span<'a> {
        Span { source: &Source::source(""), offset: 0, length: usize::MAX }
    }

    /// Checks if a `Span` is empty.
    pub fn is_empty(self) -> bool {
        self == Span::empty()
    }

    /// Creates a new `Span` which spans the space of the previous two.
    /// ```
    /// hello this is cool
    /// ^^^^^              | Span a
    ///            ^^      | Span b
    /// ^^^^^^^^^^^^^      | combined
    /// ```
    pub fn combine(a: &'a Span, b: &'a Span) -> Span<'a> {
        if a.source != b.source {
            panic!("Can't combine two Spanotations with separate sources")
        }

        if a.is_empty() { return *b; }
        if b.is_empty() { return *a; }

        let offset = a.offset.min(b.offset);
        let end    = (a.offset + a.length).max(b.offset + b.length);
        let length = end - offset;

        return Span::new(a.source, offset, length);
    }

    /// Combines a set of `Span`s (think fold-left over `Span::combine`).
    pub fn join(spans: Vec<Span>) -> Span {
        if spans.is_empty() { return Span::empty() }
        let mut combined = spans[0];

        for span in &spans[1..] {
            combined = Span::combine(&combined, span);
        }

        return combined;
    }

    /// Returns the contents of a `Span`.
    /// This indexes into the source file,
    /// so if the `Span` is along an invalid byte boundary or
    /// is empty, the program will panic.
    pub fn contents(&self) -> String {
        if self.is_empty() { panic!("An empty span does not have any contents") }
        self.source.contents[self.offset..(self.offset + self.length)].to_string()
    }

    /// Returns the start and end lines and columns of the `Span` if the `Span` is not empty.
    fn line_indicies(&self) -> Option<((usize, usize), (usize, usize))> {
        if self.is_empty() {
            return None;
        }

        let start = self.offset;
        let end   = self.offset + self.length;

        let start_lines: Vec<&str> = self.source.contents[..=start].lines().collect();
        let end_lines:   Vec<&str> = self.source.contents[..=end].lines().collect();

        let start_line = start_lines.len() - 1;
        let end_line   = end_lines.len() - 1;

        let start_col = start_lines.last()?.len() - 1;
        let end_col   = end_lines.last()?.len() - 1;

        return Some(((start_line, start_col), (end_line, end_col)));
    }
}

// TODO: tests
impl Display for Span<'_> {
    /// Given a `Span`, `fmt` will print out where the `Span` occurs in its source.
    /// Single-line `Span`s:
    /// ```
    /// 12 | x = blatant { error }
    ///    |     ^^^^^^^^^^^^^^^^^
    /// ```
    /// Multi-line `Span`s:
    /// ```
    /// 12 | > x -> {
    /// 13 | >    y = x + 1
    /// 14 | >    another { error }
    /// 15 | > }
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_empty() {
            panic!("Can't display the section corresponding with an empty Spanotation")
        }

        let lines: Vec<&str> = self.source.contents.lines().collect();
        let ((start_line, start_col), (end_line, end_col)) = match self.line_indicies() {
            Some(li) => li,
            None     => unreachable!(),
        };

        let readable_start_line = (start_line + 1).to_string();
        let readable_end_line   = (end_line   + 1).to_string();
        let readable_start_col  = (start_col  + 1).to_string();
        let padding = readable_end_line.len();

        let location  = format!("Line {}:{}", readable_start_line, readable_start_col);
        let separator = format!("{} |", " ".repeat(padding));

        if start_line == end_line {
            // TODO: Error here:
            let l = lines[end_line];

            let line = format!("{} | {}", readable_end_line, l);
            let span = format!(
                "{} | {}{}",
                " ".repeat(padding),
                " ".repeat(start_col),
                "^".repeat(self.length),
            );

            writeln!(f, "{}", location);
            writeln!(f, "{}", separator);
            writeln!(f, "{}", line);
            writeln!(f, "{}", span)
        } else {
            let formatted = lines[start_line..end_line]
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    let readable_line_no = (start_line + i + 1).to_string();
                    let partial_padding = " ".repeat(padding - readable_line_no.len());
                    format!("{}{} | > {}", partial_padding, readable_line_no, l)
                })
                .collect::<Vec<String>>()
                .join("\n");

            writeln!(f, "{}", location);
            writeln!(f, "{}", separator);
            writeln!(f, "{}", formatted)
        }
    }
}

/// A wrapper for spanning types.
/// For example, a token, such as
/// ```
/// pub enum Token {
///     Number(f64),
///     Open,
///     Close,
/// }
/// ```
/// or the like, can be spanned to indicate where it was parsed from (a `Spanned<Token>`).
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Spanned<'a, T> {
    pub item: T,
    pub span: Span<'a>,
}

// TODO: docs
impl<'a, T> Spanned<'a, T> {
    /// Takes a prede
    pub fn new(item: T, span: Span<'a>) -> Spanned<'a, T> {
        Spanned { item, span }
    }

    /// a destructive alias for `self.item`
    pub fn into(self) -> T { self.item }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combination() {
        let source = Source::source("heck, that's awesome");
        let a = Span::new(&source, 0, 5);
        let b = Span::new(&source, 11, 2);

        assert_eq!(Span::combine(&a, &b), Span::new(&source, 0, 13));
    }

    #[test]
    fn span_and_contents() {
        let source = Source::source("hello, this is some text!");
        let Spans   = vec![
            Span::new(&source, 0,  8),
            Span::new(&source, 7,  5),
            Span::new(&source, 12, 4),
        ];
        let result = Span::new(&source, 0, 16);

        assert_eq!(Span::join(Spans).contents(), result.contents());
    }
}
