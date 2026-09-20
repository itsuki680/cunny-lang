use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use cunny_lang::build_site;

const INCREMENT: &str = "😭💢💢";
const OUTPUT: &str = "💢😭😭";

#[test]
fn builds_pages_layouts_and_assets() {
    let root = temporary_site_directory("build");
    fs::create_dir_all(root.join("pages/blog")).unwrap();
    fs::create_dir_all(root.join("layout")).unwrap();
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::create_dir_all(root.join("dist")).unwrap();

    fs::write(root.join("pages/blog/index.cunny"), output_byte(b'P')).unwrap();
    fs::write(root.join("layout/header.cunny"), output_byte(b'H')).unwrap();
    fs::write(root.join("layout/footer.cunny"), output_byte(b'F')).unwrap();
    fs::write(root.join("assets/style.css"), "body { color: red; }").unwrap();
    fs::write(root.join("dist/stale.html"), "old").unwrap();

    let report = build_site(&root).unwrap();

    assert_eq!(report.pages, 1);
    assert_eq!(
        fs::read_to_string(root.join("dist/blog/index.html")).unwrap(),
        "HPF"
    );
    assert_eq!(
        fs::read_to_string(root.join("dist/assets/style.css")).unwrap(),
        "body { color: red; }"
    );
    assert!(!root.join("dist/stale.html").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_build_keeps_the_previous_output() {
    let root = temporary_site_directory("failed");
    fs::create_dir_all(root.join("pages")).unwrap();
    fs::create_dir_all(root.join("dist")).unwrap();
    fs::write(root.join("pages/index.cunny"), "not cunny").unwrap();
    fs::write(root.join("dist/index.html"), "last good build").unwrap();

    assert!(build_site(&root).is_err());
    assert_eq!(
        fs::read_to_string(root.join("dist/index.html")).unwrap(),
        "last good build"
    );

    fs::remove_dir_all(root).unwrap();
}

fn output_byte(byte: u8) -> String {
    format!("{}{OUTPUT}", INCREMENT.repeat(byte.into()))
}

fn temporary_site_directory(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "cunny-site-test-{}-{label}-{unique}",
        process::id()
    ))
}
