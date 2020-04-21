use std::usize;

// an annotation refers to a section of a source,
// much like &str, but a bit different at the same time
// but independant from the source itself
// they're meant to be paired with datastructures,
// and then be used during error reporting

// TODO: remove unnesary clones

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Ann {
    offset: usize,
    length: usize,
}

impl Ann {
    pub fn new(offset: usize, length: usize) -> Ann {
        return Ann { offset, length };
    }

    pub fn empty() -> Ann {
        // this should trigger an error
        Ann { offset: 0, length: usize::MAX }
    }

    pub fn is_empty(self) -> bool {
        self == Ann::empty()
    }

    pub fn combine(a: &Ann, b: &Ann) -> Ann {
        // creates a new annotation which spans the space of the previous two
        // example:
        // hello this is cool
        // ^^^^^              | Ann a
        //            ^^      | Ann b
        // ^^^^^^^^^^^^^      | combined

        let offset = a.offset.min(b.offset);
        let end    = (a.offset + a.length).max(b.offset + b.length);
        let length = end - offset;

        return Ann::new(offset, length);
    }

    pub fn span(annotations: Vec<Ann>) -> Ann {
        if annotations.is_empty() { panic!("Expected at least one annotation to span"); }

        // gee, reduce or an accumulator would be really useful here
        let mut combined = annotations[0];

        // Note: does [1..] throw error with length 1 array,
        // Or does it produce a [] array as I expect?
        for annotation in &annotations[1..] {
            combined = Ann::combine(&combined, annotation);
        }

        return combined;
    }

    pub fn contents(&self, source: &str) -> String {
        source[self.offset..(self.offset + self.length)].to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combination() {
        let a = Ann::new(0, 5);
        let b = Ann::new(11, 2);

        assert_eq!(Ann::combine(&a, &b), Ann::new(0, 13));
    }

    #[test]
    fn span_and_contents() {
        let source = "hello, this is some text!";
        let anns   = vec![
            Ann::new(0,  8),
            Ann::new(7,  5),
            Ann::new(12, 4),
        ];
        let result = Ann::new(0, 16);

        assert_eq!(Ann::span(anns).contents(source), result.contents(source));
    }
}
