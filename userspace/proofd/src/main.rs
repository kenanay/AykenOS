use proofd::route_request;
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn main() -> Result<(), String> {
    let mut bind = String::from("127.0.0.1:4100");
    let mut evidence_dir: Option<PathBuf> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => {
                bind = args.next().ok_or("missing value for --bind")?;
            }
            "--evidence-dir" => {
                evidence_dir = Some(PathBuf::from(
                    args.next().ok_or("missing value for --evidence-dir")?,
                ));
            }
            "-h" | "--help" => {
                println!("Usage: proofd --evidence-dir <dir> [--bind 127.0.0.1:4100]");
                return Ok(());
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    let evidence_dir = evidence_dir.ok_or("missing required --evidence-dir")?;
    let listener = TcpListener::bind(&bind).map_err(|err| format!("bind failed: {err}"))?;

    println!("proofd listening on {bind}");
    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("accept failed: {err}");
                continue;
            }
        };

        let mut buffer = [0_u8; 8192];
        let size = match stream.read(&mut buffer) {
            Ok(size) => size,
            Err(err) => {
                eprintln!("read failed: {err}");
                continue;
            }
        };
        if size == 0 {
            continue;
        }

        let request = String::from_utf8_lossy(&buffer[..size]);
        let first_line = request.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let target = parts.next().unwrap_or("/");
        let response = route_request(method, target, &evidence_dir);

        let header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            response.status_code,
            status_text(response.status_code),
            response.content_type,
            response.body.len()
        );
        stream
            .write_all(header.as_bytes())
            .and_then(|_| stream.write_all(&response.body))
            .map_err(|err| format!("write failed: {err}"))?;
    }

    Ok(())
}
