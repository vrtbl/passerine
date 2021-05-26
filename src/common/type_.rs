/// Built-in Passerine datatypes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Passerine Data (Atomic)
    Real,
    Integer,
    Boolean,
    String,

    // Function
    Function {
        arg:  Box<Type>,
        body: Box<Type>,
    },
    Fiber {
        takes:  Box<Type>,
        yields: Box<Type>,
    },

    // Compound
    Tuple(Vec<Type>),
    List(Box<Type>),
    Record(Vec<Type>), // TODO: names for records and enums
    Enum(Vec<Type>),
}
