//! The exchange: the call line, served. It listens where `data/line.tsv`
//! says, gives every wire a thread, and answers the routes of
//! `route.rs` on top of the rules. It carries no sound and no picture:
//! it keeps who is in each room's call and passes their signals on; the
//! call itself runs between the pages.
//!
//! It speaks plain HTTP on the standard library's sockets, as appkit's
//! `PCall` does from the calling side. It is not there in a browser,
//! where nothing can listen.

mod answer;
mod ask;
mod guest;
mod hub;
mod person;
mod route;
mod set;
mod told;

pub use set::Set;

use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::board::{Board, lock};
use crate::comp::Fault;
use answer::To;
use hub::Hub;

/// The line, listening.
pub struct Exchange {
    hub: Arc<Hub>,
    /// The name it listens under and its port, as a page writes them.
    at: String,
    /// Every address that name stands for on this machine.
    door: Vec<SocketAddr>,
    stop: Arc<AtomicBool>,
    /// The threads that take the wires in, one a door.
    taking: Mutex<Vec<JoinHandle<()>>>,
}

impl Exchange {
    /// Open the line where the settings say, on that board. A name such
    /// as `localhost` stands for more than one address, and the line
    /// listens on each, so a page reaches it whichever it tries first.
    /// Port 0 has the system choose one. `Err` when nothing can listen
    /// there.
    pub fn open(set: Set, board: Arc<Mutex<Board>>) -> std::io::Result<Exchange> {
        let (mut port, mut open, mut failed) = (set.port, Vec::new(), None);
        for named in (set.host.as_str(), set.port).to_socket_addrs()? {
            match TcpListener::bind(SocketAddr::new(named.ip(), port)).and_then(|door| Ok((door.local_addr()?, door))) {
                Ok((at, door)) => {
                    port = at.port();
                    open.push((at, door));
                }
                Err(error) => failed = Some(error),
            }
        }
        if open.is_empty() {
            return Err(failed.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, set.host.clone())));
        }
        let at = format!("{}:{port}", set.host);
        let (hub, stop) = (Arc::new(Hub::new(set, board)), Arc::new(AtomicBool::new(false)));
        let (mut door, mut taking) = (Vec::new(), Vec::new());
        for (at, listener) in open {
            let (inside, ended) = (Arc::clone(&hub), Arc::clone(&stop));
            taking.push(std::thread::Builder::new().spawn(move || take(&listener, &inside, &ended))?);
            door.push(at);
        }
        let (inside, ended) = (Arc::clone(&hub), Arc::clone(&stop));
        std::thread::Builder::new().spawn(move || pulse(&inside, &ended))?;
        Ok(Exchange { hub, at, door, stop, taking: Mutex::new(taking) })
    }

    /// Where it listens, as a page writes it: a name and a port.
    pub fn at(&self) -> String {
        self.at.clone()
    }

    /// Stay until the line is closed.
    pub fn wait(&self) {
        let taking: Vec<JoinHandle<()>> = lock(&self.taking).drain(..).collect();
        taking.into_iter().for_each(|taking| _ = taking.join());
    }

    /// Close the line: every page's line ends, and nothing listens.
    pub fn close(&self) {
        self.stop.store(true, Ordering::SeqCst);
        self.hub.end();
        // Each door is waiting for a wire: one more lets it see the stop.
        self.door.iter().for_each(|door| _ = TcpStream::connect(door));
    }
}

impl Drop for Exchange {
    fn drop(&mut self) {
        self.close();
    }
}

/// Take every wire in and serve it on a thread of its own, until the
/// line is closed. Past the most the limits allow, a wire is refused.
fn take(door: &TcpListener, hub: &Arc<Hub>, stop: &AtomicBool) {
    for wire in door.incoming() {
        if stop.load(Ordering::SeqCst) {
            break;
        }
        let Ok(mut wire) = wire else { continue };
        let inside = Arc::clone(hub);
        let most = hub.limit(|limit| limit.wire);
        match hub.live.fetch_add(1, Ordering::SeqCst) < most {
            true => {
                let work = move || {
                    route::serve(&inside, wire);
                    inside.live.fetch_sub(1, Ordering::SeqCst);
                };
                if std::thread::Builder::new().spawn(work).is_err() {
                    hub.live.fetch_sub(1, Ordering::SeqCst);
                }
            }
            false => {
                hub.live.fetch_sub(1, Ordering::SeqCst);
                _ = wire.write_all(&answer::fail(&hub.set, Fault::Wire, To::default()));
            }
        }
    }
}

/// Tell the board a beat as often as the limits say, until the line is
/// closed.
fn pulse(hub: &Hub, stop: &AtomicBool) {
    let (beat, tick) = hub.limit(|limit| (limit.beat, limit.tick.max(1)));
    let mut waited = 0;
    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(tick));
        waited += tick;
        if waited >= beat {
            waited = 0;
            hub.beat();
        }
    }
}
