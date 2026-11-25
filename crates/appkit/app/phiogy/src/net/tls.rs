//! TLS over a socket, for a secured place, trusting the roots the
//! system's own store holds. A browser has no sockets: there a secured
//! place is not reached this way.

use std::net::TcpStream;

/// The TLS a wire is read and written through.
#[cfg(not(target_family = "wasm"))]
pub type Secure = Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>;
#[cfg(target_family = "wasm")]
pub type Secure = TcpStream;

/// What the system trusts, once.
#[cfg(not(target_family = "wasm"))]
fn config() -> Option<std::sync::Arc<rustls::ClientConfig>> {
    use std::sync::{Arc, OnceLock};
    static MADE: OnceLock<Option<Arc<rustls::ClientConfig>>> = OnceLock::new();
    MADE.get_or_init(|| {
        let mut root = rustls::RootCertStore::empty();
        rustls_native_certs::load_native_certs().certs.into_iter().for_each(|cert| _ = root.add(cert));
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ClientConfig::builder_with_provider(provider).with_safe_default_protocol_versions().ok()?;
        Some(Arc::new(config.with_root_certificates(root).with_no_client_auth()))
    })
    .clone()
}

/// TLS to a host of that name over a socket.
#[cfg(not(target_family = "wasm"))]
pub fn over(host: &str, socket: TcpStream) -> std::io::Result<Secure> {
    use std::io::Error;
    let name = rustls::pki_types::ServerName::try_from(host.to_string()).map_err(Error::other)?;
    let config = config().ok_or_else(|| Error::other(host.to_string()))?;
    let connection = rustls::ClientConnection::new(config, name).map_err(Error::other)?;
    Ok(Box::new(rustls::StreamOwned::new(connection, socket)))
}

/// The socket under the TLS.
#[cfg(not(target_family = "wasm"))]
pub fn socket(secure: &Secure) -> &TcpStream {
    &secure.sock
}

#[cfg(target_family = "wasm")]
pub fn socket(secure: &Secure) -> &TcpStream {
    secure
}

#[cfg(target_family = "wasm")]
pub fn over(host: &str, _socket: TcpStream) -> std::io::Result<Secure> {
    Err(std::io::Error::other(host.to_string()))
}
