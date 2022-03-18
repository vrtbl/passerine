use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TySymbol(usize);

/// Built-in Passerine datatypes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    // Passerine Data (Atomic)
    Float,
    Integer,
    // Boolean, // TODO: should be just standard library enum?
    String,

    // Function
    // <arg> -> <body> / <ty>
    Function {
        arg: TySymbol,
        body: TySymbol,
        effect: TySymbol,
    },
    // TODO: fibers still good idea?
    Fiber {
        takes: TySymbol,
        yields: TySymbol,
    },

    // Compound
    Tuple(Vec<TySymbol>),
    List(TySymbol),
    // TODO: hashmap?
    Record(Vec<TySymbol>), // TODO: names for records and enums
    Enum(Vec<TySymbol>),
}

pub struct TyPool {
    tys: HashMap<TySymbol, Ty>,
}
