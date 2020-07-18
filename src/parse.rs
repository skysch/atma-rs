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
mod color;
mod combinator;
mod expr;
mod primitive;
mod result;
mod script;
mod selection;

// Exports.
pub use self::color::*;
pub use self::combinator::*;
pub use self::expr::*;
pub use self::primitive::*;
pub use self::result::*;
pub use self::script::*;
pub use self::selection::*;

