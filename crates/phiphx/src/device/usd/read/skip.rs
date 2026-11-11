//! Skip helpers — balanced-delimiter skipping and statement-boundary detection
//! for the USDA recursive-descent parser.

use super::UsdaParser;

impl<'a> UsdaParser<'a> {
    pub(crate) fn skip_value(&mut self) {
        self.skip_whitespace();
        match self.peek() {
            Some('"') => { self.read_string_literal(); }
            Some('@') => { self.read_asset_literal(); }
            Some('[') => { self.skip_balanced_brackets(); }
            Some('(') => { self.advance(); self.skip_balanced_parens(); }
            Some('{') => { self.skip_balanced_braces(); }
            _ => { self.skip_to_eol(); }
        }
    }

    pub(crate) fn skip_to_end_of_statement(&mut self) {
        let mut depth_paren = 0i32;
        let mut depth_bracket = 0i32;
        let mut in_string = false;
        while self.pos < self.input.len() {
            match self.peek() {
                Some('\\') if in_string => {
                    self.advance();
                    self.advance();
                }
                Some('"') => {
                    in_string = !in_string;
                    self.advance();
                }
                Some('(') if !in_string => {
                    depth_paren += 1;
                    self.advance();
                }
                Some(')') if !in_string => {
                    if depth_paren > 0 {
                        depth_paren -= 1;
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some('[') if !in_string => {
                    depth_bracket += 1;
                    self.advance();
                }
                Some(']') if !in_string => {
                    if depth_bracket > 0 {
                        depth_bracket -= 1;
                        self.advance();
                    } else {
                        break;
                    }
                }
                Some('\n') if !in_string && depth_paren == 0
                    && depth_bracket == 0 =>
                {
                    self.advance();
                    break;
                }
                Some('}') if !in_string && depth_paren == 0
                    && depth_bracket == 0 =>
                {
                    break;
                }
                _ => self.advance(),
            }
        }
    }

    pub(crate) fn skip_balanced_parens(&mut self) {
        let mut depth = 1i32;
        while self.pos < self.input.len() && depth > 0 {
            match self.peek() {
                Some('(') => { depth += 1; self.advance(); }
                Some(')') => { depth -= 1; self.advance(); }
                Some('"') => { self.advance(); self.skip_string_body(); }
                _ => self.advance(),
            }
        }
    }

    pub(crate) fn skip_balanced_brackets(&mut self) {
        if self.peek() == Some('[') {
            self.advance();
        }
        let mut depth = 1i32;
        while self.pos < self.input.len() && depth > 0 {
            match self.peek() {
                Some('[') => { depth += 1; self.advance(); }
                Some(']') => { depth -= 1; self.advance(); }
                Some('"') => { self.advance(); self.skip_string_body(); }
                _ => self.advance(),
            }
        }
    }

    fn skip_balanced_braces(&mut self) {
        if self.peek() == Some('{') {
            self.advance();
        }
        let mut depth = 1i32;
        while self.pos < self.input.len() && depth > 0 {
            match self.peek() {
                Some('{') => { depth += 1; self.advance(); }
                Some('}') => { depth -= 1; self.advance(); }
                Some('"') => { self.advance(); self.skip_string_body(); }
                _ => self.advance(),
            }
        }
    }

    fn skip_string_body(&mut self) {
        while self.pos < self.input.len() {
            match self.peek() {
                Some('\\') => { self.advance(); self.advance(); }
                Some('"') => { self.advance(); break; }
                _ => self.advance(),
            }
        }
    }
}
