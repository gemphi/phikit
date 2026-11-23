//! Reading a layer's pieces into prims: metadata, values, prims and
//! their bodies. What the reader does not know is stepped over whole.

use super::layer::{Prim, Property, Spec};
use super::token::Token;
use super::value::Value;

/// Reads a layer's pieces in order.
pub(super) struct Reader {
    token: Vec<Token>,
    at: usize,
}

impl Reader {
    pub(super) fn new(token: Vec<Token>) -> Reader {
        Reader { token, at: 0 }
    }

    /// Whether every piece is read.
    pub(super) fn done(&self) -> bool {
        self.at >= self.token.len()
    }

    /// Step over one piece.
    pub(super) fn step(&mut self) {
        self.at += 1;
    }

    fn peek(&self) -> Option<&Token> {
        self.token.get(self.at)
    }

    fn mark(&self, c: char) -> bool {
        self.peek() == Some(&Token::Mark(c))
    }

    fn word(&self) -> Option<String> {
        match self.peek() {
            Some(Token::Word(w)) => Some(w.clone()),
            _ => None,
        }
    }

    /// Step over a balanced group that opens here.
    fn skip(&mut self) {
        let open = match self.peek() {
            Some(Token::Mark(c)) if matches!(c, '(' | '{' | '[') => *c,
            _ => {
                self.at += 1;
                return;
            }
        };
        let close = match open {
            '(' => ')',
            '{' => '}',
            _ => ']',
        };
        let mut depth = 0;
        while let Some(t) = self.peek().cloned() {
            self.at += 1;
            match t {
                Token::Mark(c) if c == open => depth += 1,
                Token::Mark(c) if c == close => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                return;
            }
        }
    }

    /// Metadata in parentheses, where it opens here: `key = value`, with
    /// `prepend`, `append`, `add` or `delete` before a key kept with it.
    pub(super) fn meta(&mut self) -> Vec<(String, Value)> {
        let mut out = Vec::new();
        if !self.mark('(') {
            return out;
        }
        self.at += 1;
        while !self.done() && !self.mark(')') {
            match self.peek().cloned() {
                Some(Token::Word(w)) => {
                    self.at += 1;
                    let (key, edit) = match matches!(w.as_str(), "prepend" | "append" | "add" | "delete" | "reorder") {
                        true => (self.word().unwrap_or_default(), true),
                        false => (w, false),
                    };
                    self.at += edit as usize;
                    // A typed entry, as in customData: `float3 hint = (..)`.
                    let key = match (self.word(), self.mark('=')) {
                        (Some(name), false) => {
                            self.at += 1;
                            name
                        }
                        _ => key,
                    };
                    if self.mark('=') {
                        self.at += 1;
                        out.push((key, self.value()));
                    }
                }
                Some(Token::Text(doc)) => {
                    self.at += 1;
                    out.push(("doc".to_string(), Value::Text(doc)));
                }
                _ => self.at += 1,
            }
        }
        self.at += 1;
        out
    }

    /// A value: a number, text, asset, path, tuple, list or dictionary.
    fn value(&mut self) -> Value {
        let Some(t) = self.peek().cloned() else { return Value::Nothing };
        self.at += 1;
        match t {
            Token::Number(n) => Value::Number(n),
            Token::Text(s) => Value::Text(s),
            Token::Path(p) => Value::Path(p),
            Token::Asset(a) => match self.peek().cloned() {
                Some(Token::Path(p)) => {
                    self.at += 1;
                    Value::Asset(a, p)
                }
                _ => Value::Asset(a, String::new()),
            },
            Token::Word(w) => match w.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                "None" => Value::Nothing,
                _ => Value::Text(w),
            },
            Token::Mark('(') => Value::Tuple(self.items(')')),
            Token::Mark('[') => Value::List(self.items(']')),
            Token::Mark('{') => self.dict(),
            Token::Mark(_) => Value::Nothing,
        }
    }

    /// The items of a tuple or list up to `close`.
    fn items(&mut self, close: char) -> Vec<Value> {
        let mut out = Vec::new();
        while !self.done() && !self.mark(close) {
            match self.mark(',') {
                true => self.at += 1,
                false => out.push(self.value()),
            }
        }
        self.at += 1;
        out
    }

    /// A dictionary or time samples, up to its `}`: `key = value`,
    /// `type key = value` or `time: value`.
    fn dict(&mut self) -> Value {
        let mut out = Vec::new();
        while !self.done() && !self.mark('}') {
            let key = match self.peek().cloned() {
                Some(Token::Number(n)) => Some(n.to_string()),
                Some(Token::Text(s)) => Some(s),
                Some(Token::Word(w)) => Some(match self.token.get(self.at + 1) {
                    Some(Token::Word(name)) => {
                        self.at += 1;
                        name.clone()
                    }
                    _ => w,
                }),
                _ => None,
            };
            self.at += 1;
            let key = key.filter(|_| self.mark(':') || self.mark('='));
            key.into_iter().for_each(|key| {
                self.at += 1;
                out.push((key, self.value()));
            });
            if self.mark(',') {
                self.at += 1;
            }
        }
        self.at += 1;
        Value::Dict(out)
    }

    /// A prim, where one begins here.
    pub(super) fn prim(&mut self) -> Option<Prim> {
        let spec = match self.word()?.as_str() {
            "def" => Spec::Def,
            "over" => Spec::Over,
            "class" => Spec::Class,
            _ => return None,
        };
        self.at += 1;
        let kind = self.word().inspect(|_| self.at += 1).unwrap_or_default();
        let name = match self.peek().cloned() {
            Some(Token::Text(name)) => {
                self.at += 1;
                name
            }
            _ => return None,
        };
        let meta = self.meta();
        let mut prim = Prim { spec, kind, name, meta, ..Prim::default() };
        if self.mark('{') {
            self.at += 1;
            self.body(&mut prim);
        }
        Some(prim)
    }

    /// What is in a prim's braces, up to its `}`.
    fn body(&mut self, prim: &mut Prim) {
        while !self.done() && !self.mark('}') {
            if let Some(child) = self.prim() {
                prim.child.push(child);
                continue;
            }
            match self.word().as_deref() {
                // `reorder nameChildren = [..]`, a variant set: stepped over.
                Some("reorder") | Some("variantSet") => {
                    while !self.done() && !self.mark('=') {
                        self.at += 1;
                    }
                    self.at += 1;
                    self.skip();
                }
                Some(_) => self.property(prim),
                None => self.skip(),
            }
        }
        self.at += 1;
    }

    /// An attribute or a relationship: words up to its name, then its
    /// value and metadata where it has them.
    fn property(&mut self, prim: &mut Prim) {
        let mut word = Vec::new();
        while let Some(w) = self.word() {
            self.at += 1;
            if self.mark('[') && self.token.get(self.at + 1) == Some(&Token::Mark(']')) {
                self.at += 2;
            }
            word.push(w);
            if self.mark('=') || self.mark('(') || self.mark('}') || self.word().is_some_and(|w| matches!(w.as_str(), "def" | "over" | "class")) {
                break;
            }
        }
        let name = word.pop().unwrap_or_default();
        let kind = word.into_iter().filter(|w| !matches!(w.as_str(), "uniform" | "custom" | "varying" | "prepend" | "append" | "add" | "delete")).collect::<Vec<_>>().join(" ");
        let value = match self.mark('=') {
            true => {
                self.at += 1;
                self.value()
            }
            false => Value::Nothing,
        };
        self.meta();
        prim.property.push(Property { name, kind, value });
    }
}
