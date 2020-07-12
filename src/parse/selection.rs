////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Cell selection parsing.
////////////////////////////////////////////////////////////////////////////////

// Local imports.
use crate::cell::CellRef;
use crate::cell::Position;
use crate::parse::char;
use crate::parse::char_matching;
use crate::parse::circumfix;
use crate::parse::Failure;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::repeat;
use crate::parse::intersperse_collect;
use crate::parse::Success;
use crate::parse::uint;
use crate::parse::whitespace;
use crate::cell::CellSelection;
use crate::cell::CellSelector;
use crate::cell::PositionSelector;
use crate::cell::REF_ALL_TOKEN;
use crate::cell::REF_POS_SEP_TOKEN;
use crate::cell::REF_PREFIX_TOKEN;
use crate::cell::REF_RANGE_TOKEN;
use crate::cell::REF_SEP_TOKEN;



////////////////////////////////////////////////////////////////////////////////
// Parse cell selections.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelection.
pub fn cell_selection<'t>(text: &'t str) -> ParseResult<'t, CellSelection<'t>> {
    // TODO: Permit empty selection?
    intersperse_collect(1, None,
            cell_selector,
            circumfix(
                char(REF_SEP_TOKEN),
                maybe(whitespace)))
        (text)
        .source_for("cell selection")
        .map_value(CellSelection::from)
}


////////////////////////////////////////////////////////////////////////////////
// Parse cell selectors.
////////////////////////////////////////////////////////////////////////////////

/// Parses a CellSelector.
pub fn cell_selector<'t>(text: &'t str) -> ParseResult<'t, CellSelector<'t>> {
    if let Ok(all_suc) = char(REF_ALL_TOKEN)(text) {
        Ok(all_suc.map_value(|_| CellSelector::All))

    } else if let Ok(pos_suc) = position(text) {
        if let Ok(ub_suc) = range_suffix(position)(pos_suc.rest) {
            match CellSelector::position_range(pos_suc.value, ub_suc.value) {
                Ok(pos_range) => Ok(pos_suc.join_with(ub_suc, text,
                    |_, _| pos_range)),
                Err(range_err) => Err(Failure {
                    context: pos_suc.join(ub_suc, text).token,
                    expected: "valid position range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(pos_suc.map_value(
                |pos| CellSelector::PositionSelector(pos.into())))
        }

    } else if let Ok(pos_sel_suc) = position_selector(text) {
            Ok(pos_sel_suc.map_value(
                |pos_sel| CellSelector::PositionSelector(pos_sel)))

    } else if let Ok(index_suc) = index(text) {
        if let Ok(ub_suc) = range_suffix(index)(index_suc.rest) {
            match CellSelector::index_range(index_suc.value, ub_suc.value) {
                Ok(index_range) => Ok(index_suc.join_with(ub_suc, text,
                    |_, _| index_range)),
                Err(range_err) => Err(Failure {
                    context: index_suc.join(ub_suc, text).token,
                    expected: "valid index range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(index_suc.map_value(
                |index| CellSelector::Index(index)))
        }

    } else if let Ok(group_all_suc) = group_all(text) {
            Ok(group_all_suc.map_value(
                |group| CellSelector::GroupAll(group.into())))

    } else if let Ok(group_suc) = group(text) {
        if let Ok(ub_suc) = range_suffix(group)(group_suc.rest) {
            match CellSelector::group_range(
                group_suc.value.0.into(),
                group_suc.value.1,
                ub_suc.value.0.into(),
                ub_suc.value.1)
            {
                Ok(group_range) => Ok(group_suc.join_with(ub_suc, text,
                    |_, _| group_range)),
                Err(range_err) => Err(Failure {
                    context: group_suc.join(ub_suc, text).token,
                    expected: "valid group range".into(),
                    source: Some(Box::new(range_err)),
                    rest: text,
                }),
            }
        } else {
            Ok(group_suc.map_value(|(group, idx)| CellSelector::Group {
                group: group.into(), 
                idx 
            }))
        }

    } else {
        name(text)
            .map_value(|name| CellSelector::Name(name.into()))
            .source_for("cell selector value")
    }
}

/// Parses a PositionSelector.
pub fn position_selector<'t>(text: &'t str)
    -> ParseResult<'t, PositionSelector>
{
    let pre_suc = char(REF_PREFIX_TOKEN)(text)
        .source_for("cell ref position selector prefix")
        .with_new_context("", text)?;
    
    let page_suc = u16_or_all(pre_suc.rest)
        .source_for("cell ref position selector page")
        .with_join_context(&pre_suc, text)?;
    let page_suc = pre_suc.join_with(page_suc, text, |_, p| p);
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(page_suc.rest)
        .source_for("cell ref position selector separator")
        .with_join_context(&page_suc, text)?;
    let sep_suc = page_suc.join_with(sep_suc, text, |p, _| p);
    
    let line_suc = u16_or_all(sep_suc.rest)
        .source_for("cell ref position selector line")
        .with_join_context(&sep_suc, text)?;
    let line_suc = sep_suc.join_with(line_suc, text, |p, l| (p, l));
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(line_suc.rest)
        .source_for("cell ref position selector separator")
        .with_join_context(&line_suc, text)?;
    let sep_suc = line_suc.join_with(sep_suc, text, |(p, l), _| (p, l));

    let column_suc = u16_or_all(sep_suc.rest)
        .source_for("cell ref position selector column")
        .with_join_context(&sep_suc, text)?;
    let column_suc = sep_suc.join_with(column_suc, text, |(p, l), c| (p, l, c));
    
    Ok(column_suc.map_value(|(page, line, column)| PositionSelector {
        page,
        line,
        column,
    }))
}

/// Parses a u16 or an 'all' selector token. Return Some if a u16 was parsed, or
/// None if 'all' was parsed.
pub fn u16_or_all<'t>(text: &'t str) -> ParseResult<'t, Option<u16>> {
    if let Ok(all_suc) = char(REF_ALL_TOKEN)(text) {
        Ok(all_suc.map_value(|_| None))
    } else {
        uint::<u16>("u16")(text).map_value(Some)
    }
}

