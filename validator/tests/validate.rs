use catalog_validator::{validate_catalog, Finding};

/// Helper: collect findings from a JSON string.
fn findings(json: &str) -> Vec<Finding> {
    validate_catalog(json).err().unwrap_or_default()
}

#[test]
fn valid_empty_catalog_passes() {
    let json = r#"{"schema_version":1,"plugins":[]}"#;
    assert!(validate_catalog(json).is_ok());
}

#[test]
fn valid_full_entry_passes() {
    let json = r#"{
        "schema_version": 1,
        "plugins": [{
            "id": "json-tool", "name": "JSON Tool", "description": "fmt",
            "author": "me", "version": "1.0.0",
            "tags": ["utility"], "capabilities": ["kv"],
            "download_url": "https://github.com/x/json-tool/releases/download/v1.0.0/json-tool.aikpkg",
            "signing_pubkey": "abcdef1234567890",
            "icon_url": "https://raw.githubusercontent.com/x/icon.png"
        }]
    }"#;
    assert!(validate_catalog(json).is_ok());
}

#[test]
fn malformed_json_is_finding() {
    let fs = findings("{not json");
    assert!(matches!(fs[0], Finding::MalformedJson { .. }));
}

#[test]
fn schema_version_not_1_is_finding() {
    let fs = findings(r#"{"schema_version":2,"plugins":[]}"#);
    assert!(matches!(fs[0], Finding::SchemaVersion { got: 2 }));
}

#[test]
fn missing_required_field_is_empty_field_finding() {
    // version absent → defaults to "" → EmptyField
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::EmptyField { field: "version", .. })));
}

#[test]
fn empty_name_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::EmptyField { field: "name", .. })));
}

#[test]
fn bad_semver_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"not-semver","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadSemver { version, .. } if version == "not-semver")));
}

#[test]
fn empty_pubkey_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":""}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::EmptyField { field: "signing_pubkey", .. })));
}

#[test]
fn bad_hex_pubkey_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":"xyz-not-hex"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadPubkey { .. })));
}

#[test]
fn odd_length_hex_pubkey_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abc"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadPubkey { .. })));
}

#[test]
fn bad_id_charset_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"bad id!","name":"X","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadId { .. })));
}

#[test]
fn non_https_download_url_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"1.0.0","download_url":"http://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadUrl { field: "download_url", .. })));
}

#[test]
fn non_https_icon_url_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"1.0.0","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd","icon_url":"http://x.com/icon.png"}]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::BadUrl { field: "icon_url", .. })));
}

#[test]
fn duplicate_id_is_finding() {
    let json = r#"{"schema_version":1,"plugins":[
        {"id":"dup","name":"A","version":"1.0.0","download_url":"https://a.com/a.aikpkg","signing_pubkey":"abcd"},
        {"id":"dup","name":"B","version":"1.0.0","download_url":"https://b.com/b.aikpkg","signing_pubkey":"ef01"}
    ]}"#;
    let fs = findings(json);
    assert!(fs.iter().any(|f| matches!(f, Finding::DuplicateId { id, .. } if id == "dup")));
}

#[test]
fn display_outputs_human_readable_line() {
    let fs = findings(r#"{"schema_version":1,"plugins":[{"id":"x","name":"X","version":"bad","download_url":"https://x.com/x.aikpkg","signing_pubkey":"abcd"}]}"#);
    let rendered = fs[0].to_string();
    assert!(rendered.to_lowercase().contains("bad semver"));
}