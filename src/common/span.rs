use std::{
    fmt::{
        Formatter,
        Debug,
        Display,
        Result
    },
    usize,
    rc::Rc,
};

use crate::common::source::Source;

/// A `Span` refers to a section of a source,
/// much like a `&str`, but with a reference to a `Source` rather than a `String`.
/// A `Span` is  meant to be paired with other datastructures,
/// to be used during error reporting.
#[derive(Clone, Eq, PartialEq)]
pub struct Span {
    pub source: Option<Rc<Source>>,
    pub offset: usize,
    pub length: usize,
}

impl Span {
    /// Create a new `Span` from an offset with a length.
    /// All `Span`s have access to the `Source` from whence they came,
    /// So they can't be misinterpreted or miscombined.
    pub fn new(source: &Rc<Source>, offset: usize, length: usize) -> Span {
        Span { source: Some(Rc::clone(source)), offset, length }
    }

    /// A `Span` that points at a specific point in the source.
    pub fn point(source: &Rc<Source>, offset: usize) -> Span {
        // NOTE: maybe it should be 0?
        Span { source: Some(Rc::clone(source)), offset, length: 1 }
    }

    /// Create a new empty `Span`.
    /// An empty `Span` has only a source,
    /// if combined with another `Span`, the resulting `Span` will just be the other.
    pub fn empty() -> Span {
        Span { source: None, offset: 0, length: usize::MAX }
    }

    /// Checks if a `Span` is empty.
    pub fn is_empty(&self) -> bool {
        self.source == None
    }

    pub fn end(&self) -> usize {
        self.offset + self.length
    }

    /// Compares two Spans.
    /// returns true if this span starts the latest
    /// or is the longest in the case of a tie
    /// but false there is a total tie
    /// or otherwise
    pub fn later_than(&self, other: &Span) -> bool {
        self.offset > other.offset
           || (self.offset == other.offset
              && self.end() > other.end())
    }

    /// Creates a new `Span` which spans the space of the previous two.
    /// ```plain
    /// hello this is cool
    /// ^^^^^              | Span a
    ///            ^^      | Span b
    /// ^^^^^^^^^^^^^      | combined
    /// ```
    pub fn combine(a: &Span, b: &Span) -> Span {
        if a.is_empty() { return b.clone(); }
        if b.is_empty() { return a.clone(); }

        if a.source != b.source {
            panic!("Can't combine two Spans with separate sources")
        }

        let offset = a.offset.min(b.offset);
        let end    = a.end().max(b.end());
        let length = end - offset;

        // `a` should not be empty at this point
        return Span::new(&a.source.as_ref().unwrap(), offset, length);
    }

    /// Combines a set of `Span`s (think fold-left over `Span::combine`).
    pub fn join(mut spans: Vec<Span>) -> Span {
        let mut combined = match spans.pop() {
            Some(span) => span,
            None       => return Span::empty(),
        };

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
        if self.is_empty() { panic!("An empty span does not have any contents") }
        self.source.as_ref().unwrap().contents[self.offset..(self.end())].to_string()
    }

    // NOTE: once split_inclusive is included in rust's stdlib,
    // just replace this method with the std version.
    /// Splits a string by the newline character into a Vector of string slices.
    /// Includes the trailing newline in each slice.
    fn split_lines_inclusive(string: &str) -> Vec<&str> {
        let newline = "\n";

        let mut indicies: Vec<usize> = vec![0];
        indicies.append(&mut string
            .match_indices(newline).collect::<Vec<(usize, &str)>>()
            .into_iter().map(|(s, _)| s + newline.len()).collect()
        );
        indicies.push(string.len());

        println!("indicies: {:?}", indicies);

        let mut lines = vec![];
        for i in 0..(indicies.len() - 1) {
            lines.push(&string[indicies[i]..indicies[i + 1]]);
        }

        println!("done, lol");

        return lines;
    }

