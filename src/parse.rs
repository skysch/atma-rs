////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parsing module.
////////////////////////////////////////////////////////////////////////////////

// Internal modules.
mod result;
mod combinator;
mod error;
mod primitive;
mod selection;

// Exports.
pub use self::result::*;
pub use self::combinator::*;
pub use self::error::*;
pub use self::primitive::*;
pub use self::selection::*;

