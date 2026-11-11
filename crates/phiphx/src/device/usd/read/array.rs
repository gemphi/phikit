//! Array and tuple readers — parse `(x, y, z)` tuples and `[...]` array
//! literals into typed Rust vectors for the USDA parser.

use super::super::error::UsdaError;
use super::UsdaParser;

impl<'a> UsdaParser<'a> {
    pub(crate) fn read_tuple3(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<[f32; 3], UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('(') {
            return Err(self.parse_error(prim_path, attr_name, "expected '('"));
        }
        self.advance();
        self.skip_whitespace();
        let x = self.read_number_str();
        self.skip_separator();
        let y = self.read_number_str();
        self.skip_separator();
        let z = self.read_number_str();
        self.skip_whitespace();
        if self.peek() == Some(')') { self.advance(); }
        let xf = x.parse::<f32>()
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float3 x component"))?;
        let yf = y.parse::<f32>()
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float3 y component"))?;
        let zf = z.parse::<f32>()
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float3 z component"))?;
        Ok([xf, yf, zf])
    }

    pub(crate) fn read_float3_array(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<Vec<[f32; 3]>, UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('[') {
            return Err(self.parse_error(prim_path, attr_name, "expected '['"));
        }
        self.advance();
        let mut result = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') { self.advance(); break; }
            if self.pos >= self.input.len() { break; }
            result.push(self.read_tuple3(prim_path, attr_name)?);
            self.skip_separator();
        }
        Ok(result)
    }

    pub(crate) fn read_float2_array(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<Vec<[f32; 2]>, UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('[') {
            return Err(self.parse_error(prim_path, attr_name, "expected '['"));
        }
        self.advance();
        let mut result = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') { self.advance(); break; }
            if self.pos >= self.input.len() { break; }
            result.push(self.read_tuple2(prim_path, attr_name)?);
            self.skip_separator();
        }
        Ok(result)
    }

    fn read_tuple2(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<[f32; 2], UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('(') {
            return Err(self.parse_error(prim_path, attr_name, "expected '('"));
        }
        self.advance();
        self.skip_whitespace();
        let x = self.read_number_str();
        self.skip_separator();
        let y = self.read_number_str();
        self.skip_whitespace();
        if self.peek() == Some(')') { self.advance(); }
        let xf = x.parse::<f32>()
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float2 x component"))?;
        let yf = y.parse::<f32>()
            .map_err(|_| self.parse_error(prim_path, attr_name, "invalid float2 y component"))?;
        Ok([xf, yf])
    }

    pub(crate) fn read_int_array(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<Vec<i32>, UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('[') {
            return Err(self.parse_error(prim_path, attr_name, "expected '['"));
        }
        self.advance();
        let mut result = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') { self.advance(); break; }
            if self.pos >= self.input.len() { break; }
            let s = self.read_number_str();
            if s.is_empty() { self.skip_separator(); continue; }
            let v = s.parse::<i32>()
                .map_err(|_| self.parse_error(prim_path, attr_name, "invalid int in array"))?;
            result.push(v);
            self.skip_separator();
        }
        Ok(result)
    }

    pub(crate) fn read_string_array(
        &mut self, prim_path: &str, _attr_name: &str,
    ) -> Result<Vec<String>, UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('[') {
            return Err(self.parse_error(prim_path, _attr_name, "expected '['"));
        }
        self.advance();
        let mut result = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some(']') { self.advance(); break; }
            if self.pos >= self.input.len() { break; }
            let s = self.read_string_literal();
            result.push(s);
            self.skip_separator();
        }
        Ok(result)
    }
}

impl<'a> UsdaParser<'a> {
    /// A matrix4d: four rows of four numbers, ( (a, b, c, d), ... ).
    pub(crate) fn read_matrix4(
        &mut self, prim_path: &str, attr_name: &str,
    ) -> Result<[f64; 16], UsdaError> {
        self.skip_whitespace();
        if self.peek() != Some('(') {
            return Err(self.parse_error(prim_path, attr_name, "expected '('"));
        }
        self.advance();
        let mut out = [0.0f64; 16];
        for slot in out.iter_mut() {
            self.skip_whitespace();
            while matches!(self.peek(), Some('(') | Some(',')) {
                self.advance();
                self.skip_whitespace();
            }
            let n = self.read_number_str();
            *slot = n
                .parse::<f64>()
                .map_err(|_| self.parse_error(prim_path, attr_name, "invalid matrix4d entry"))?;
        }
        self.skip_whitespace();
        while matches!(self.peek(), Some(')') | Some(',')) {
            self.advance();
            self.skip_whitespace();
        }
        Ok(out)
    }
}
