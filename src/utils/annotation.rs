use std::fmt::{Formatter, Display, Result};
use std::usize;
use crate::pipeline::source::Source;

// an annotation refers to a section of a source,
// much like &str, but a bit different at the same time
// but independant from the source itself
// they're meant to be paired with datastructures,
// and then be used during error reporting

// TODO: remove unnesary clones

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ann<'a> {
    pub source: &'a Source,
    pub offset: usize,
    pub length: usize,
}

impl<'a> Ann<'a> {
    pub fn new(source: &'a Source, offset: usize, length: usize) -> Ann<'a> {
        return Ann { source, offset, length };
    }

    pub fn empty() -> Ann<'a> {
        // this should trigger an error
        Ann { source: &Source::source(""), offset: 0, length: usize::MAX }
    }

    pub fn is_empty(self) -> bool {
        self == Ann::empty()
    }

    pub fn combine(a: &'a Ann, b: &'a Ann) -> Ann<'a> {
        // creates a new annotation which spans the space of the previous two
        // example:
        // hello this is cool
        // ^^^^^              | Ann a
        //            ^^      | Ann b
        // ^^^^^^^^^^^^^      | combined
        // ignore empty annotations

        if a.source != b.source {
            panic!("Can't combine two annotations with separate sources")
        }

        if a.is_empty() { return *b; }
        if b.is_empty() { return *a; }

        let offset = a.offset.min(b.offset);
        let end    = (a.offset + a.length).max(b.offset + b.length);
        let length = end - offset;

        return Ann::new(a.source, offset, length);
    }

    pub fn span(annotations: Vec<Ann>) -> Ann {
        if annotations.is_empty() { return Ann::empty() }

        // gee, reduce or an accumulator would be really useful here
        let mut combined = annotations[0];

        // Note: does [1..] throw error with length 1 array,
        // Or does it produce a [] array as I expect?
        for annotation in &annotations[1..] {
            combined = Ann::combine(&combined, annotation);
        }

        return combined;
    }

    pub fn contents(&self) -> String {
        if self.is_empty() { panic!("An empty annotation does not have any contents") }
        self.source.contents[self.offset..(self.offset + self.length)].to_string()
    }

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
impl Display for Ann<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Does:
        // 12 | x = blatant { error }
        //    |     ^^^^^^^^^^^^^^^^^
        // and:
        // 12 | > x -> {
        // 13 | >    y = x + 1
        // 14 | >    another { error }
        // 15 | > }

        if self.is_empty() {
            panic!("Can't display the section corresponding with an empty annotation")
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combination() {
        let source = Source::source("heck, that's awesome");
        let a = Ann::new(&source, 0, 5);
        let b = Ann::new(&source, 11, 2);

        assert_eq!(Ann::combine(&a, &b), Ann::new(&source, 0, 13));
    }

    #[test]
    fn span_and_contents() {
        let source = Source::source("hello, this is some text!");
        let anns   = vec![
            Ann::new(&source, 0,  8),
            Ann::new(&source, 7,  5),
            Ann::new(&source, 12, 4),
        ];
        let result = Ann::new(&source, 0, 16);

        assert_eq!(Ann::span(anns).contents(), result.contents());
    }
}
