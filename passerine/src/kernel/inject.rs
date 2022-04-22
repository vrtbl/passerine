use crate::vm::data::Data;

pub trait Inject: TryFrom<Data, Error = ()> + Into<Data> {}

macro_rules! impl_inject {
    ($type:ty where $data:ident => $from:expr, $item:ident => $into:expr,) => {
        // Data -> Item conversion
        impl TryFrom<Data> for $type {
            type Error = ();
            fn try_from($data: Data) -> Result<Self, ()> { $from }
        }

        // Item -> Data conversion
        impl From<$type> for Data {
            fn from($item: $type) -> Self { $into }
        }

        // With the above two implemented,
        // we can implement inject automatically.
        impl Inject for $type {}
    };
}

// Unit typec

impl_inject! {
    () where
    from => { assert_eq!(from, Data::Unit); Ok(()) },
    _into => Data::Unit,
}

// Floats

impl_inject! {
    f64 where
    from => match from {
        Data::Float(f) => Ok(f),
        _ => Err(()),
    },
    into => Data::Float(into),
}

// Integers

impl_inject! {
    i64 where
    from => match from {
        Data::Integer(i) => Ok(i),
        _ => Err(()),
    },
    into => Data::Integer(into),
}

// Booleans

impl_inject! {
    bool where
    from => match from {
        Data::Boolean(b) => Ok(b),
        _ => Err(()),
    },
    into => Data::Boolean(into),
}

// Strings

impl_inject! {
    String where
    from => match from {
        Data::String(s) => Ok(s),
        _ => Err(()),
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
