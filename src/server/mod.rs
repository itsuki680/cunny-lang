mod http;
mod watch;
mod web;

use std::io;
use std::net::TcpListener;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use crate::build_site;
use http::handle_connection;
pub(crate) use watch::site_fingerprint;

#[cfg(test)]
pub(crate) use web::{reload_script, request_path};

const ADDRESS: &str = "127.0.0.1:3000";
const POLL_INTERVAL: Duration = Duration::from_millis(400);

pub fn brat_correction(root: &Path) -> Result<(), String> {
    let report = build_site(root)?;
    let output = report.output;
    let listener = TcpListener::bind(ADDRESS)
        .map_err(|error| format!("could not start live server at {ADDRESS}: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("could not configure live server: {error}"))?;

    let mut fingerprint = site_fingerprint(root)
        .map_err(|error| format!("could not watch '{}': {error}", root.display()))?;
    let mut last_check = Instant::now();
    let mut version = 1_u64;

    println!("Brat correction is watching http://{ADDRESS}");
    println!("Press Ctrl-C to stop.");

    loop {
        if last_check.elapsed() >= POLL_INTERVAL {
            match site_fingerprint(root) {
                Ok(next) if next != fingerprint => {
                    fingerprint = next;
                    match build_site(root) {
                        Ok(report) => {
                            version = version.wrapping_add(1);
                            println!("Corrected {} page(s).", report.pages);
                        }
                        Err(error) => eprintln!("Correction failed: {error}"),
                    }
                }
                Ok(_) => {}
                Err(error) => eprintln!("Could not check for changes: {error}"),
            }
            last_check = Instant::now();
        }

        match listener.accept() {
            Ok((stream, _)) => {
                if let Err(error) = handle_connection(stream, &output, version) {
                    eprintln!("Live server request failed: {error}");
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(error) => return Err(format!("live server stopped: {error}")),
        }
    }
}
