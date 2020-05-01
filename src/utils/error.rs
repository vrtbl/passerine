use std::fmt::{Display, Formatter, Result};

use crate::vm::data::Data;
use crate::utils::annotation::Ann;

pub enum PResult<'a, T> {
    Ok(T),
    // Kind, Message, Source, Annotation
    Error(String, String, &'a str, Ann),
    Trace(String, String, Vec<(&'a str, Ann)>),
}

fn line_indicies(source: &str, ann: Ann) -> Option<((usize, usize), (usize, usize))> {
    if ann.is_empty() {
        return None;
    }

    let start = ann.offset;
    let end   = ann.offset + ann.length;

    let start_lines: Vec<&str> = source[..=start].lines().collect();
    let end_lines:   Vec<&str> = source[..=end].lines().collect();

    let start_line = start_lines.len() - 1;
    let end_line   = end_lines.len() - 1;

    let start_col = start_lines.last()?.len() - 1;
    let end_col   = end_lines.last()?.len() - 1;

    return Some(((start_line, start_col), (end_line, end_col)));
}

fn display_section(source: &str, ann: Ann) -> String {
    // Does:
    // 12 | x = blatant { error }
    //    |     ^^^^^^^^^^^^^^^^^
    // and:
    // 12 | > x -> {
    // 13 | >    y = x + 1
    // 14 | >    another { error }
    // 15 | > }

    if ann.is_empty() {
        panic!("Can't display the section corresponding with an empty annotation")
    }

    let lines: Vec<&str> = source.lines().collect();
    let ((start_line, start_col), (end_line,   end_col)) = match line_indicies(source, ann) {
        Some(((s, c), (e, l))) => ((s, c), (e, l)),
        None                 => unreachable!(),
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
            "^".repeat(ann.length),
        );

        return location + "\n" + &separator + "\n" + &line + "\n" + &span + "\n";
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

        return location + "\n" + &separator + "\n" + &formatted + "\n";
    }
}

// TODO: error and trace are really similar...
// Combine?

impl<T> Display for PResult<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            PResult::Ok(_) => panic!("Can't display a non-error result!"),

            PResult::Error(l, m, s, a) => {
                write!(f, "{}", display_section(s, *a));
                write!(f, "Encountered a {}: {}", l, m.to_string())
            },

            PResult::Trace(l, m, v) => {
                write!(f, "Traceback, most recent call last\n");

                for (s, a) in v.iter() {
                    write!(f, "{}", display_section(s, *a));
                }

                write!(f, "Runtime {}: {}", l, m.to_string())
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn error() {
        // This is just a demo to check formatting
        // might not coincide with an actual Passerine error
        let error: PResult<'_, ()> = PResult::Error(
            "SyntaxError".to_string(),
            "Unexpected token '\"Hello, world!\"'".to_string(),
            "x = \"Hello, world\" -> y + 1",
            Ann::new(4, 14),
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

        let source = "incr = x -> x + 1
dub_incr = z -> (incr x) + (incr x)
forever = a -> a = a + (dub_incr a)
forever RandomLabel
";
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

        let traceback: PResult<'_, ()> = PResult::Trace(
            "TypeError".to_string(),
            "Can't add Label to Label".to_string(),
            vec![
                (source, Ann::new(12, 5)),
                (source, Ann::new(34, 8)),
                (source, Ann::new(77, 12)),
                (source, Ann::new(90, 19)),
            ]
        );

        let result = format!("{}", traceback);
        assert_eq!(result, target);
    }
}
