////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parser helpers.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::parse::*;
use crate::cell::CellRef;
use crate::cell::Position;
use crate::selection::CellSelector;
use crate::selection::PositionSelector;

// Standard library imports.
// use std::convert::TryInto;
// use std::convert::TryFrom;

pub(crate) const REF_ALL_TOKEN: char = '*';
pub(crate) const REF_POS_SEP_TOKEN: char = '.';
pub(crate) const REF_PREFIX_TOKEN: char = ':';
pub(crate) const REF_RANGE_TOKEN: char = '-';
pub(crate) const REF_SEP_TOKEN: char = ',';




////////////////////////////////////////////////////////////////////////////////
// Parse cell selections.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelection.
pub fn cell_selection<'t>(text: &'t str)
    -> ParseResult<'t, Vec<CellSelector<'t>>>
{
    unimplemented!()
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell selectors.
////////////////////////////////////////////////////////////////////////////////
/// Parses a CellSelector.
pub fn cell_selector<'t>(text: &'t str) -> ParseResult<'t, CellSelector<'t>> {
    // if let Ok(all_suc) = char(REF_ALL_TOKEN)(text) {
    //     Ok(all_suc.map_value(|_| CellSelector::All))
    // } else if let Ok(pos_suc) = position(text) {

    // }

    unimplemented!()


}

/// Parses a PositionSelector.
pub fn position_selector<'t>(text: &'t str)
    -> ParseResult<'t, PositionSelector>
{
    let pre_suc = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell ref position selector prefix")?;
    
    let page_suc = u16_or_all(pre_suc.rest)
        .with_parse_context(pre_suc.token, text)
        .source_for("cell ref position selector page")?;
    let page_suc = pre_suc.join_with(page_suc, text, |_, p| p);
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(page_suc.rest)
        .with_parse_context(page_suc.token, text)
        .source_for("cell ref position selector separator")?;
    let sep_suc = page_suc.join_with(sep_suc, text, |p, _| p);
    
    let line_suc = u16_or_all(sep_suc.rest)
        .with_parse_context(sep_suc.token, text)
        .source_for("cell ref position selector line")?;
    let line_suc = sep_suc.join_with(line_suc, text, |p, l| (p, l));
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(line_suc.rest)
        .with_parse_context(line_suc.token, text)
        .source_for("cell ref position selector separator")?;
    let sep_suc = line_suc.join_with(sep_suc, text, |(p, l), _| (p, l));

    let column_suc = u16_or_all(sep_suc.rest)
        .with_parse_context(sep_suc.token, text)
        .source_for("cell ref position selector column")?;
    let column_suc = sep_suc.join_with(column_suc, text, |(p, l), c| (p, l, c));
    
    Ok(column_suc.map_value(|(page, line, column)| PositionSelector {
        page,
        line,
        column,
    }))
}


pub fn u16_or_all<'t>(text: &'t str) -> ParseResult<'t, Option<u16>> {
    if let Ok(all_suc) = char(REF_ALL_TOKEN)(text) {
        Ok(all_suc.map_value(|_| None))
    } else {
        uint::<u16>("u16")(text).map_value(Some)
    }
}

pub fn range_suffix<'t, F, V>(mut parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    move |text| {
        let ws_suc = maybe(whitespace)(text).unwrap();

        let range_suc = char(REF_RANGE_TOKEN)(ws_suc.rest)
            .with_parse_context(ws_suc.token, text)
            .source_for("cell selector range separator")?;
        let range_suc = ws_suc.join(range_suc, text);

        let ws_suc = maybe(whitespace)(range_suc.rest).unwrap();
        let ws_suc = range_suc.join(ws_suc, text);

        let parser_suc = (parser)(ws_suc.rest)
            .with_parse_context(ws_suc.token, text)
            .source_for("cell selector range upper bound")?;
        let parser_suc = ws_suc.join_with(parser_suc, text, |_, r| r);

        Ok(parser_suc)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Parse cell refs.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellRef.
pub fn cell_ref<'t>(text: &'t str) -> ParseResult<'t, CellRef<'t>> {
    
    if let Ok(pos_suc) = position(text) {
        Ok(pos_suc.map_value(|pos| CellRef::Position(pos)))
    
    } else if let Ok(idx_suc) = index(text) {
        Ok(idx_suc.map_value(|idx| CellRef::Index(idx)))
    
    } else if let Ok(group_suc) = group(text) {
        Ok(group_suc.map_value(|(group, idx)| CellRef::Group {
            group: group.into(),
            idx,
        }))

    } else {
        name(text)
            .map_value(|name| CellRef::Name(name.into()))
            .with_parse_context("", text)
            .source_for("cell ref value")
    }
}

/// Parses a Position.
pub fn position<'t>(text: &'t str) -> ParseResult<'t, Position> {
    let pre_suc = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell ref position prefix")?;
    
    let page_suc = uint::<u16>("u16")(pre_suc.rest)
        .with_parse_context(pre_suc.token, text)
        .source_for("cell ref position page")?;
    let page_suc = pre_suc.join_with(page_suc, text, |_, p| p);
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(page_suc.rest)
        .with_parse_context(page_suc.token, text)
        .source_for("cell ref position separator")?;
    let sep_suc = page_suc.join_with(sep_suc, text, |p, _| p);
    
    let line_suc = uint::<u16>("u16")(sep_suc.rest)
        .with_parse_context(sep_suc.token, text)
        .source_for("cell ref position line")?;
    let line_suc = sep_suc.join_with(line_suc, text, |p, l| (p, l));
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(line_suc.rest)
        .with_parse_context(line_suc.token, text)
        .source_for("cell ref position separator")?;
    let sep_suc = line_suc.join_with(sep_suc, text, |(p, l), _| (p, l));

    let column_suc = uint::<u16>("u16")(sep_suc.rest)
        .with_parse_context(sep_suc.token, text)
        .source_for("cell ref position column")?;
    let column_suc = sep_suc.join_with(column_suc, text, |(p, l), c| (p, l, c));
    
    Ok(column_suc.map_value(|(page, line, column)| Position {
        page,
        line,
        column,
    }))
}

/// Parses a Index.
pub fn index<'t>(text: &'t str) -> ParseResult<'t, u32> {
    let pre = char(REF_PREFIX_TOKEN)(text)
        .with_parse_context("", text)
        .source_for("cell ref index prefix")?;

    uint::<u32>("u32")(pre.rest)
        .with_parse_context(pre.token, text)
        .source_for("cell ref index prefix")
}


/// Parses a name.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let valid_char = char_matching(|c| ![
        REF_ALL_TOKEN,
        REF_SEP_TOKEN,
        REF_POS_SEP_TOKEN,
        REF_PREFIX_TOKEN,
        REF_RANGE_TOKEN,
    ].contains(&c));

    let res = one_or_more(valid_char)(text)
        .with_parse_context("", text)
        .source_for("cell ref name")?;

    let context = &text[0..(text.len() - res.rest.len())];
    Ok(Success {
        value: context.trim(),
        token: context,
        rest: res.rest,
    })
}

/// Parses a group name with its index.
pub fn group<'t>(text: &'t str) -> ParseResult<'t, (&'t str, u32)> {
    let name_suc = name(text)
        .with_parse_context("", text)
        .source_for("cell ref group name")?;

    let index_suc = index(name_suc.rest)
        .with_parse_context(name_suc.token, text)
        .source_for("cell ref group index")?;

    Ok(name_suc.join_with(index_suc, text, |group, idx| (group, idx)))
}
