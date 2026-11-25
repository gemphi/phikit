//! A place over the network, reached as a page reaches it: the app's
//! server (phiegg.com), Phiora on it, the line calls are on, or a line
//! on this machine. A place is written as an address
//! (`localhost:4310`, `https://phiegg.com/wrt`): plain or secured, a
//! name, a port, and the path everything there is asked under. A
//! secured place is reached over TLS, trusting what the system trusts.
//! Its words are `data/net.tsv`.

mod tls;

use std::io::{BufRead, BufReader, Error, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use appkit::service::PTable;

use crate::data;

/// Where something is over the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub secure: bool,
    pub host: String,
    pub port: u16,
    /// The path everything there is asked under: `/wrt`, or none.
    pub base: String,
}

impl Place {
    /// A place as it is written: an address, with or without its
    /// scheme, its port and its path.
    pub fn read(text: &str) -> Place {
        let word = PTable::read(data::NET);
        let text = text.trim();
        let (secure, rest) = match (text.strip_prefix(word.get("secure")), text.strip_prefix(word.get("plain"))) {
            (Some(rest), _) => (true, rest),
            (None, Some(rest)) => (false, rest),
            (None, None) => (false, text),
        };
        let (authority, base) = rest.split_once('/').map_or((rest, ""), |(authority, base)| (authority, base.trim_end_matches('/')));
        let base = match base.is_empty() {
            true => String::new(),
            false => format!("/{base}"),
        };
        let own = match secure {
            true => word.get("secure.port"),
            false => word.get("plain.port"),
        };
        let (host, port) = authority.rsplit_once(':').and_then(|(host, port)| Some((host.to_string(), port.parse().ok()?))).unwrap_or_else(|| (authority.to_string(), own.parse().unwrap_or_default()));
        Place { secure, host, port, base }
    }

    /// Its name as an ask's head gives it: with its port where that is
    /// not its scheme's own.
    pub fn authority(&self) -> String {
        let word = PTable::read(data::NET);
        let own = match self.secure {
            true => word.get("secure.port"),
            false => word.get("plain.port"),
        };
        match self.port.to_string() == own {
            true => self.host.clone(),
            false => format!("{}:{}", self.host, self.port),
        }
    }

    /// The place written whole, as an address begins.
    pub fn address(&self) -> String {
        let word = PTable::read(data::NET);
        let scheme = match self.secure {
            true => word.get("secure"),
            false => word.get("plain"),
        };
        format!("{scheme}{}{}", self.authority(), self.base)
    }

    /// Whether it is on this machine.
    pub fn near(&self) -> bool {
        PTable::read(data::NET).get("near").split_whitespace().any(|name| name == self.host)
    }

    /// How long it is waited for, where a place here would be waited for
    /// so long: a place over the internet at least `far`.
    pub fn patience(&self, near: Duration) -> Duration {
        let far = Duration::from_millis(PTable::read(data::NET).get("far").parse().unwrap_or_default());
        match self.near() {
            true => near,
            false => near.max(far),
        }
    }
}

/// A wire to a place: the socket, and over it TLS where the place is
/// secured. What is written and read goes through the TLS.
pub struct Wire {
    socket: TcpStream,
    secure: Option<tls::Secure>,
}

impl Wire {
    /// The socket under it, to wait on or close from another thread.
    pub fn socket(&self) -> std::io::Result<TcpStream> {
        self.socket.try_clone()
    }

    /// How long a read waits, on every socket it holds: on Windows a
    /// socket's copy keeps a wait of its own, so the one under the TLS
    /// is told too.
    pub fn set_read_timeout(&self, wait: Option<Duration>) -> std::io::Result<()> {
        self.socket.set_read_timeout(wait)?;
        self.secure.as_ref().map_or(Ok(()), |secure| tls::socket(secure).set_read_timeout(wait))
    }
}

impl Read for Wire {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self.secure.as_mut() {
            Some(secure) => secure.read(buf),
            None => self.socket.read(buf),
        }
    }
}

impl Write for Wire {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self.secure.as_mut() {
            Some(secure) => secure.write(buf),
            None => self.socket.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self.secure.as_mut() {
            Some(secure) => secure.flush(),
            None => self.socket.flush(),
        }
    }
}

