//! Value parsing — decodes typed values from USDA text into `UsdValue`
//! variants. Scalar and inferred-type parsing lives here; array/tuple readers
//! are in the sibling `arrays` module.

use crate::device::usd::attribute::UsdValue;
use crate::device::usd::error::UsdaError;
use crate::device::usd::read::UsdaParser;

impl<'a> UsdaParser<'a> {
    /// Parse a value based on the declared type hint.
    pub(crate) fn parse_value(
        &mut self,
        type_hint: &str,
        prim_path: &str,
        attr_name: &str,
    ) -> Result<UsdValue, UsdaError> {
        self.skip_whitespace();
        match type_hint {
            "bool" => self.parse_bool(prim_path, attr_name),
            "int" => self.parse_int(prim_path, attr_name),
            "float" | "double" | "float1" => self.parse_float(prim_path, attr_name),
            "token" => Ok(UsdValue::Token(self.read_string_literal())),
            "string" | "asset" => Ok(UsdValue::String(self.read_string_or_asset_literal())),
            "float3" | "double3" | "normal3f" | "point3f" | "vector3f"
            | "color3f" => {
                let v = self.read_tuple3(prim_path, attr_name)?;
                Ok(UsdValue::Float3(v))
            }
            "matrix4d" => Ok(UsdValue::Matrix(self.read_matrix4(prim_path, attr_name)?)),
            "float3[]" | "double3[]" | "normal3f[]" | "point3f[]"
            | "vector3f[]" | "color3f[]" => {
                let arr = self.read_float3_array(prim_path, attr_name)?;
                Ok(UsdValue::Float3Array(arr))
            }
            "float2[]" | "double2[]" | "texCoord2f[]" => {
                let arr = self.read_float2_array(prim_path, attr_name)?;
                Ok(UsdValue::Float2Array(arr))
            }
            "int[]" => {
                let arr = self.read_int_array(prim_path, attr_name)?;
                Ok(UsdValue::IntArray(arr))
            }
            "token[]" | "string[]" => {
                let arr = self.read_string_array(prim_path, attr_name)?;
                Ok(UsdValue::StringArray(arr))
            }
            _ => self.parse_value_inferred(prim_path, attr_name),
        }
    }

    fn parse_bool(&mut self, prim_path: &str, attr_name: &str) -> Result<UsdValue, UsdaError> {
        let tok = self.read_identifier();
        match tok.as_str() {
            "true" | "1" => Ok(UsdValue::Bool(true)),
            "false" | "0" => Ok(UsdValue::Bool(false)),
            _ => Err(self.parse_error(prim_path, attr_name, "invalid bool")),
        }
    }

    fn parse_int(&mut self, prim_path: &str, attr_name: &str) -> Result<UsdValue, UsdaError> {
        let s = self.read_number_str();
        s.parse::<i64>()
            .map(UsdValue::Int)
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid int"))
    }

    fn parse_float(&mut self, prim_path: &str, attr_name: &str) -> Result<UsdValue, UsdaError> {
        let s = self.read_number_str();
        s.parse::<f64>()
            .map(UsdValue::Float)
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float"))
    }

    pub(crate) fn parse_value_inferred(
        &mut self,
        prim_path: &str,
        attr_name: &str,
    ) -> Result<UsdValue, UsdaError> {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => Ok(UsdValue::Token(self.read_string_literal())),
            Some('@') => Ok(UsdValue::String(self.read_asset_literal())),
            Some('(') => {
                let v = self.read_tuple3(prim_path, attr_name)?;
                Ok(UsdValue::Float3(v))
            }
            Some('[') => self.parse_inferred_array(prim_path, attr_name),
            Some(c) if c == '-' || c == '+' || c == '.' || c.is_ascii_digit() => {
                let s = self.read_number_str();
                if s.contains('.') {
                    s.parse::<f64>()
                        .map(UsdValue::Float)
                        .map_err(|_| self.parse_error(prim_path, attr_name, "invalid number"))
                } else {
                    s.parse::<i64>()
                        .map(UsdValue::Int)
                        .map_err(|_| self.parse_error(prim_path, attr_name, "invalid number"))
                }
            }
            Some('t') | Some('f') => {
                let tok = self.read_identifier();
                match tok.as_str() {
                    "true" => Ok(UsdValue::Bool(true)),
                    "false" => Ok(UsdValue::Bool(false)),
                    _ => Ok(UsdValue::Token(tok)),
                }
            }
            _ => {
                self.skip_to_end_of_statement();
                Err(self.parse_error(prim_path, attr_name, "unsupported value type"))
            }
        }
    }

    fn parse_inferred_array(
        &mut self,
        prim_path: &str,
        attr_name: &str,
    ) -> Result<UsdValue, UsdaError> {
        let saved = self.pos;
        self.advance(); // [
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.advance();
            return Ok(UsdValue::IntArray(vec![]));
        }
        if self.peek() == Some('(') {
            self.pos = saved;
            match self.read_float3_array(prim_path, attr_name) {
                Ok(arr) => return Ok(UsdValue::Float3Array(arr)),
                Err(_) => {
                    self.pos = saved;
                    let arr = self.read_float2_array(prim_path, attr_name)?;
                    return Ok(UsdValue::Float2Array(arr));
                }
            }
        }
        if self.peek() == Some('"') {
            self.pos = saved;
            let arr = self.read_string_array(prim_path, attr_name)?;
            return Ok(UsdValue::StringArray(arr));
        }
        self.pos = saved;
        let arr = self.read_int_array(prim_path, attr_name)?;
        Ok(UsdValue::IntArray(arr))
    }
}
