use crate::pipeline::ast::Node;
use crate::pipeline::

// so, constanst table is made by walking the tree and sweeping for values
// then, a second pass walks the tree and builds the bytecode
// then, a third pass walks the tree and optimizes the bytecode
