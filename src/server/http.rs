use std::fs;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

use super::web::{content_type, reload_script, request_path};

pub(super) fn handle_connection(
    mut stream: TcpStream,
    output: &Path,
    version: u64,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("could not configure connection: {error}"))?;

    let mut buffer = [0_u8; 8192];
    let length = stream
        .read(&mut buffer)
        .map_err(|error| format!("could not read request: {error}"))?;
    if length == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..length]);
    let Some(first_line) = request.lines().next() else {
        return send_response(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Bad request",
            false,
        );
    };
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");
    let head_only = method == "HEAD";

    if method != "GET" && !head_only {
        return send_response(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain",
            b"Method not allowed",
            false,
        );
    }

    if target.split('?').next() == Some("/__cunny_version") {
        let body = version.to_string();
        return send_response(
            &mut stream,
            "200 OK",
            "text/plain; charset=utf-8",
            body.as_bytes(),
            head_only,
        );
    }

    let Some(relative) = request_path(target) else {
        return send_response(
            &mut stream,
            "400 Bad Request",
            "text/plain",
            b"Bad path",
            head_only,
        );
    };
    let mut path = output.join(relative);
    if path.is_dir() {
        path.push("index.html");
    }

    let mut body = match fs::read(&path) {
        Ok(body) => body,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return send_response(
                &mut stream,
                "404 Not Found",
                "text/plain",
                b"Not found",
                head_only,
            );
        }
        Err(error) => return Err(format!("could not read '{}': {error}", path.display())),
    };
    let content_type = content_type(&path);
    if content_type.starts_with("text/html") {
        body.extend_from_slice(reload_script(version).as_bytes());
    }

    send_response(&mut stream, "200 OK", content_type, &body, head_only)
}

fn send_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<(), String> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .map_err(|error| format!("could not write response: {error}"))?;

    if !head_only {
        stream
            .write_all(body)
            .map_err(|error| format!("could not write response body: {error}"))?;
    }
    Ok(())
}
