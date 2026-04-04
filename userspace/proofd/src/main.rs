use proofd::{route_request_with_body, DiagnosticsResponse, API_VERSION};
use serde_json::{json, Map, Value};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

enum Command {
    Serve {
        bind: String,
        evidence_dir: PathBuf,
    },
    InternalReplay {
        run_dir: PathBuf,
        verify_request_path: Option<PathBuf>,
    },
}

fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        400 => "Bad Request",
        409 => "Conflict",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    }
}

fn build_http_header(response: &DiagnosticsResponse) -> String {
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nX-Ayken-API-Version: {}\r\nX-Ayken-Authority: none\r\nX-Ayken-Deterministic: true\r\nConnection: close\r\n\r\n",
        response.status_code,
        status_text(response.status_code),
        response.content_type,
        response.body.len(),
        API_VERSION,
    )
}

fn manifest_string_field<'a>(manifest: &'a Value, field: &str) -> Result<&'a str, String> {
    manifest
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("run manifest missing {field}"))
}

fn read_json_file(path: &Path) -> Result<Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("read failed for {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("invalid json at {}: {error}", path.display()))
}

fn load_verify_request_snapshot(path: &Path, source_run_id: &str) -> Result<Value, String> {
    let request = read_json_file(path)?;
    let run_id = request
        .get("run_id")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("verify request missing run_id at {}", path.display()))?;
    if run_id != source_run_id {
        return Err(format!(
            "verify request run_id mismatch: expected {source_run_id}, found {run_id}"
        ));
    }
    Ok(request)
}

fn build_internal_replay_request(
    run_dir: &Path,
    verify_request_path: Option<&Path>,
) -> Result<Vec<u8>, String> {
    let manifest_path = run_dir.join("proofd_run_manifest.json");
    let manifest = read_json_file(&manifest_path)?;
    let source_run_id = run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("invalid run dir: {}", run_dir.display()))?;
    let manifest_run_id = manifest_string_field(&manifest, "run_id")?;
    if manifest_run_id != source_run_id {
        return Err(format!(
            "run manifest id mismatch: expected {source_run_id}, found {manifest_run_id}"
        ));
    }

    let verify_request = if let Some(path) = verify_request_path {
        load_verify_request_snapshot(path, source_run_id)?
    } else if run_dir.join("proofd_verify_request.json").is_file() {
        load_verify_request_snapshot(&run_dir.join("proofd_verify_request.json"), source_run_id)?
    } else {
        let mut verify_request = Map::new();
        for field in [
            "bundle_path",
            "policy_path",
            "registry_path",
            "receipt_mode",
        ] {
            verify_request.insert(
                field.to_string(),
                Value::String(manifest_string_field(&manifest, field)?.to_string()),
            );
        }
        verify_request.insert(
            "run_id".to_string(),
            Value::String(manifest_run_id.to_string()),
        );
        if let Some(receipt_signer) = manifest.get("receipt_signer") {
            verify_request.insert("receipt_signer".to_string(), receipt_signer.clone());
        }
        Value::Object(verify_request)
    };

    serde_json::to_vec(&json!({
        "source_run_id": source_run_id,
        "verify_request": verify_request,
    }))
    .map_err(|error| format!("failed to serialize replay request: {error}"))
}

fn build_internal_replay_cli_output_from_response(
    run_dir: &Path,
    response_body: &Value,
) -> Result<Value, String> {
    let request_fingerprint = response_body
        .get("request_fingerprint")
        .and_then(Value::as_str)
        .ok_or("internal replay response missing request_fingerprint")?;
    let artifact_hash = response_body
        .get("recomputed_artifact_hash")
        .and_then(Value::as_str)
        .ok_or("internal replay response missing recomputed_artifact_hash")?;
    let match_result = response_body
        .get("matches_original")
        .and_then(Value::as_bool)
        .ok_or("internal replay response missing matches_original")?;
    let incident_path = if run_dir
        .join("verification_determinism_incident.json")
        .is_file()
    {
        Some("verification_determinism_incident.json".to_string())
    } else {
        None
    };

    Ok(json!({
        "request_fingerprint": request_fingerprint,
        "artifact_hash": artifact_hash,
        "match_result": match_result,
        "incident_path": incident_path,
    }))
}

