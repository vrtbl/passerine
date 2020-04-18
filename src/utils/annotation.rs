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
        // TODO: this checks for in-bounds annotations,
        // but is this even nessesary?
        // if source.len() < (offset + length) {
        //     panic!("Can't annotate past end of source!")
        // }

        return Ann { offset, length };
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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combination() {
        let source = "hello this is cool";
        let a = Ann::new(source, 0, 5);
        let b = Ann::new(source, 11, 2);

        assert_eq!(Ann::combine(&a, &b), Ann::new(source, 0, 13));
    }

    #[test]
    fn different() {
        let your_iq = "heck";
        let moms_iq = "holy cow, it's over 9000";

        assert_ne!(
            Ann::new(your_iq, 0, 1),
            Ann::new(moms_iq, 0, 1)
        );

        // less trivial
        // at first glance, they should be different
        // however, static &strs are used
        // which means the rust compiler reuses the same memory
        // which means your_iq is the same as an_idiots
        let an_idiots = "heck";
        assert_eq!(
            Ann::new(your_iq, 0, 4),
            Ann::new(an_idiots, 0, 4)
        );
    }

    #[test]
    fn span() {
        let source = "hello, this is some text!";
        let anns   = vec![
            Ann::new(source, 0,  19),
            Ann::new(source, 7,  18),
            Ann::new(source, 0,  5),
            Ann::new(source, 12, 4),
        ];
        let result = Ann::new(source, 0, 25);

        assert_eq!(Ann::span(anns), result);
    }
}
