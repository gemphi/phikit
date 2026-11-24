//! The outlines of a drawing: every `d` of an svg file's paths, read as
//! closed rings of points (y turned up, as a font's is). Moves, lines
//! (straight, across, down) and arcs are read, in place or relative;
//! an arc is cut into `step` pieces a quarter turn. Curves of other
//! kinds are not read (the house's drawings have none).

use std::f32::consts::{FRAC_PI_2, TAU};

/// Every closed ring of every path in the file.
pub fn rings(file: &str, step: usize) -> Vec<Vec<[f32; 2]>> {
    let mut out = Vec::new();
    for (at, _) in file.match_indices(" d=\"") {
        let data = file[at + 4..].split('"').next().unwrap_or_default();
        out.extend(path(data, step));
    }
    out.into_iter().map(|ring| ring.into_iter().map(|[x, y]| [x, -y]).collect()).filter(|r: &Vec<[f32; 2]>| r.len() >= 3).collect()
}

/// One path's rings.
fn path(data: &str, step: usize) -> Vec<Vec<[f32; 2]>> {
    let token = tokens(data);
    let (mut out, mut ring): (Vec<Vec<[f32; 2]>>, Vec<[f32; 2]>) = (Vec::new(), Vec::new());
    let (mut at, mut command, mut k) = ([0.0f32; 2], 'M', 0usize);
    let number = |k: &mut usize| -> f32 {
        let value = match token.get(*k) {
            Some(Token::Number(v)) => *v,
            _ => 0.0,
        };
        *k += 1;
        value
    };
    while k < token.len() {
        if let Token::Command(c) = token[k] {
            command = c;
            k += 1;
        }
        let relative = command.is_ascii_lowercase();
        let base = match relative {
            true => at,
            false => [0.0, 0.0],
        };
        match command.to_ascii_uppercase() {
            'Z' => {
                // A closed path's pen goes back to where it began.
                at = ring.first().copied().unwrap_or(at);
                out.push(std::mem::take(&mut ring));
                if !matches!(token.get(k), Some(Token::Command(_))) {
                    k += 1;
                }
                continue;
            }
            'M' => {
                if !ring.is_empty() {
                    out.push(std::mem::take(&mut ring));
                }
                at = [base[0] + number(&mut k), base[1] + number(&mut k)];
                ring.push(at);
                command = match relative {
                    true => 'l',
                    false => 'L',
                };
            }
            'L' => at = [base[0] + number(&mut k), base[1] + number(&mut k)],
            'H' => at = [base[0] + number(&mut k), at[1]],
            'V' => at = [at[0], base[1] + number(&mut k)],
            'A' => {
                let (rx, ry, turn, large, sweep) = (number(&mut k), number(&mut k), number(&mut k), number(&mut k) != 0.0, number(&mut k) != 0.0);
                let end = [base[0] + number(&mut k), base[1] + number(&mut k)];
                ring.extend(arc(at, [rx, ry], turn, large, sweep, end, step));
                at = end;
                continue;
            }
            _ => {
                k += 1;
                continue;
            }
        }
        if !command.eq_ignore_ascii_case(&'M') && ring.last() != Some(&at) {
            ring.push(at);
        }
    }
    if !ring.is_empty() {
        out.push(ring);
    }
    out.into_iter().map(|mut r| {
        if r.len() > 1 && r.first() == r.last() {
            r.pop();
        }
        r
    }).collect()
}

/// A path's words: its commands and its numbers.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Command(char),
    Number(f32),
}

fn tokens(data: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut Vec<Token>| {
        if let Ok(v) = word.parse() {
            out.push(Token::Number(v));
        }
        word.clear();
    };
    for c in data.chars() {
        match c {
            c if c.is_ascii_alphabetic() && c != 'e' && c != 'E' => {
                flush(&mut word, &mut out);
                out.push(Token::Command(c));
            }
            '-' if !word.ends_with(['e', 'E']) => {
                flush(&mut word, &mut out);
                word.push(c);
            }
            '.' if word.contains('.') => {
                flush(&mut word, &mut out);
                word.push(c);
            }
            ' ' | ',' | '\n' | '\t' | '\r' => flush(&mut word, &mut out),
            c => word.push(c),
        }
    }
    flush(&mut word, &mut out);
    out
}

/// The points of an svg arc after its start, to its end (the spec's
/// endpoint-to-centre conversion, appendix B.2.4).
fn arc(from: [f32; 2], radius: [f32; 2], turn: f32, large: bool, sweep: bool, to: [f32; 2], step: usize) -> Vec<[f32; 2]> {
    let (mut rx, mut ry) = (radius[0].abs(), radius[1].abs());
    if rx == 0.0 || ry == 0.0 || from == to {
        return vec![to];
    }
    let (s, c) = turn.to_radians().sin_cos();
    let (dx, dy) = ((from[0] - to[0]) / 2.0, (from[1] - to[1]) / 2.0);
    let (x1, y1) = (c * dx + s * dy, -s * dx + c * dy);
    let lambda = x1 * x1 / (rx * rx) + y1 * y1 / (ry * ry);
    if lambda > 1.0 {
        (rx, ry) = (rx * lambda.sqrt(), ry * lambda.sqrt());
    }
    let num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let sign = match large == sweep {
        true => -1.0,
        false => 1.0,
    };
    let coef = sign * (num / den).max(0.0).sqrt();
    let (cx1, cy1) = (coef * rx * y1 / ry, -coef * ry * x1 / rx);
    let centre = [c * cx1 - s * cy1 + (from[0] + to[0]) / 2.0, s * cx1 + c * cy1 + (from[1] + to[1]) / 2.0];
    let angle = |u: [f32; 2], v: [f32; 2]| (u[0] * v[1] - u[1] * v[0]).atan2(u[0] * v[0] + u[1] * v[1]);
    let start = angle([1.0, 0.0], [(x1 - cx1) / rx, (y1 - cy1) / ry]);
    let mut span = angle([(x1 - cx1) / rx, (y1 - cy1) / ry], [(-x1 - cx1) / rx, (-y1 - cy1) / ry]);
    span = match (sweep, span) {
        (false, d) if d > 0.0 => d - TAU,
        (true, d) if d < 0.0 => d + TAU,
        (_, d) => d,
    };
    let n = ((span.abs() / FRAC_PI_2) * step as f32).ceil().max(1.0) as usize;
    (1..=n)
        .map(|k| match k == n {
            true => to,
            false => {
                let t = start + span * k as f32 / n as f32;
                let (ts, tc) = t.sin_cos();
                [centre[0] + rx * tc * c - ry * ts * s, centre[1] + rx * tc * s + ry * ts * c]
            }
        })
        .collect()
}
