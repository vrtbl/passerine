// I need to make sure that this points to a source,
// but doesn't clone it or make a copy of something...
// I don't need a bazillion copies of the same thing floating around

// an annotation refers to a section of a source,
// much like &str, but a bit different at the same time
// they're meant to be paired with datastructures,
// and then be used during error reporting

// TODO: remove unnesary clones

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Annotation {
    source: &'static str,
    offset: usize,
    length: usize,
}

impl Annotation {
    pub fn new(source: &'static str, offset: usize, length: usize) -> Annotation {
        if source.len() < (offset + length) {
            panic!("Can't annotate past end of source!")
        }

        Annotation {
            source: source,
            offset: offset,
            length: length,
        }
    }

    pub fn combine(a: &Annotation, b: &Annotation) -> Annotation {
        // creates a new annotation which spans the space of the previous two
        // example:
        // hello this is cool
        // ^^^^^              | Annotation a
        //            ^^      | Annotation b
        // ^^^^^^^^^^^^^      | combined

        // To compare pointers,
        // or to not compare...
        if a.source.as_ptr() != b.source.as_ptr() {
            panic!("Tried to merge two Annotations of different sources");
        }

        let offset = a.offset.min(b.offset);
        let end    = (a.offset + a.length).max(b.offset + b.length);
        let length = end - offset;

        return Annotation::new(a.source, offset, length);
    }

    pub fn span(annotations: Vec<Annotation>) -> Annotation {
        // gee, reduce or an accumulator would be really useful here
        let mut combined = annotations[0].clone();

        for annotation in &annotations[1..] {
            combined = Annotation::combine(&combined, annotation);
        }

        return combined;
    }

    pub fn contents(&self) -> &str {
        &self.source[self.offset..(self.offset + self.length)]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combination() {
        let source = "hello this is cool";
        let a = Annotation::new(source, 0, 5);
        let b = Annotation::new(source, 11, 2);

        assert_eq!(Annotation::combine(&a, &b), Annotation::new(source, 0, 13));
    }

    #[test]
    fn different() {
        let your_iq = "heck";
        let moms_iq = "holy cow, it's over 9000";

        assert_ne!(
            Annotation::new(your_iq, 0, 1),
            Annotation::new(moms_iq, 0, 1)
        );

        // less trivial
        // at first glance, they should be different
        // however, static &strs are used
        // which means the rust compiler reuses the same memory
        // which means your_iq is the same as an_idiots
        let an_idiots = "heck";
        assert_eq!(
            Annotation::new(your_iq, 0, 4),
            Annotation::new(an_idiots, 0, 4)
        );
    }

    #[test]
    fn span() {
        let source = "hello, this is some text!";
        let anns   = vec![
            Annotation::new(source, 0,  19),
            Annotation::new(source, 7,  18),
            Annotation::new(source, 0,  5),
            Annotation::new(source, 12, 4),
        ];
        let result = Annotation::new(source, 0, 25);

        assert_eq!(Annotation::span(anns), result);
    }
}
