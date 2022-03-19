use std::{
    fmt::{self, Debug, Display, Formatter},
    rc::Rc,
    usize,
};

use crate::common::source::Source;

/// A `Span` refers to a section of a source,
/// much like a `&str`, but with a reference to a `Source` rather than a `String`.
/// A `Span` is  meant to be paired with other datastructures,
/// to be used during error reporting.
#[derive(Clone, Eq, PartialEq)]
pub struct Span {
    source: Rc<Source>,
    offset: usize,
    length: usize,
}

impl Span {
    /// Create a new `Span` from an offset with a length.
    /// All `Span`s have access to the `Source` from whence they came,
    /// So they can't be misinterpreted or miscombined.
    pub fn new(source: &Rc<Source>, offset: usize, length: usize) -> Span {
        Span {
            source: Rc::clone(source),
            offset,
            length,
        }
    }

    /// A `Span` that points at a specific point in the source.
    /// Has a length of `0`.
    pub fn point(source: &Rc<Source>, offset: usize) -> Span {
        Span {
            source: Rc::clone(source),
            offset,
            length: 0,
        }
    }

    /// Return the index of the end of the `Span`.
    pub fn end(&self) -> usize {
        self.offset + self.length
    }

    pub fn len(&self) -> usize {
        self.length
    }

    /// Creates a new `Span` which spans the space of the previous two.
    /// ```plain
    /// hello this is cool
    /// ^^^^^              | Span a
    ///            ^^      | Span b
    /// ^^^^^^^^^^^^^      | combined
    /// ```
    pub fn combine(a: &Span, b: &Span) -> Span {
        if a.source != b.source {
            panic!("Can't combine two Spans with separate sources");
        }

        let offset = a.offset.min(b.offset);
        let end = a.end().max(b.end());
        let length = end - offset;

        return Span::new(&a.source, offset, length);
    }

    /// Combines a set of `Span`s (think fold-left over `Span::combine`).
    /// If the vector of spans passed in is empty, this method panics.
    pub fn join(mut spans: Vec<Span>) -> Span {
        let mut combined = spans.pop().expect("Expected at least one span");

        while let Some(span) = spans.pop() {
            combined = Span::combine(&combined, &span)
        }

        return combined;
    }

    /// Returns the contents of a `Span`.
    /// This indexes into the source file,
    /// so if the `Span` is along an invalid byte boundary or
    /// is empty, the program will panic.
    pub fn contents(&self) -> String {
        self.source.as_ref().contents[self.offset..self.end()].to_string()
    }

    pub fn lines(&self) -> Vec<String> {
        let full_source = &self.source.as_ref().contents;
        let lines: Vec<_> = full_source.split("\n").collect();
        let start_line = self.line(self.offset);
        let end_line = self.line(self.end());
        let slice = lines[start_line..=end_line]
            .iter()
            .map(|s| s.to_string())
            .collect();
        return slice;
    }

    pub fn path(&self) -> String {
        self.source.clone().path.to_string_lossy().to_string()
    }

    pub fn line(&self, index: usize) -> usize {
        let lines = self.source.contents[..index].split_inclusive("\n").count();
        return lines.saturating_sub(1);
    }

    pub fn col(&self, index: usize) -> usize {
        let lines = &self.source.contents[..index]
            .split_inclusive("\n")
            .last()
            .unwrap_or("")
            .chars()
            .count();
        return *lines;
    }

    pub fn format(&self) -> FormattedSpan {
        FormattedSpan {
            path: self.path(),
            start: self.line(self.offset),
            lines: self.lines(),
            start_col: self.col(self.offset),
            end_col: self.col(self.end()),
        }
    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Span")
            .field("contents", &self.contents())
            .field("start", &self.offset)
            .field("end", &self.end())
            .finish()
    }
}

// TODO: tests
// TODO: this can be vastly simplified
impl Display for Span {
    /// Given a `Span`, `fmt` will print out where the `Span` occurs in its source.
    /// Single-line `Span`s:
    /// ```plain
    /// 12 | x = blatant { error }
    ///    |     ^^^^^^^^^^^^^^^^^
    /// ```
    /// Multi-line `Span`s:
    /// ```plain
    /// 12 > x -> {
    /// 13 >    y = x + 1
    /// 14 >    another { error }
    /// 15 > }
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.format())
    }
}

/// Represents a formatted span, ready to be displayed.
/// Contains information about where the span is from,
/// and where in the text it starts and ends
/// relative to the lines in the source.
pub struct FormattedSpan {
    pub path: String,
    pub start: usize,
    pub lines: Vec<String>,
    pub start_col: usize,
    pub end_col: usize,
}

impl FormattedSpan {
    pub fn is_multiline(&self) -> bool {
        self.lines.len() != 1
    }

    pub fn end(&self) -> usize {
        (self.start - 1) + self.lines.len()
    }

    pub fn gutter_padding(&self) -> usize {
        self.start.to_string().len()
    }

    /// If a single line span, returns the number of carrots between cols.
    pub fn carrots(&self) -> Option<usize> {
        if self.lines.len() == 1 {
            Some(self.end_col - self.start_col)
        } else {
            None
        }
    }
}

impl Display for FormattedSpan {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "In {}:{}:{}",
            self.path,
            self.start + 1,
            self.start_col + 1
        )?;
        writeln!(f, "{} |", " ".repeat(self.gutter_padding()))?;

        if !self.is_multiline() {
            writeln!(f, "{} | {}", self.start + 1, self.lines[0])?;
            writeln!(
                f,
                "{} | {}{}",
                " ".repeat(self.gutter_padding()),
                " ".repeat(self.start_col),
                "^".repeat(self.carrots().unwrap().max(1)),
            )?;
        } else {
            for (index, line) in self.lines.iter().enumerate() {
                let line_no = (self.start + index).to_string();
                let padding = " ".repeat(self.gutter_padding() - line_no.len());
                writeln!(f, "{}{} > {}", line_no, padding, line)?;
            }
        }

        Ok(())
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    /// Takes a generic item, and wraps in in a `Span` to make it `Spanned`.
    pub fn new(item: T, span: Span) -> Spanned<T> {
        Spanned { item, span }
    }

    /// Joins a Vector of spanned items into a single span.
    pub fn build(spanneds: &[Spanned<T>]) -> Span {
        let spans = spanneds
            .iter()
            .map(|s| s.span.clone())
            .collect::<Vec<Span>>();
        Span::join(spans)
    }

    /// Applies a function a `Spanned`'s item.
    pub fn map<B, E>(self, f: fn(T) -> Result<B, E>) -> Result<Spanned<B>, E> {
        Ok(Spanned::new(f(self.item)?, self.span))
    }
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
        let spans = vec![
            Span::new(&source, 0, 8),
            Span::new(&source, 7, 5),
            Span::new(&source, 12, 4),
        ];
        let result = Span::new(&source, 0, 16);

        assert_eq!(Span::join(spans).contents(), result.contents());
    }

    #[test]
    fn empty() {
        let source = Source::source("");
        let span = Span::point(&source, 0);
        format!("{}", span);
    }
}
