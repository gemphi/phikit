//! The pieces a USD text layer is made of: words, numbers, strings,
//! asset and prim paths, and the marks between them.

/// One piece of a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A name, a keyword or a type: `def`, `float3`, `xformOp:orient`.
    Word(String),
    Number(f64),
    Text(String),
    Asset(String),
    Path(String),
    /// A mark: `( ) { } [ ] = , : ;`.
    Mark(char),
}

/// The pieces of a whole layer, its comments left out.
pub fn split(text: &str) -> Vec<Token> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let (token, next) = match c {
            _ if c.is_whitespace() => (None, i + 1),
            '#' => (None, until(&chars, i, |c| c == '\n')),
            '"' => quoted(&chars, i),
            '\'' => quoted(&chars, i),
            '@' => {
                let end = until(&chars, i + 1, |c| c == '@');
                (Some(Token::Asset(chars[i + 1..end].iter().collect())), end + 1)
            }
            '<' => {
                let end = until(&chars, i + 1, |c| c == '>');
                (Some(Token::Path(chars[i + 1..end].iter().collect())), end + 1)
            }
            '(' | ')' | '{' | '}' | '[' | ']' | '=' | ',' | ':' | ';' => (Some(Token::Mark(c)), i + 1),
            _ if c.is_ascii_digit() || ((c == '-' || c == '.') && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit() || *n == '.')) => {
                let end = until(&chars, i + 1, |c| !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+'));
                let text: String = chars[i..end].iter().collect();
                (Some(text.parse().map(Token::Number).unwrap_or(Token::Word(text))), end)
            }
            _ => {
                let end = until(&chars, i, |c| !(c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-')));
                let end = end.max(i + 1);
                let word: String = chars[i..end].iter().collect();
                // `a:b` is one name; a lone `:` is a mark, as in time samples.
                (Some(Token::Word(word.trim_end_matches(':').to_string())), end - word.len() + word.trim_end_matches(':').len())
            }
        };
        out.extend(token);
        i = next.max(i + 1);
    }
    out
}

/// The first place from `from` where `stop` holds, or the end.
fn until(chars: &[char], from: usize, stop: impl Fn(char) -> bool) -> usize {
    (from..chars.len()).find(|&i| stop(chars[i])).unwrap_or(chars.len())
}

/// A string in quotes, single or triple, with its escapes.
fn quoted(chars: &[char], at: usize) -> (Option<Token>, usize) {
    let q = chars[at];
    let triple = chars.get(at + 1) == Some(&q) && chars.get(at + 2) == Some(&q);
    let (open, mut i) = match triple {
        true => (3, at + 3),
        false => (1, at + 1),
    };
    let mut text = String::new();
    while i < chars.len() {
        let closes = match triple {
            true => chars[i] == q && chars.get(i + 1) == Some(&q) && chars.get(i + 2) == Some(&q),
            false => chars[i] == q,
        };
        if closes {
            return (Some(Token::Text(text)), i + open);
        }
        match chars[i] == '\\' && i + 1 < chars.len() {
            true => {
                text.push(chars[i + 1]);
                i += 2;
            }
            false => {
                text.push(chars[i]);
                i += 1;
            }
        }
    }
    (Some(Token::Text(text)), chars.len())
}