    /// Returns the start and end lines and columns of the `Span` if the `Span` is not empty.
    fn line_indicies(&self) -> Option<((usize, usize), (usize, usize))> {
        if self.is_empty() { panic!("Can not return the line indicies of an empty span") }

        let start = self.offset;
        let end   = self.end();

        let full_source = &self.source.as_ref().unwrap().contents;
        let start_lines: Vec<&str> = Span::split_lines_inclusive(&full_source[..=start]);
        let end_lines:   Vec<&str> = Span::split_lines_inclusive(&full_source[..end]);

        println!("{} {}", self.offset, self.length);
        println!("{:?}", full_source);
        println!("{:?}", start_lines);
        println!("{:?}", end_lines);

        let start_line = start_lines.len() - 1;
        let end_line   = end_lines.len() - 1;

        let start_col = start_lines.last()?.len() - 1;
        let end_col   = end_lines.last()?.len() - 1;

        return Some(((start_line, start_col), (end_line, end_col)));

    }
}

impl Debug for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if !self.is_empty() {
            write!(f, "Span {{ {:?}, ({}, {}) }}", self.contents(), self.offset, self.length)
        } else {
            write!(f, "Span {{ Empty }}")
        }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.is_empty() {
            panic!("Can't display the section corresponding with an empty Span")
        }

        let lines: Vec<&str> = self.source.as_ref().unwrap().contents.lines().collect();
        let ((start_line, start_col), (end_line, _end_col)) = match self.line_indicies() {
            Some(li) => li,
            None     => unreachable!(),
        };

        let readable_start_line = (start_line + 1).to_string();
        let readable_end_line   = (end_line   + 1).to_string();
        let readable_start_col  = (start_col  + 1).to_string();
        let padding = readable_end_line.len();

        let location  = format!(
            "While compiling {}:{}:{}",
            self.source.clone().unwrap()
                .path.to_string_lossy(),
            readable_start_line,
            readable_start_col
        );

        let separator = format!("{} |", " ".repeat(padding));

        if start_line == end_line {
            let l = lines[end_line];

            let line = format!("{} | {}", readable_end_line, l);
            let span = format!(
                "{} | {}{}",
                " ".repeat(padding),
                " ".repeat(start_col),
                "^".repeat(self.length),
            );

            writeln!(f, "{}", location)?;
            writeln!(f, "{}", separator)?;
            writeln!(f, "{}", line)?;
            writeln!(f, "{}", span)
        } else {
            let formatted = lines[start_line..=end_line]
                .iter()
                .enumerate()
                .map(|(i, l)| {
                    let readable_line_no = (start_line + i + 1).to_string();
                    let partial_padding = " ".repeat(padding - readable_line_no.len());
                    format!("{}{} > {}", partial_padding, readable_line_no, l)
                })
                .collect::<Vec<String>>()
                .join("\n");

            writeln!(f, "{}", location)?;
            writeln!(f, "{}", separator)?;
            writeln!(f, "{}", formatted)?;
            writeln!(f, "{}", separator)
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Spanned<T> {
    pub item: T,
    pub span: Span,
}

// TODO: docs
impl<T> Spanned<T> {
    /// Takes a prede
    pub fn new(item: T, span: Span) -> Spanned<T> {
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
        let spans   = vec![
            Span::new(&source, 0,  8),
            Span::new(&source, 7,  5),
            Span::new(&source, 12, 4),
        ];
        let result = Span::new(&source, 0, 16);

        assert_eq!(Span::join(spans).contents(), result.contents());
    }

    #[test]
    fn display() {
        let source = Source::source("hello\nbanana boat\nmagination\n");
        let span = Span::new(&source, 16, 12);
        assert_eq!(format!("{}", span), "\
            While compiling ./source:2:11\n  \
              |\n\
            2 > banana boat\n\
            3 > magination\n  \
              |\n\
            "
        )
    }
}
