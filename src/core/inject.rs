use crate::vm::data::Data;

pub trait Inject<'a>: From<&'a Data> + Into<Data> {}

macro_rules! from_data {
    ($data:ident : $type:ty => $expr:expr) => {
        impl<'a> From<&'a Data> for $type {
            fn from($data: &'a Data) -> Self { $expr }
        }
    };
}

macro_rules! into_data {
    ($item:ident : $type:ty => $expr:expr) => {
        impl From<$type> for Data {
            fn from($item: $type) -> Self { $expr }
        }
    };
}

macro_rules! bind_data {
    ($type:ty) => {
        impl<'a> Inject<'a> for $type {}
    };
}

macro_rules! inject {
    // ($type:ty where $data:ident => $from:expr, $item:ident => $into:expr) =>
    // {     inject! { $type where $data => { $from } $item => { $into } }
    // };
    // ($type:ty where $data:ident => $from:expr, $item:ident => $into:expr,) =>
    // {     inject! { $type where $data => { $from } $item => { $into } }
    // };
    // ($type:ty where $data:ident => $from:block $item:ident => $into:expr,) =>
    // {     inject! { $type where $data => { $from } $item => { $into } }
    // };
    // ($type:ty where $data:ident => $from:block $item:ident => $into:block) =>
    // {     from_data! { $data: $type => $from }
    //     into_data! { $item: $type => $into }
    //     bind_data! { $type }
    // };
    ($type:ty where $data:ident => $from:expr, $item:ident => $into:expr,) => {
        from_data! { $data: $type => $from }
        into_data! { $item: $type => $into }
        bind_data! { $type }
    };
}

// Unit type

inject! {
    () where
    from => { assert_eq!(from, &Data::Unit); },
    into => Data::Unit,
}

// Floats

inject! {
    f64 where
    from => match from {
        Data::Float(f) => *f,
        _ => panic!(),
    },
    into => Data::Float(into),
}

// Integers

inject! {
    i64 where
    from => match from {
        Data::Integer(i) => *i,
        _ => panic!(),
    },
    into => Data::Integer(into),
}

// Booleans

inject! {
    bool where
    from => match from {
        Data::Boolean(b) => *b,
        _ => panic!(),
    },
    into => Data::Boolean(into),
}

// Strings

inject! {
    String where
    from => match from {
        Data::String(s) => s.to_string(),
        _ => panic!(),
    },
    into => Data::String(into),
}

// Tuples

// inject! {
//     Vec<Data> where
//     from => match from {
//         Data::Tuple(t) => t.to_owned(),
//         _ => panic!(),
//     },
//     into => Data::Tuple(into),
// }
