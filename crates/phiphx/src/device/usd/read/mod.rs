//! USDA text parser — recursive-descent parser that produces a `UsdStage`
//! from the human-readable `.usda` serialization format.
//!
//! Mirrors the parsing subset of Apple's USDKit framework for single-layer,
//! sublayer-free `.usda` documents.

mod array;
mod attribute;
mod lexer;
mod metadata;
pub mod peek;
mod prim;
pub mod scene;
mod skip;
mod value;

use crate::device::usd::error::UsdaError;
use crate::device::usd::stage::UsdStage;

/// A hand-written recursive-descent parser over USDA text.
///
/// Operates on a `&str` with a cursor position, consuming tokens
/// character-by-character.
pub struct UsdaParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> UsdaParser<'a> {
    /// Parse USDA text into a `UsdStage`.
    ///
    /// Returns `UsdaError::MissingHeader` if the input does not begin with
    /// the `#usda 1.0` magic header.
    pub fn parse(input: &'a str) -> Result<UsdStage, UsdaError> {
        let mut parser = Self { input, pos: 0 };
        parser.skip_whitespace();
        parser.expect_magic_header()?;
        let mut stage = UsdStage::new();
        parser.skip_whitespace_and_comments();

        // Check for stage metadata block: ( ... )
        if parser.peek() == Some('(') {
            parser.advance();
            parser.parse_stage_metadata(&mut stage)?;
        }

        // Parse top-level prim definitions
        parser.skip_whitespace_and_comments();
        while parser.pos < parser.input.len() {
            parser.skip_whitespace_and_comments();
            if parser.pos >= parser.input.len() {
                break;
            }
            if parser.starts_with("def ")
                || parser.starts_with("over ")
                || parser.starts_with("class ")
            {
                let prim = parser.parse_prim("")?;
                stage.root.add_child(prim);
            } else {
                parser.advance();
            }
        }

        Ok(stage)
    }

    fn expect_magic_header(&mut self) -> Result<(), UsdaError> {
        self.skip_whitespace();
        if self.starts_with("#usda 1.0") {
            self.pos += 9;
            self.skip_to_eol();
            Ok(())
        } else {
            Err(UsdaError::MissingHeader)
        }
    }

    fn parse_error(&self, path: &str, attr: &str, msg: &str) -> UsdaError {
        UsdaError::ParseError {
            path: path.to_string(),
            attr: attr.to_string(),
            msg: msg.to_string(),
        }
    }
}