/// Reach a place, waiting no longer than `wait` for it and for each
/// answer.
pub fn reach(place: &Place, wait: Duration) -> std::io::Result<Wire> {
    let mut last = Error::other(place.address());
    for address in (place.host.as_str(), place.port).to_socket_addrs()? {
        match TcpStream::connect_timeout(&address, wait) {
            Ok(socket) => {
                socket.set_read_timeout(Some(wait))?;
                socket.set_nodelay(true)?;
                let secure = match place.secure {
                    true => Some(tls::over(&place.host, socket.try_clone()?)?),
                    false => None,
                };
                return Ok(Wire { socket, secure });
            }
            Err(error) => last = error,
        }
    }
    Err(last)
}

/// Read the head of an answer: its status, and its lines.
pub fn head(reader: &mut impl BufRead) -> std::io::Result<(String, Vec<String>)> {
    let mut line = Vec::new();
    loop {
        let mut text = String::new();
        if reader.read_line(&mut text)? == 0 || text.trim().is_empty() {
            break;
        }
        line.push(text.trim().to_string());
    }
    let status = line.first().and_then(|first| first.split_whitespace().nth(1)).unwrap_or_default().to_string();
    Ok((status, line))
}

/// Whether something is written as an address over the network, not as
/// a path on this disk.
pub fn far(written: &str) -> bool {
    let word = PTable::read(data::NET);
    written.starts_with(word.get("secure")) || written.starts_with(word.get("plain"))
}

/// What a line of a head says under a name, in small letters.
pub fn named(said: &[String], name: &str) -> Option<String> {
    let found = said.iter().filter_map(|line| line.split_once(':')).find(|(known, _)| known.trim().eq_ignore_ascii_case(name));
    found.map(|(_, value)| value.trim().to_ascii_lowercase())
}

/// The body of an answer whose head was read: as long as the head says,
/// in pieces where it says so, else to the wire's end.
pub fn body(reader: &mut impl BufRead, said: &[String]) -> Option<Vec<u8>> {
    let word = PTable::read(data::NET);
    let length = named(said, word.get("length")).and_then(|length| length.parse::<usize>().ok());
    match (named(said, word.get("pieces")), length) {
        (Some(how), _) if how.contains(word.get("piece")) => pieces(reader),
        (_, Some(length)) => {
            let mut body = vec![0; length];
            reader.read_exact(&mut body).ok().map(|_| body)
        }
        _ => {
            let mut body = Vec::new();
            // A secured place may end without saying so: what came is kept.
            _ = reader.read_to_end(&mut body);
            Some(body)
        }
    }
}

/// A body that comes in pieces: each is its length on a line, in
/// sixteens, then that many bytes; a piece of no length ends it.
fn pieces(reader: &mut impl BufRead) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        let size = usize::from_str_radix(line.split(';').next()?.trim(), 16).ok()?;
        if size == 0 {
            return Some(body);
        }
        let from = body.len();
        body.resize(from + size, 0);
        reader.read_exact(&mut body[from..]).ok()?;
        // The end of the piece's own line.
        reader.read_line(&mut line).ok()?;
    }
}

/// A file read whole from an address (`https://phiegg.com/cdn/...`):
/// nothing when it is not there, refused, or slower than it is waited
/// for (`wait`, `far`).
pub fn fetch(address: &str) -> Option<Vec<u8>> {
    let word = PTable::read(data::NET);
    let place = Place::read(address);
    let wait = Duration::from_millis(word.get("wait").parse().unwrap_or_default());
    let path = match place.base.is_empty() {
        true => "/".to_string(),
        false => place.base.clone(),
    };
    let fill = |line: &str| line.replace("{path}", &path).replace("{speak}", word.get("speak")).replace("{host}", &place.authority()).replace("{agent}", word.get("agent"));
    let asked: String = data::row(data::NET).into_iter().filter(|cell| cell.first() == Some(&"get")).filter_map(|cell| Some(fill(cell.get(1)?) + "\r\n")).collect::<String>() + "\r\n";
    let mut wire = reach(&place, place.patience(wait)).ok()?;
    wire.write_all(asked.as_bytes()).ok()?;
    let mut reader = BufReader::new(wire);
    let (status, said) = head(&mut reader).ok()?;
    (status == word.get("fine")).then(|| body(&mut reader, &said)).flatten()
}
