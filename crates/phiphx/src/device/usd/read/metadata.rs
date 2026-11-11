//! Stage metadata parsing — populates `UsdStage` fields from the `( ... )`
//! metadata block that follows the `#usda 1.0` header.

use crate::device::usd::stage::{UsdStage, UsdUpAxis};
use crate::device::usd::read::UsdaParser;

impl<'a> UsdaParser<'a> {
    /// Parse the stage metadata block (already past the opening `(`).
    /// Populates `default_prim`, `up_axis`, and `meters_per_unit`.
    pub(crate) fn parse_stage_metadata(
        &mut self,
        stage: &mut UsdStage,
    ) -> Result<(), super::super::error::UsdaError> {
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(')') {
                self.advance();
                break;
            }
            if self.pos >= self.input.len() {
                break;
            }
            let key = self.read_identifier();
            if key.is_empty() {
                self.advance();
                continue;
            }
            self.skip_whitespace();
            if self.peek() != Some('=') {
                continue;
            }
            self.advance(); // consume '='
            self.skip_whitespace();

            match key.as_str() {
                "defaultPrim" => {
                    let val = self.read_string_literal();
                    stage.default_prim = Some(val);
                }
                "upAxis" => {
                    let val = self.read_string_literal();
                    stage.up_axis = match val.as_str() {
                        "Z" => UsdUpAxis::Z,
                        _ => UsdUpAxis::Y,
                    };
                }
                "metersPerUnit" => {
                    let val = self.read_number_str();
                    if let Ok(v) = val.parse::<f64>() {
                        stage.meters_per_unit = v;
                    }
                }
                _ => {
                    self.skip_value();
                }
            }
        }
        Ok(())
    }
}
