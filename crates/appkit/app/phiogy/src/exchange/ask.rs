//! An ask: one request as a page sends it, read off the wire. Plain
//! HTTP, as appkit's own `PCall` speaks it from the other side: a first
//! line, the head, and a body as long as the head says.

use std::io::Read;

use crate::comp::Fault;

/// The names of the head's lines the exchange reads.
pub const ORIGIN: &str = "origin";
pub const COOKIE: &str = "cookie";
pub const LENGTH: &str = "content-length";
/// How many times the most a request may be is read of one that is
/// too large, before it is refused.
const DRAIN: usize = 16;
/// What ends the head.
const END: &[u8] = b"\r\n\r\n";

/// One request.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ask {
    pub method: String,
    pub path: String,
    query: String,
    head: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Ask {
    /// Read a request no longer than `most` bytes. `Err(None)` when the
    /// wire closed or what came is no request, and there is nobody to
    /// answer; `Err` with a fault when there is.
    pub fn read(wire: &mut impl Read, most: usize) -> Result<Ask, Option<Fault>> {
        let (mut got, mut piece) = (Vec::new(), [0u8; 2048]);
        let cut = loop {
            if let Some(at) = got.windows(END.len()).position(|window| window == END) {
                break at;
            }
            if got.len() > most {
                return Err(Some(Fault::Large));
            }
            match wire.read(&mut piece) {
                Ok(0) | Err(_) => return Err(None),
                Ok(count) => got.extend_from_slice(&piece[..count]),
            }
        };
        let text = String::from_utf8_lossy(&got[..cut]).to_string();
        let mut line = text.split("\r\n");
        let mut first = line.next().unwrap_or_default().split_whitespace();
        let (method, target) = (first.next().ok_or(None)?, first.next().ok_or(None)?);
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let head: Vec<(String, String)> = line.filter_map(|line| line.split_once(':')).map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string())).collect();
        let mut ask = Ask { method: method.to_string(), path: path.to_string(), query: query.to_string(), head, body: got[cut + END.len()..].to_vec() };
        let length = ask.head(LENGTH).parse::<usize>().unwrap_or_default();
        if length > most {
            // It is read to its end and dropped, so whoever sent it is
            // still there to hear that it was too much.
            let mut left = length.saturating_sub(ask.body.len()).min(most.saturating_mul(DRAIN));
            while left > 0 {
                match wire.read(&mut piece) {
                    Ok(0) | Err(_) => break,
                    Ok(count) => left = left.saturating_sub(count),
                }
            }
            return Err(Some(Fault::Large));
        }
        while ask.body.len() < length {
            match wire.read(&mut piece) {
                Ok(0) | Err(_) => return Err(None),
                Ok(count) => ask.body.extend_from_slice(&piece[..count]),
            }
        }
        ask.body.truncate(length);
        Ok(ask)
    }

    /// A line of the head, by its name in small letters; nothing for one
    /// it does not have.
    pub fn head(&self, name: &str) -> &str {
        self.head.iter().find(|(known, _)| known == name).map_or("", |(_, value)| value.as_str())
    }

    /// The value of a cookie.
    pub fn cookie(&self, name: &str) -> Option<&str> {
        let pair = self.head(COOKIE).split(';').filter_map(|pair| pair.trim().split_once('='));
        pair.into_iter().find(|(known, _)| *known == name).map(|(_, value)| value)
    }

    /// What the path's query gives under a name; nothing when it gives
    /// none.
    pub fn query(&self, name: &str) -> &str {
        let pair = self.query.split('&').filter_map(|pair| pair.split_once('='));
        pair.into_iter().find(|(known, _)| *known == name).map_or("", |(_, value)| value)
    }

    /// The parts of the path, between its strokes.
    pub fn part(&self) -> Vec<&str> {
        self.path.split('/').skip(1).collect()
    }
}
