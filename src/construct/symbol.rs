/// Represents a symbol that corresponds to a name.
/// In other words, if two variables have the same name,
/// even if they exist in different scopes,
/// They will have the same [`SharedSymbol`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SharedSymbol(pub usize);

/// Represents a unique symbol that corresponds to a single variable.
/// In other words, if two variables with the same name exist in different scopes,
/// They will have different [`UniqueSymbol`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UniqueSymbol(pub usize);
