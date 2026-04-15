use aegis_ai_runtime::{Aegis, Policy};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_test_dir(name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("aegis-tests-{}-{}", name, nonce));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("malicious")
        .join(name);
    fs::read_to_string(path).unwrap()
}

fn start_single_response_http_server(body: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });

    format!("http://{}/ok", addr)
}

#[test]
fn test_default_denies_all_privileged_capabilities() {
    let aegis = Aegis::new();
    let scripts = [
        "http_get(\"http://127.0.0.1:1\")",
        "kv_set(\"x\", \"y\")",
        "kv_get(\"x\")",
        "file_read(\"/tmp/x\")",
        "file_write(\"/tmp/x\", \"y\")",
        "file_list(\"/tmp\")",
    ];

    for script in scripts {
        let err = aegis.execute(script).unwrap_err();
        assert!(
            err.contains("Function not found"),
            "expected function denied by default, got: {err}"
        );
    }
}

#[test]
fn test_http_get_granted_only_by_policy_and_domain_constrained() {
    let server_url = start_single_response_http_server("ok");
    let policy_yaml = r#"
tools:
  http_tool:
    capabilities:
      - http_get
    resource_limits:
      memory_mb: 64
      max_operations: 1000000
      max_call_depth: 64
      timeout_seconds: 30
    allowed_domains:
      - 127.0.0.1
"#;

    let policy = Policy::from_yaml(policy_yaml).unwrap();
    let aegis = Aegis::new().with_policy(&policy, "http_tool");

    let allowed_script = format!("http_get(\"{}\")", server_url);
    assert!(
        aegis.execute(&allowed_script).is_ok(),
        "expected allowed domain request to succeed"
    );

    let script = fixture("domain_disallowed.rhai");
    assert!(!policy
        .get_tool("http_tool")
        .unwrap()
        .allows_domain("http://example.com"));
    let denied = aegis.execute(&script).unwrap();
    assert!(
        format!("{denied:?}").contains("core::result::Result"),
        "expected disallowed domain to return Result error object"
    );
}

#[test]
fn test_kv_capabilities_granted_by_policy_and_prefix_constrained() {
    let policy_yaml = r#"
tools:
  kv_tool:
    capabilities:
      - kv_set
      - kv_get
    resource_limits:
      memory_mb: 64
      max_operations: 1000000
      max_call_depth: 64
      timeout_seconds: 30
    allowed_key_prefixes:
      - "weather:"
"#;

    let policy = Policy::from_yaml(policy_yaml).unwrap();
    let aegis = Aegis::new().with_policy(&policy, "kv_tool");

    assert!(
        aegis
            .execute("kv_set(\"weather:today\", \"sunny\"); kv_get(\"weather:today\")")
            .is_ok(),
        "expected allowed key prefix operations to succeed"
    );

    let script = fixture("key_prefix_disallowed.rhai");
    assert!(!policy.get_tool("kv_tool").unwrap().allows_key("secret:key"));
    let denied = aegis.execute(&script).unwrap();
    assert!(
        format!("{denied:?}").contains("core::result::Result"),
        "expected disallowed key to return Result error object"
    );
}

#[test]
fn test_file_capabilities_granted_by_policy_and_path_constrained() {
    let test_dir = unique_test_dir("files");
    let input_path = test_dir.join("input.txt");
    let output_path = test_dir.join("output.txt");
    fs::write(&input_path, "safe data").unwrap();

    let allowed_root = format!("{}/", test_dir.to_string_lossy());
    let policy_yaml = format!(
        r#"
tools:
  file_tool:
    capabilities:
      - file_read
      - file_write
      - file_list
    resource_limits:
      memory_mb: 64
      max_operations: 1000000
      max_call_depth: 64
      timeout_seconds: 30
    allowed_paths:
      - "{}"
"#,
        allowed_root
    );

    let policy = Policy::from_yaml(&policy_yaml).unwrap();
    let aegis = Aegis::new().with_policy(&policy, "file_tool");

    let read_script = format!("file_read(\"{}\")", input_path.to_string_lossy());
    assert!(
        aegis.execute(&read_script).is_ok(),
        "expected allowed file_read to succeed"
    );

    let write_script = format!(
        "file_write(\"{}\", \"written\")",
        output_path.to_string_lossy()
    );
    assert!(
        aegis.execute(&write_script).is_ok(),
        "expected allowed file_write to succeed"
    );
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "written");

    let list_script = format!("file_list(\"{}\")", test_dir.to_string_lossy());
    assert!(aegis.execute(&list_script).is_ok());

    let script = fixture("path_outside_allowlist.rhai");
    assert!(!policy
        .get_tool("file_tool")
        .unwrap()
        .allows_path("/etc/passwd"));
    let denied = aegis.execute(&script).unwrap();
    assert!(
        format!("{denied:?}").contains("core::result::Result"),
        "expected disallowed path to return Result error object"
    );
}