fn build_internal_replay_cli_output(
    run_dir: &Path,
    verify_request_path: Option<&Path>,
) -> Result<Value, String> {
    let evidence_dir = run_dir
        .parent()
        .ok_or_else(|| format!("run dir has no evidence root parent: {}", run_dir.display()))?;
    let replay_request = build_internal_replay_request(run_dir, verify_request_path)?;
    let response = route_request_with_body(
        "POST",
        "/internal/replay",
        Some(replay_request.as_slice()),
        evidence_dir,
    );
    let response_body = serde_json::from_slice::<Value>(&response.body)
        .map_err(|error| format!("invalid internal replay response: {error}"))?;

    if response.status_code != 200 && response.status_code != 409 {
        let code = response_body
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("internal_replay_failed");
        return Err(format!(
            "internal replay returned HTTP {} ({code})",
            response.status_code
        ));
    }

    build_internal_replay_cli_output_from_response(run_dir, &response_body)
}

fn parse_args() -> Result<Command, String> {
    let mut bind = String::from("127.0.0.1:4100");
    let mut evidence_dir: Option<PathBuf> = None;
    let mut run_dir: Option<PathBuf> = None;
    let mut verify_request_path: Option<PathBuf> = None;
    let mut internal_replay = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--internal-replay" => {
                internal_replay = true;
            }
            "--run-dir" => {
                run_dir = Some(PathBuf::from(
                    args.next().ok_or("missing value for --run-dir")?,
                ));
            }
            "--verify-request-path" => {
                verify_request_path = Some(PathBuf::from(
                    args.next()
                        .ok_or("missing value for --verify-request-path")?,
                ));
            }
            "--bind" => {
                bind = args.next().ok_or("missing value for --bind")?;
            }
            "--evidence-dir" => {
                evidence_dir = Some(PathBuf::from(
                    args.next().ok_or("missing value for --evidence-dir")?,
                ));
            }
            "-h" | "--help" => {
                println!(
                    "Usage:\n  proofd --evidence-dir <dir> [--bind 127.0.0.1:4100]\n  proofd --internal-replay --run-dir <evidence/run-*/run-id> [--verify-request-path /path/to/proofd_verify_request.json]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown arg: {other}")),
        }
    }

    if internal_replay {
        let run_dir = run_dir.ok_or("missing required --run-dir")?;
        return Ok(Command::InternalReplay {
            run_dir,
            verify_request_path,
        });
    }

    let evidence_dir = evidence_dir.ok_or("missing required --evidence-dir")?;
    Ok(Command::Serve { bind, evidence_dir })
}

fn main() -> Result<(), String> {
    match parse_args()? {
        Command::InternalReplay {
            run_dir,
            verify_request_path,
        } => {
            let output =
                build_internal_replay_cli_output(&run_dir, verify_request_path.as_deref())?;
            println!(
                "{}",
                serde_json::to_string_pretty(&output)
                    .map_err(|error| format!("failed to serialize replay output: {error}"))?
            );
            Ok(())
        }
        Command::Serve { bind, evidence_dir } => serve(bind, evidence_dir),
    }
}

