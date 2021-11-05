//! This module contains the core VM implementation.
//! Note that these modules are public for documentation visiblility,
//! But should never be used outside of the module by `common` or `compiler`.

pub mod vm;

pub mod tag;
pub mod stack;
pub mod trace;
pub mod slot;
pub mod data;
pub mod closure;
