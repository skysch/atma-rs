////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Palette cell definitions.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::expr::Expr;

// External library imports.
use serde::Serialize;
use serde::Deserialize;


////////////////////////////////////////////////////////////////////////////////
// Cell
////////////////////////////////////////////////////////////////////////////////
/// A cell holding a color expression.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct Cell {
    /// The cell's expression.
    expr: Expr,
}

impl Default for Cell {
    fn default() -> Self {
        Cell {
            expr: Default::default(),
        }
    }
}


////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell` in a palette.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub enum CellRef {
    /// A reference to a cell based on an internal index.
    Index(u32),

    /// A reference to a cell based on an assigned name.
    Name(String),

    /// A reference to a cell based on an assigned position.
    Position(Position),

    /// A reference to a cell based on an assigned group and index within that
    /// group.
    Group {
        /// The name of the group.
        name: String,
        /// The index of the cell within the group.
        idx: u32,
    },
}

impl std::fmt::Display for CellRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CellRef::*;
        match self {
            Index(idx) => write!(f, "index {}", idx),
            Name(name) => write!(f, "{}", name),
            Position(position) => write!(f, "{}", position),
            Group { name, idx } => write!(f, "{}:{}", name, idx),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// CellRef
////////////////////////////////////////////////////////////////////////////////
/// A reference to a `Cell` in a palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct Position {
    /// The page number of the cell.
    pub page: u16,
    /// The line number of the cell.
    pub line: u16,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$P{}L{}", self.page, self.line)
    }
}