fn serve(bind: String, evidence_dir: PathBuf) -> Result<(), String> {
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

        let request_bytes = &buffer[..size];
        let header_end = request_bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|offset| offset + 4)
            .or_else(|| {
                request_bytes
                    .windows(2)
                    .position(|window| window == b"\n\n")
                    .map(|offset| offset + 2)
            })
            .unwrap_or(size);
        let request = String::from_utf8_lossy(&request_bytes[..header_end]);
        let body = if header_end < size {
            Some(&request_bytes[header_end..])
        } else {
            None
        };
        let first_line = request.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();
        let method = parts.next().unwrap_or_default();
        let target = parts.next().unwrap_or("/");
        let response = route_request_with_body(method, target, body, &evidence_dir);
        let header = build_http_header(&response);
        stream
            .write_all(header.as_bytes())
            .and_then(|_| stream.write_all(&response.body))
            .map_err(|err| format!("write failed: {err}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        build_http_header, build_internal_replay_cli_output_from_response,
        build_internal_replay_request,
    };
    use proofd::DiagnosticsResponse;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let counter = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("{prefix}-{unique}-{counter:016x}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    #[test]
    fn http_header_advertises_phase14_api_contract() {
        let response = DiagnosticsResponse {
            status_code: 200,
            body: br#"{"status":"ok"}"#.to_vec(),
            content_type: "application/json; charset=utf-8",
        };

        let header = build_http_header(&response);

        assert!(header.contains("HTTP/1.1 200 OK\r\n"));
        assert!(header.contains("Content-Length: 15\r\n"));
        assert!(header.contains("X-Ayken-API-Version: 1\r\n"));
        assert!(header.contains("X-Ayken-Authority: none\r\n"));
        assert!(header.contains("X-Ayken-Deterministic: true\r\n"));
        assert!(header.ends_with("\r\n\r\n"));
    }

    #[test]
    fn internal_replay_request_falls_back_to_run_manifest() {
        let evidence_root = temp_dir("proofd-main-replay-request");
        let run_dir = evidence_root.join("run-proofd-main-r1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": "run-proofd-main-r1",
                "bundle_path": "/abs/bundle",
                "policy_path": "/abs/policy.json",
                "registry_path": "/abs/registry.json",
                "receipt_mode": "emit_unsigned",
            }))
            .expect("serialize manifest"),
        )
        .expect("write manifest");

        let request_bytes =
            build_internal_replay_request(&run_dir, None).expect("build replay request");
        let request: Value = serde_json::from_slice(&request_bytes).expect("parse request");

        assert_eq!(
            request.get("source_run_id").and_then(Value::as_str),
            Some("run-proofd-main-r1")
        );
        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("bundle_path"))
                .and_then(Value::as_str),
            Some("/abs/bundle")
        );
        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("run_id"))
                .and_then(Value::as_str),
            Some("run-proofd-main-r1")
        );

        let _ = fs::remove_dir_all(&evidence_root);
    }

    #[test]
    fn internal_replay_request_fallback_copies_receipt_signer_from_manifest() {
        let evidence_root = temp_dir("proofd-main-replay-request-signed-fallback");
        let run_dir = evidence_root.join("run-proofd-main-r1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": "run-proofd-main-r1",
                "bundle_path": "/abs/bundle",
                "policy_path": "/abs/policy.json",
                "registry_path": "/abs/registry.json",
                "receipt_mode": "emit_signed",
                "receipt_signer": {
                    "verifier_node_id": "node-a",
                    "verifier_key_id": "key-a",
                    "signature_algorithm": "ed25519",
                    "private_key": "abc123",
                    "verified_at_utc": "2026-04-03T00:00:00Z"
                }
            }))
            .expect("serialize manifest"),
        )
        .expect("write manifest");

        let request_bytes =
            build_internal_replay_request(&run_dir, None).expect("build replay request");
        let request: Value = serde_json::from_slice(&request_bytes).expect("parse request");

        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("receipt_signer"))
                .and_then(|value| value.get("verifier_key_id"))
                .and_then(Value::as_str),
            Some("key-a")
        );

        let _ = fs::remove_dir_all(&evidence_root);
    }

    #[test]
    fn internal_replay_request_prefers_explicit_verify_request_snapshot() {
        let evidence_root = temp_dir("proofd-main-replay-request-snapshot");
        let run_dir = evidence_root.join("run-proofd-main-r1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": "run-proofd-main-r1",
                "bundle_path": "/abs/bundle",
                "policy_path": "/abs/policy.json",
                "registry_path": "/abs/registry.json",
                "receipt_mode": "emit_unsigned",
            }))
            .expect("serialize manifest"),
        )
        .expect("write manifest");

        let verify_request_path = evidence_root.join("proofd_verify_request.json");
        fs::write(
            &verify_request_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "bundle_path": "/abs/snapshot-bundle",
                "policy_path": "/abs/snapshot-policy.json",
                "registry_path": "/abs/snapshot-registry.json",
                "receipt_mode": "emit_signed",
                "run_id": "run-proofd-main-r1",
                "receipt_signer": {
                    "verifier_node_id": "node-a",
                    "verifier_key_id": "key-a",
                    "signature_algorithm": "ed25519",
                    "private_key": "abc123",
                    "verified_at_utc": "2026-04-03T00:00:00Z"
                }
            }))
            .expect("serialize request"),
        )
        .expect("write request");

        let request_bytes = build_internal_replay_request(&run_dir, Some(&verify_request_path))
            .expect("build replay request");
        let request: Value = serde_json::from_slice(&request_bytes).expect("parse request");

        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("bundle_path"))
                .and_then(Value::as_str),
            Some("/abs/snapshot-bundle")
        );
        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("receipt_signer"))
                .and_then(|value| value.get("verified_at_utc"))
                .and_then(Value::as_str),
            Some("2026-04-03T00:00:00Z")
        );

        let _ = fs::remove_dir_all(&evidence_root);
    }

    #[test]
    fn internal_replay_request_auto_uses_sibling_verify_request_snapshot() {
        let evidence_root = temp_dir("proofd-main-replay-request-auto-snapshot");
        let run_dir = evidence_root.join("run-proofd-main-r1");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("proofd_run_manifest.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "run_id": "run-proofd-main-r1",
                "bundle_path": "/abs/bundle",
                "policy_path": "/abs/policy.json",
                "registry_path": "/abs/registry.json",
                "receipt_mode": "emit_unsigned",
            }))
            .expect("serialize manifest"),
        )
        .expect("write manifest");
        fs::write(
            run_dir.join("proofd_verify_request.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "bundle_path": "/abs/snapshot-bundle",
                "policy_path": "/abs/snapshot-policy.json",
                "registry_path": "/abs/snapshot-registry.json",
                "receipt_mode": "emit_signed",
                "run_id": "run-proofd-main-r1",
                "receipt_signer": {
                    "verifier_node_id": "node-a",
                    "verifier_key_id": "key-a",
                    "signature_algorithm": "ed25519",
                    "private_key": "abc123",
                    "verified_at_utc": "2026-04-03T00:00:00Z"
                }
            }))
            .expect("serialize request"),
        )
        .expect("write request");

        let request_bytes =
            build_internal_replay_request(&run_dir, None).expect("build replay request");
        let request: Value = serde_json::from_slice(&request_bytes).expect("parse request");

        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("bundle_path"))
                .and_then(Value::as_str),
            Some("/abs/snapshot-bundle")
        );
        assert_eq!(
            request
                .get("verify_request")
                .and_then(|value| value.get("receipt_signer"))
                .and_then(|value| value.get("verifier_key_id"))
                .and_then(Value::as_str),
            Some("key-a")
        );

        let _ = fs::remove_dir_all(&evidence_root);
    }

    #[test]
    fn internal_replay_cli_output_reflects_contract_comparison_result() {
        let evidence_root = temp_dir("proofd-main-replay-output");
        let run_dir = evidence_root.join("run-proofd-main-r2");
        fs::create_dir_all(&run_dir).expect("create run dir");
        fs::write(
            run_dir.join("verification_determinism_incident.json"),
            br#"{"type":"determinism_incident"}"#,
        )
        .expect("write incident");

        let output = build_internal_replay_cli_output_from_response(
            &run_dir,
            &serde_json::json!({
                "request_fingerprint": "sha256:test",
                "recomputed_artifact_hash": "sha256:expected",
                "matches_original": false
            }),
        )
        .expect("build replay cli output from contract comparison");

        assert_eq!(
            output.get("request_fingerprint").and_then(Value::as_str),
            Some("sha256:test")
        );
        assert_eq!(
            output.get("artifact_hash").and_then(Value::as_str),
            Some("sha256:expected")
        );
        assert_eq!(
            output.get("match_result").and_then(Value::as_bool),
            Some(false)
        );
        assert_eq!(
            output.get("incident_path").and_then(Value::as_str),
            Some("verification_determinism_incident.json")
        );

        let _ = fs::remove_dir_all(&evidence_root);
    }
}
