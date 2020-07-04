////////////////////////////////////////////////////////////////////////////////
// Atma structured color palette
////////////////////////////////////////////////////////////////////////////////
// Copyright 2020 Skylor R. Schermer
// This code is dual licenced using the MIT or Apache 2 license.
// See licence-mit.md and licence-apache.md for details.
////////////////////////////////////////////////////////////////////////////////
//! Parse primitives.
////////////////////////////////////////////////////////////////////////////////
// TODO: This module is currently under development.
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(missing_docs)]

// Local imports.
use crate::color::Cmyk;
use crate::color::Color;
use crate::color::Hsl;
use crate::color::Hsv;
use crate::color::Rgb;
use crate::color::Xyz;
use crate::parse::bracket;
use crate::parse::char_in;
use crate::parse::circumfix;
use crate::parse::Failure;
use crate::parse::float;
use crate::parse::intersperse_collect;
use crate::parse::literal_ignore_ascii_case;
use crate::parse::maybe;
use crate::parse::ParseResult;
use crate::parse::ParseResultExt as _;
use crate::parse::postfix;
use crate::parse::prefix;
use crate::parse::repeat;
use crate::parse::Success;
use crate::parse::uint_digits_value;
use crate::parse::whitespace;

// Standard library imports.
use std::borrow::Cow;


/// Floating point sign token.
pub const RGB_HEX_PREFIX: char = '#';

/// Floating point sign token.
pub const FUNCTIONAL_SEPARATOR: char = ',';
pub const FUNCTIONAL_OPEN_BRACKET: char = '(';
pub const FUNCTIONAL_CLOSE_BRACKET: char = ')';

////////////////////////////////////////////////////////////////////////////////
// Color
////////////////////////////////////////////////////////////////////////////////

/// Parses a Color.
pub fn color<'t>(text: &'t str) -> ParseResult<'t, Color> {
    let mut fail = Failure {
        context: "",
        expected: "color value".into(),
        source: None,
        rest: text,
    };

    match rgb_hex(text).or_else(|_| rgb_functional(text)) {
        Ok(rgb_suc)   => return Ok(rgb_suc.map_value(Color::from)),
        Err(rgb_fail) => {
            fail.context = rgb_fail.context;
            fail.rest = rgb_fail.rest;
            fail.source = Some(Box::new(rgb_fail.to_owned()));
        }
    }

    match cmyk_functional(text) {
        Ok(cmyk_suc)   => return Ok(cmyk_suc.map_value(Color::from)),
        Err(cmyk_fail) => if cmyk_fail.context.len() > fail.context.len() {
            fail.context = cmyk_fail.context;
            fail.rest = cmyk_fail.rest;
            fail.source = Some(Box::new(cmyk_fail.to_owned()));
        }
    }

    match hsv_functional(text) {
        Ok(hsv_suc)   => return Ok(hsv_suc.map_value(Color::from)),
        Err(hsv_fail) => if hsv_fail.context.len() > fail.context.len() {
            fail.context = hsv_fail.context;
            fail.rest = hsv_fail.rest;
            fail.source = Some(Box::new(hsv_fail.to_owned()));
        }
    }
    
    match hsl_functional(text) {
        Ok(hsl_suc)   => return Ok(hsl_suc.map_value(Color::from)),
        Err(hsl_fail) => if hsl_fail.context.len() > fail.context.len() {
            fail.context = hsl_fail.context;
            fail.rest = hsl_fail.rest;
            fail.source = Some(Box::new(hsl_fail.to_owned()));
        }
    }
    
    match xyz_functional(text) {
        Ok(xyz_suc)   => return Ok(xyz_suc.map_value(Color::from)),
        Err(xyz_fail) => if xyz_fail.context.len() > fail.context.len() {
            fail.context = xyz_fail.context;
            fail.rest = xyz_fail.rest;
            fail.source = Some(Box::new(xyz_fail.to_owned()));
        }
    }

    Err(fail)
}


