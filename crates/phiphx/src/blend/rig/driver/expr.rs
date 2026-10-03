//! Blender's simple driver expressions (`BLI_expr_pylike_eval`): the
//! Python-like subset Blender evaluates itself, without Python — numbers,
//! the driver's variables and `frame`, `pi` / `True` / `False`, `+ - * /`,
//! comparisons (chained, as Python chains them), `not` / `and` / `or`,
//! `a if c else b`, and its function table (`abs`, `sin`, `clamp`, `min`,
//! `max`, `lerp`, `smoothstep`, …). Anything else is not a simple
//! expression; Blender would hand it to Python, which a file opened here
//! never runs.

/// A parsed expression.
#[derive(Clone, Debug, PartialEq)]
pub enum BExpr {
    Num(f64),
    Var(usize),
    Neg(Box<BExpr>),
    Not(Box<BExpr>),
    Bin(char, Box<BExpr>, Box<BExpr>),
    /// `a op b op c …`, Python's chained comparison.
    Cmp(Box<BExpr>, Vec<(&'static str, BExpr)>),
    And(Box<BExpr>, Box<BExpr>),
    Or(Box<BExpr>, Box<BExpr>),
    If { body: Box<BExpr>, cond: Box<BExpr>, other: Box<BExpr> },
    Call(String, Vec<BExpr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Op(&'static str),
}

/// The parser over the expression's tokens; `names` are the parameters
/// (`frame`, then the variables).
struct BParse<'a> {
    toks: Vec<Tok>,
    at: usize,
    names: &'a [String],
}

impl BExpr {
    /// Parse `text` against the parameter `names`; `None` when it is not a
    /// simple expression.
    pub fn parse(text: &str, names: &[String]) -> Option<Self> {
        let mut p = BParse { toks: BParse::tokens(text)?, at: 0, names };
        let e = p.expr()?;
        (p.at == p.toks.len()).then_some(e)
    }

    /// Evaluate with the parameters' values; `None` on a math error
    /// (division by zero, a domain error), as Blender flags the driver.
    pub fn eval(&self, v: &[f64]) -> Option<f64> {
        let t = |b: bool| if b { 1.0 } else { 0.0 };
        Some(match self {
            Self::Num(n) => *n,
            Self::Var(i) => *v.get(*i)?,
            Self::Neg(a) => -a.eval(v)?,
            Self::Not(a) => t(a.eval(v)? == 0.0),
            Self::Bin(op, a, b) => {
                let (a, b) = (a.eval(v)?, b.eval(v)?);
                match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    _ if b == 0.0 => return None,
                    _ => a / b,
                }
            }
            Self::Cmp(first, rest) => {
                let mut left = first.eval(v)?;
                for (op, e) in rest {
                    let right = e.eval(v)?;
                    let ok = match *op {
                        "==" => left == right,
                        "!=" => left != right,
                        "<" => left < right,
                        "<=" => left <= right,
                        ">" => left > right,
                        _ => left >= right,
                    };
                    if !ok {
                        return Some(0.0);
                    }
                    left = right;
                }
                1.0
            }
            Self::And(a, b) => {
                let a = a.eval(v)?;
                if a == 0.0 { a } else { b.eval(v)? }
            }
            Self::Or(a, b) => {
                let a = a.eval(v)?;
                if a != 0.0 { a } else { b.eval(v)? }
            }
            Self::If { body, cond, other } => {
                if cond.eval(v)? != 0.0 { body.eval(v)? } else { other.eval(v)? }
            }
            Self::Call(name, args) => {
                let a: Vec<f64> = args.iter().map(|e| e.eval(v)).collect::<Option<_>>()?;
                let r = Self::call(name, &a)?;
                if r.is_finite() { r } else { return None }
            }
        })
    }

    /// Blender's `builtin_ops`, by name and argument count.
    fn call(name: &str, a: &[f64]) -> Option<f64> {
        Some(match (name, a) {
            ("min", [_, ..]) => a.iter().copied().fold(f64::INFINITY, f64::min),
            ("max", [_, ..]) => a.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            ("radians", [x]) => x * std::f64::consts::PI / 180.0,
            ("degrees", [x]) => x * 180.0 / std::f64::consts::PI,
            ("abs" | "fabs", [x]) => x.abs(),
            ("floor", [x]) => x.floor(),
            ("ceil", [x]) => x.ceil(),
            ("trunc" | "int", [x]) => x.trunc(),
            ("round", [x]) => x.round(),
            ("sin", [x]) => x.sin(),
            ("cos", [x]) => x.cos(),
            ("tan", [x]) => x.tan(),
            ("asin", [x]) => x.asin(),
            ("acos", [x]) => x.acos(),
            ("atan", [x]) => x.atan(),
            ("atan2", [y, x]) => y.atan2(*x),
            ("exp", [x]) => x.exp(),
            ("log", [x]) => x.ln(),
            ("log", [x, b]) => x.ln() / b.ln(),
            ("sqrt", [x]) => x.sqrt(),
            ("pow", [x, y]) => x.powf(*y),
            ("fmod", [x, y]) => x % y,
            ("lerp", [a, b, x]) => a * (1.0 - x) + b * x,
            ("clamp", [x]) => x.clamp(0.0, 1.0),
            ("clamp", [x, lo, hi]) => x.max(*lo).min(*hi),
            ("smoothstep", [a, b, x]) => {
                let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            }
            _ => return None,
        })
    }
}

impl BParse<'_> {
    /// Split into numbers, names and operators; `None` on a character the
    /// simple evaluator does not know.
    fn tokens(text: &str) -> Option<Vec<Tok>> {
        let c: Vec<char> = text.chars().collect();
        let (mut i, mut out) = (0, Vec::new());
        const OPS: [&str; 15] = ["==", "!=", "<=", ">=", "<", ">", "+", "-", "*", "/", "(", ")", ",", ".", "="];
        while i < c.len() {
            let ch = c[i];
            if ch.is_whitespace() {
                i += 1;
            } else if ch.is_ascii_digit() || (ch == '.' && c.get(i + 1).is_some_and(char::is_ascii_digit)) {
                let start = i;
                while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.' || ((c[i] == 'e' || c[i] == 'E') && i > start) || ((c[i] == '-' || c[i] == '+') && matches!(c[i - 1], 'e' | 'E'))) {
                    i += 1;
                }
                out.push(Tok::Num(c[start..i].iter().collect::<String>().parse().ok()?));
            } else if ch.is_alphabetic() || ch == '_' {
                let start = i;
                while i < c.len() && (c[i].is_alphanumeric() || c[i] == '_') {
                    i += 1;
                }
                out.push(Tok::Name(c[start..i].iter().collect()));
            } else {
                let two: String = c[i..(i + 2).min(c.len())].iter().collect();
                let op = OPS.iter().find(|o| two.starts_with(**o))?;
                if *op == "=" || *op == "." {
                    return None;
                }
                out.push(Tok::Op(op));
                i += op.len();
            }
        }
        Some(out)
    }

    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.at)
    }

    fn op(&mut self, op: &str) -> bool {
        let hit = matches!(self.peek(), Some(Tok::Op(o)) if *o == op);
        self.at += usize::from(hit);
        hit
    }

    fn word(&mut self, w: &str) -> bool {
        let hit = matches!(self.peek(), Some(Tok::Name(n)) if n == w);
        self.at += usize::from(hit);
        hit
    }

    fn expr(&mut self) -> Option<BExpr> {
        let body = self.or()?;
        if self.word("if") {
            let cond = self.or()?;
            if !self.word("else") {
                return None;
            }
            let other = self.expr()?;
            return Some(BExpr::If { body: Box::new(body), cond: Box::new(cond), other: Box::new(other) });
        }
        Some(body)
    }

    fn or(&mut self) -> Option<BExpr> {
        let a = self.and()?;
        if self.word("or") { Some(BExpr::Or(Box::new(a), Box::new(self.or()?))) } else { Some(a) }
    }

    fn and(&mut self) -> Option<BExpr> {
        let a = self.not()?;
        if self.word("and") { Some(BExpr::And(Box::new(a), Box::new(self.and()?))) } else { Some(a) }
    }

    fn not(&mut self) -> Option<BExpr> {
        if self.word("not") { Some(BExpr::Not(Box::new(self.not()?))) } else { self.cmp() }
    }

    fn cmp(&mut self) -> Option<BExpr> {
        let first = self.add()?;
        let mut rest = Vec::new();
        while let Some(op) = ["==", "!=", "<=", ">=", "<", ">"].into_iter().find(|o| matches!(self.peek(), Some(Tok::Op(t)) if t == o)) {
            self.at += 1;
            rest.push((op, self.add()?));
        }
        if rest.is_empty() { Some(first) } else { Some(BExpr::Cmp(Box::new(first), rest)) }
    }

    fn add(&mut self) -> Option<BExpr> {
        let mut a = self.mul()?;
        loop {
            let op = if self.op("+") { '+' } else if self.op("-") { '-' } else { return Some(a) };
            a = BExpr::Bin(op, Box::new(a), Box::new(self.mul()?));
        }
    }

    fn mul(&mut self) -> Option<BExpr> {
        let mut a = self.unary()?;
        loop {
            let op = if self.op("*") { '*' } else if self.op("/") { '/' } else { return Some(a) };
            a = BExpr::Bin(op, Box::new(a), Box::new(self.unary()?));
        }
    }

    fn unary(&mut self) -> Option<BExpr> {
        if self.op("-") {
            return Some(BExpr::Neg(Box::new(self.unary()?)));
        }
        if self.op("+") {
            return self.unary();
        }
        if self.op("(") {
            let e = self.expr()?;
            return self.op(")").then_some(e);
        }
        match self.peek()?.clone() {
            Tok::Num(n) => {
                self.at += 1;
                Some(BExpr::Num(n))
            }
            Tok::Name(name) => {
                self.at += 1;
                if self.op("(") {
                    let mut args = Vec::new();
                    if !self.op(")") {
                        loop {
                            args.push(self.expr()?);
                            if self.op(")") {
                                break;
                            }
                            if !self.op(",") {
                                return None;
                            }
                        }
                    }
                    return Some(BExpr::Call(name, args));
                }
                if let Some(i) = self.names.iter().position(|n| *n == name) {
                    return Some(BExpr::Var(i));
                }
                match name.as_str() {
                    "pi" => Some(BExpr::Num(std::f64::consts::PI)),
                    "True" => Some(BExpr::Num(1.0)),
                    "False" => Some(BExpr::Num(0.0)),
                    _ => None,
                }
            }
            Tok::Op(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(text: &str, vars: &[(&str, f64)]) -> Option<f64> {
        let names: Vec<String> = std::iter::once("frame".to_string()).chain(vars.iter().map(|v| v.0.to_string())).collect();
        let values: Vec<f64> = std::iter::once(1.0).chain(vars.iter().map(|v| v.1)).collect();
        BExpr::parse(text, &names)?.eval(&values)
    }

    #[test]
    fn rigify_expressions_evaluate_as_python_would() {
        assert_eq!(run("var == 1", &[("var", 1.0)]), Some(1.0));
        assert_eq!(run("var == 2", &[("var", 1.0)]), Some(0.0));
        let v = run("(1-sy)*pi", &[("sy", 0.25)]).unwrap();
        assert!((v - 0.75 * std::f64::consts::PI).abs() < 1e-12);
        assert_eq!(run("1 < var < 3", &[("var", 2.0)]), Some(1.0));
        assert_eq!(run("1 < var < 3", &[("var", 3.0)]), Some(0.0));
        assert_eq!(run("0 or var and 5", &[("var", 2.0)]), Some(5.0));
        assert_eq!(run("2 if var > 1 else -3", &[("var", 0.0)]), Some(-3.0));
        assert_eq!(run("max(1, var, 0.5) + clamp(var, 0, 1.5)", &[("var", 2.0)]), Some(3.5));
        assert_eq!(run("1 / var", &[("var", 0.0)]), None, "division by zero flags the driver");
        assert!(BExpr::parse("var ** 2", &["var".into()]).is_none(), "not a simple expression");
        assert!(BExpr::parse("bpy.data", &[]).is_none());
    }
}
