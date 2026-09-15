//! Container health probe.
//!
//! The runtime image is distroless, so there is no shell, `curl` or `wget`
//! for a Docker `HEALTHCHECK` to call. This binary sends `GET /health` to the
//! server on loopback, using the port from [`gallery_api::bind_addr`], and
//! exits `0` on HTTP 200 or `1` on anything else, including a timeout.
//!
//! It speaks raw HTTP/1.1 over `std::net` so the probe pulls in no HTTP
//! client and starts in a few milliseconds.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::ExitCode;
use std::time::Duration;

/// Upper bound for connect, write and read, kept below the `HEALTHCHECK`
/// timeout so the probe reports its own failure instead of being killed.
const TIMEOUT: Duration = Duration::from_secs(3);

fn main() -> ExitCode {
    match probe() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("healthcheck failed: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// Performs one `GET /health` against the local server and checks the
/// status line.
fn probe() -> anyhow::Result<()> {
    let port = gallery_api::bind_addr()?.port();
    let target = SocketAddr::from((Ipv4Addr::LOCALHOST, port));

    let mut stream = TcpStream::connect_timeout(&target, TIMEOUT)?;
    stream.set_read_timeout(Some(TIMEOUT))?;
    stream.set_write_timeout(Some(TIMEOUT))?;
    stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let status_line = response.lines().next().unwrap_or_default();
    anyhow::ensure!(
        status_line.split_whitespace().nth(1) == Some("200"),
        "unexpected response `{status_line}`"
    );
    Ok(())
}
