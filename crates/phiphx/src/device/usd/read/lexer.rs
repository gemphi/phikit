//! Lexer helpers — low-level character consumption, whitespace skipping,
//! identifier and literal reading for the USDA recursive-descent parser.

use super::UsdaParser;

impl<'a> UsdaParser<'a> {
    pub(crate) fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    pub(crate) fn advance(&mut self) {
        if let Some(c) = self.peek() { self.pos += c.len_utf8(); }
    }

    pub(crate) fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() { self.advance(); } else { break; }
        }
    }

    pub(crate) fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek() == Some('#') { self.skip_to_eol(); } else { break; }
        }
    }

    pub(crate) fn skip_to_eol(&mut self) {
        while self.pos < self.input.len() {
            if self.peek() == Some('\n') { self.advance(); break; }
            self.advance();
        }
    }

    pub(crate) fn skip_separator(&mut self) {
        self.skip_whitespace();
        if self.peek() == Some(',') { self.advance(); }
        self.skip_whitespace();
    }

    pub(crate) fn read_identifier(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { self.advance(); } else { break; }
        }
        self.input[start..self.pos].to_string()
    }

    pub(crate) fn peek_identifier(&self) -> String {
        let mut p = self.pos;
        while p < self.input.len() {
            let c = self.input[p..].chars().next().unwrap();
            if c.is_alphanumeric() || c == '_' { p += c.len_utf8(); } else { break; }
        }
        self.input[self.pos..p].to_string()
    }

    pub(crate) fn read_type_name(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' { self.advance(); } else { break; }
        }
        let mut name = self.input[start..self.pos].to_string();
        if self.peek() == Some('[') {
            self.advance();
            if self.peek() == Some(']') { self.advance(); }
            name.push_str("[]");
        }
        name
    }

    pub(crate) fn read_attribute_name(&mut self) -> String {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    pub(crate) fn read_string_literal(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() != Some('"') { return self.read_identifier(); }
        self.advance();
        let start = self.pos;
        while self.pos < self.input.len() {
            match self.peek() {
                Some('\\') => { self.advance(); self.advance(); }
                Some('"') => break,
                _ => self.advance(),
            }
        }
        let val = self.input[start..self.pos].to_string();
        if self.peek() == Some('"') { self.advance(); }
        val
    }

    pub(crate) fn read_string_or_asset_literal(&mut self) -> String {
        self.skip_whitespace();
        if self.peek() == Some('@') { return self.read_asset_literal(); }
        self.read_string_literal()
    }

    pub(crate) fn read_asset_literal(&mut self) -> String {
        if self.peek() != Some('@') { return String::new(); }
        self.advance();
        let start = self.pos;
        while self.pos < self.input.len() {
            if self.peek() == Some('@') { break; }
            self.advance();
        }
        let val = self.input[start..self.pos].to_string();
        if self.peek() == Some('@') { self.advance(); }
        val
    }

    pub(crate) fn read_number_str(&mut self) -> String {
        self.skip_whitespace();
        let start = self.pos;
        if matches!(self.peek(), Some('-') | Some('+')) { self.advance(); }
        while self.pos < self.input.len() {
            match self.peek() {
                Some(c) if c.is_ascii_digit() || c == '.' || c == 'e' || c == 'E'
                    || c == '-' || c == '+' =>
                {
                    if (c == '-' || c == '+') && self.pos > start {
                        let prev = self.input.as_bytes()[self.pos - 1];
                        if prev != b'e' && prev != b'E' { break; }
                    }
                    self.advance();
                }
                _ => break,
            }
        }
        self.input[start..self.pos].to_string()
    }
}
