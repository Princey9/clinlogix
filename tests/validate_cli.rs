use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

fn start_test_server(body: &str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    listener.set_nonblocking(true).expect("set nonblocking");
    let addr = listener.local_addr().expect("server addr");
    let body = body.to_string();
    let handle = thread::spawn(move || {
        let start = Instant::now();
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let _ = stream.read(&mut buf);
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/fhir+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.as_bytes().len(),
                        body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    break;
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > Duration::from_secs(2) {
                        break;
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (format!("http://{}", addr), handle)
}

#[test]
fn validate_cli_exits_nonzero_on_error_outcome() {
    let outcome = r#"{"resourceType":"OperationOutcome","issue":[{"severity":"error","code":"invalid","diagnostics":"Missing id"}]}"#;
    let (base_url, handle) = start_test_server(outcome);

    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let example = manifest_dir.join("examples").join("patient-bad.json");

    let output = Command::new(env!("CARGO_BIN_EXE_clinlogix"))
        .arg("validate")
        .arg(example)
        .arg("--base-url")
        .arg(&base_url)
        .output()
        .expect("run clinlogix validate");

    let _ = handle.join();

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("FHIR Validation"));
    assert!(stdout.contains("Result: FAIL"));
    assert!(stderr.contains("FHIR validation failed") || stdout.contains("FHIR validation failed"));
}