////////////////////////////////////////////////////////////////////////////////
// rgb_hex
////////////////////////////////////////////////////////////////////////////////

/// Parses an RGB hex code.
pub fn rgb_hex<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    rgb_hex_6(text).or_else(|_| rgb_hex_3(text))
}


/// Parses a 6-digit RGB hex code.
pub fn rgb_hex_6<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    use crate::parse::char;

    let suc = prefix(
            uint_digits_value::<u32>("u32", 6, Some(6), 16),
            char(RGB_HEX_PREFIX))
        (text)?;

    Ok(suc.map_value(Rgb::from))
}

/// Parses a 3-digit RGB hex code.
pub fn rgb_hex_3<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    use crate::parse::char;

    let suc = prefix(
            uint_digits_value::<u32>("u32", 3, Some(3), 16),
            char(RGB_HEX_PREFIX))
        (text)?;

    Ok(suc.map_value(|v| {
        let mut expanded = 0;
        expanded |= v & 0x00F;
        expanded |= (v & 0x00F) << 4;
        expanded |= (v & 0x0F0) << 4;
        expanded |= (v & 0x0F0) << 8;
        expanded |= (v & 0xF00) << 8;
        expanded |= (v & 0xF00) << 12;
        Rgb::from(expanded)
    }))
}


////////////////////////////////////////////////////////////////////////////////
// functional notation
////////////////////////////////////////////////////////////////////////////////

/// Parses an RGB value from it functional notation.
pub fn rgb_functional<'t>(text: &'t str) -> ParseResult<'t, Rgb> {
    let suc = prefix(
            functional(3),
            literal_ignore_ascii_case("rgb"))
        (text)?;
    let rgb = Rgb::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
    ]);

    Ok(suc.map_value(|_| rgb))
}

/// Parses an HSV value from it functional notation.
pub fn hsv_functional<'t>(text: &'t str) -> ParseResult<'t, Hsv> {
    let suc = prefix(
            functional(3),
            literal_ignore_ascii_case("hsv"))
        (text)?;
    let hsv = Hsv::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
    ]);

    Ok(suc.map_value(|_| hsv))
}

/// Parses an HSL value from it functional notation.
pub fn hsl_functional<'t>(text: &'t str) -> ParseResult<'t, Hsl> {
    let suc = prefix(
            functional(3),
            literal_ignore_ascii_case("hsl"))
        (text)?;
    let hsl = Hsl::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
    ]);

    Ok(suc.map_value(|_| hsl))
}

/// Parses an CMYK value from it functional notation.
pub fn cmyk_functional<'t>(text: &'t str) -> ParseResult<'t, Cmyk> {
    let suc = prefix(
            functional(4),
            literal_ignore_ascii_case("cmyk"))
        (text)?;
    let cmyk = Cmyk::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
        suc.value[3],
    ]);

    Ok(suc.map_value(|_| cmyk))
}

/// Parses an XYZ value from it functional notation.
pub fn xyz_functional<'t>(text: &'t str) -> ParseResult<'t, Xyz> {
    let suc = prefix(
            functional(3),
            literal_ignore_ascii_case("xyz"))
        (text)?;
    let xyz = Xyz::from([
        suc.value[0],
        suc.value[1],
        suc.value[2],
    ]);

    Ok(suc.map_value(|_| xyz))
}

/// Returns a parser which parses a functional suffix with n float parameters.
fn functional<'t>(n: usize)
    -> impl FnMut(&'t str) -> ParseResult<'t, Vec<f32>>
{
    use crate::parse::char;
    bracket(
        intersperse_collect(n, Some(n),
            float::<f32>("f32"),
            circumfix(
                char(FUNCTIONAL_SEPARATOR),
                maybe(whitespace))),
        postfix(
            char(FUNCTIONAL_OPEN_BRACKET),
            maybe(whitespace)),
        prefix(
            char(FUNCTIONAL_CLOSE_BRACKET),
            maybe(whitespace)))
}