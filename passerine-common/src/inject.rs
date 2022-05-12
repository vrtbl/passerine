use crate::data::Data;

/// Indicates that a Rust data structure can be serialized to Passerine data,
/// and that passerine data can be deserialized into that datastructure.
/// The conversion should be round-trip such that `T ==
/// deserialize(serialize(T))`.
pub trait Inject
where
    Self: Sized,
{
    /// An infallible conversion from self to `Data`.
    fn serialize(item: Self) -> Data;
    /// A potentially fallible conversion (due to malformed `Data`) that creates
    /// `Self` by deserializing some `Data`.
    fn deserialize(data: Data) -> Option<Self>;
}

macro_rules! impl_inject {
    ($type:ty where $data:ident => $from:expr, $item:ident => $into:expr,) => {
        // With the above two implemented,
        // we can implement inject automatically.
        impl Inject for $type {
            fn serialize($item: Self) -> Data { $into }
            fn deserialize($data: Data) -> Option<Self> { $from }
        }
    };
}

// Data type

impl_inject! {
    Data where
    from => Some(from),
    into => into,
}

// Unit type

impl_inject! {
    () where
    from => { assert_eq!(from, Data::Unit); Some(()) },
    _into => Data::Unit,
}

// Floats

impl_inject! {
    f64 where
    from => match from {
        Data::Float(f) => Some(f),
        _ => None,
    },
    into => Data::Float(into),
}

// Integers

impl_inject! {
    i64 where
    from => match from {
        Data::Integer(i) => Some(i),
        _ => None,
    },
    into => Data::Integer(into),
}

// Booleans

impl_inject! {
    bool where
    from => match from {
        Data::Boolean(b) => Some(b),
        _ => None,
    },
    into => Data::Boolean(into),
}

// Strings

impl_inject! {
    String where
    from => match from {
        Data::String(s) => Some(s),
        _ => None,
    },
    into => Data::String(into),
}

// Tuples

// impl_inject! {
//     Vec<Data> where
//     from => match from {
//         Data::Tuple(t) => t.to_owned(),
//         _ => panic!(),
//     },
//     into => Data::Tuple(into),
// }
