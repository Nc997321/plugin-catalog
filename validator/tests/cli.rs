use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn write_catalog(contents: &str) -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, "{contents}").unwrap();
    f
}

#[test]
fn cli_valid_empty_catalog_exits_zero() {
    let f = write_catalog(r#"{"schema_version":1,"plugins":[]}"#);
    Command::cargo_bin("catalog-check")
        .unwrap()
        .args(["--no-reachability", f.path().to_str().unwrap()])
        .assert()
        .success();
}

#[test]
fn cli_invalid_catalog_exits_nonzero_with_stderr() {
    let f = write_catalog(
        r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"bad","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#,
    );
    Command::cargo_bin("catalog-check")
        .unwrap()
        .args(["--no-reachability", f.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn cli_missing_path_arg_exits_two() {
    Command::cargo_bin("catalog-check")
        .unwrap()
        .assert()
        .failure()
        .code(2);
}

#[test]
fn cli_nonexistent_file_exits_two() {
    Command::cargo_bin("catalog-check")
        .unwrap()
        .args(["--no-reachability", "C:/does/not/exist/catalog.json"])
        .assert()
        .failure()
        .code(2);
}