/// Returns a parser which parses the separator and upper bound of a range using
/// the given parser.
pub fn range_suffix<'t, F, V>(parser: F)
    -> impl FnMut(&'t str) -> ParseResult<'t, V>
    where F: FnMut(&'t str) -> ParseResult<'t, V>
{
    prefix(
        parser, 
        circumfix(
            char(REF_RANGE_TOKEN),
            maybe(whitespace)))
}

/// Parses a group all selector.
pub fn group_all<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    postfix(
        name, 
        postfix(
            char(REF_PREFIX_TOKEN),
            char(REF_ALL_TOKEN)))
    (text)
        .source_for("cell ref group all")
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
            .source_for("cell ref value")
    }
}

/// Parses a Position.
pub fn position<'t>(text: &'t str) -> ParseResult<'t, Position> {
    let pre_suc = char(REF_PREFIX_TOKEN)(text)
        .source_for("cell ref position prefix")
        .with_new_context("", text)?;
    
    let page_suc = uint::<u16>("u16")(pre_suc.rest)
        .source_for("cell ref position page")
        .with_join_context(&pre_suc, text)?;
    let page_suc = pre_suc.join_with(page_suc, text, |_, p| p);
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(page_suc.rest)
        .source_for("cell ref position separator")
        .with_join_context(&page_suc, text)?;
    let sep_suc = page_suc.join_with(sep_suc, text, |p, _| p);
    
    let line_suc = uint::<u16>("u16")(sep_suc.rest)
        .source_for("cell ref position line")
        .with_join_context(&sep_suc, text)?;
    let line_suc = sep_suc.join_with(line_suc, text, |p, l| (p, l));
    
    let sep_suc = char(REF_POS_SEP_TOKEN)(line_suc.rest)
        .source_for("cell ref position separator")
        .with_join_context(&line_suc, text)?;
    let sep_suc = line_suc.join_with(sep_suc, text, |(p, l), _| (p, l));

    let column_suc = uint::<u16>("u16")(sep_suc.rest)
        .source_for("cell ref position column")
        .with_join_context(&sep_suc, text)?;
    let column_suc = sep_suc.join_with(column_suc, text, |(p, l), c| (p, l, c));
    
    Ok(column_suc.map_value(|(page, line, column)| Position {
        page,
        line,
        column,
    }))
}

/// Parses a Index.
pub fn index<'t>(text: &'t str) -> ParseResult<'t, u32> {
    prefix(
        uint::<u32>("u32"),
        char(REF_PREFIX_TOKEN))
    (text)
        .source_for("cell ref index")
}


/// Parses a name.
pub fn name<'t>(text: &'t str) -> ParseResult<'t, &'t str> {
    let valid_char = char_matching(|c| ![
        REF_ALL_TOKEN,
        REF_SEP_TOKEN,
        REF_POS_SEP_TOKEN,
        REF_PREFIX_TOKEN,
        REF_RANGE_TOKEN,
    ].contains(&c) && !c.is_whitespace());

    let res = repeat(1, None, valid_char)(text)
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
        .source_for("cell ref group name")?;

    let index_suc = index(name_suc.rest)
        .source_for("cell ref group index")
        .with_join_context(&name_suc, text)?;

    Ok(name_suc.join_with(index_suc, text, |group, idx| (group, idx)))
}
