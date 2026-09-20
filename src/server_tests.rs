use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::{reload_script, request_path, site_fingerprint};

#[test]
fn live_server_resolves_safe_paths() {
    assert_eq!(request_path("/"), Some(PathBuf::from("index.html")));
    assert_eq!(
        request_path("/assets/main.css?version=1"),
        Some(PathBuf::from("assets/main.css"))
    );
    assert!(request_path("/../secret").is_none());
    assert!(request_path("/%2e%2e/secret").is_none());
}

#[test]
fn reload_script_tracks_the_build_version() {
    let script = reload_script(42);
    assert!(script.contains("/__cunny_version"));
    assert!(script.contains("let v=\"42\""));
    assert!(script.contains("location.reload()"));
}

#[test]
fn fingerprint_changes_when_site_files_change() {
    let root = temporary_directory();
    fs::create_dir_all(root.join("pages")).unwrap();
    fs::write(root.join("pages/index.cunny"), "😭💢💢").unwrap();
    let before = site_fingerprint(&root).unwrap();

    fs::write(root.join("pages/about.cunny"), "💢😭😭").unwrap();
    let after = site_fingerprint(&root).unwrap();

    assert_ne!(before, after);
    fs::remove_dir_all(root).unwrap();
}

fn temporary_directory() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("cunny-server-test-{}-{unique}", process::id()))
}
