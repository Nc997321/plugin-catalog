//! Plugin catalog validator (standalone, no allinkit dependency).
//!
//! Pure format validation + cross-entry uniqueness for `catalog.json`.
//! Reachability (`check_reachability`) is network IO, added in a later task
//! and called only by the CLI / CI; it is not unit-tested.

use std::collections::HashSet;

/// One validation finding. `Display` renders a single human-readable line.
#[derive(Debug, Clone)]
pub enum Finding {
    MalformedJson { detail: String },
    SchemaVersion { got: u32 },
    DuplicateId { id: String },
    EmptyField { id: String, field: &'static str },
    BadSemver { id: String, version: String },
    BadPubkey { id: String, reason: String },
    BadId { id: String },
    BadUrl { id: String, field: &'static str, url: String },
    Unreachable { id: String, url: String, status: String },
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Finding::MalformedJson { detail } => write!(f, "malformed json: {detail}"),
            Finding::SchemaVersion { got } => write!(f, "schema_version must be 1, got {got}"),
            Finding::DuplicateId { id } => write!(f, "[{id}] duplicate id"),
            Finding::EmptyField { id, field } => write!(f, "[{id}] empty required field: {field}"),
            Finding::BadSemver { id, version } => write!(f, "[{id}] bad semver: {version}"),
            Finding::BadPubkey { id, reason } => write!(f, "[{id}] bad signing_pubkey: {reason}"),
            Finding::BadId { id } => write!(f, "[{id}] id must match [A-Za-z0-9_-]+"),
            Finding::BadUrl { id, field, url } => write!(f, "[{id}] {field} must be https://: {url}"),
            Finding::Unreachable { id, url, status } => write!(f, "[{id}] unreachable: {url} ({status})"),
        }
    }
}

// Catalog serde model — mirrors the shipped client's CatalogEntry shape (deserialization only).
// All fields default so a missing required field becomes an EmptyField Finding, not a hard error.
#[derive(serde::Deserialize, Default)]
struct CatalogEntry {
    #[serde(default)] id: String,
    #[serde(default)] name: String,
    #[serde(default)] description: String,
    #[serde(default)] author: String,
    #[serde(default)] version: String,
    #[serde(default)] tags: Vec<String>,
    #[serde(default)] capabilities: Vec<String>,
    #[serde(default)] download_url: String,
    #[serde(default)] signing_pubkey: String,
    #[serde(default)] icon_url: Option<String>,
}

#[derive(serde::Deserialize)]
struct CatalogJson {
    #[serde(default)] schema_version: u32,
    #[serde(default)] plugins: Vec<CatalogEntry>,
}

fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn require_non_empty(id: &str, value: &str, field: &'static str, out: &mut Vec<Finding>) {
    if value.is_empty() {
        out.push(Finding::EmptyField { id: id.to_string(), field });
    }
}

/// Pure validation: parse + field checks + cross-entry uniqueness. No network.
pub fn validate_catalog(json: &str) -> Result<(), Vec<Finding>> {
    let catalog: CatalogJson = match serde_json::from_str(json) {
        Ok(c) => c,
        Err(e) => return Err(vec![Finding::MalformedJson { detail: e.to_string() }]),
    };

    let mut findings = Vec::new();
    if catalog.schema_version != 1 {
        findings.push(Finding::SchemaVersion { got: catalog.schema_version });
    }

    let mut seen: HashSet<String> = HashSet::new();
    for p in &catalog.plugins {
        // id: non-empty + charset
        if p.id.is_empty() {
            findings.push(Finding::EmptyField { id: String::new(), field: "id" });
            // can't use this id for further reporting or dedup; skip dedup
        } else if !is_valid_id(&p.id) {
            findings.push(Finding::BadId { id: p.id.clone() });
        } else if !seen.insert(p.id.clone()) {
            findings.push(Finding::DuplicateId { id: p.id.clone() });
        }
        let rid = if p.id.is_empty() { "<missing>" } else { p.id.as_str() };

        require_non_empty(rid, &p.name, "name", &mut findings);
        require_non_empty(rid, &p.version, "version", &mut findings);
        require_non_empty(rid, &p.download_url, "download_url", &mut findings);
        require_non_empty(rid, &p.signing_pubkey, "signing_pubkey", &mut findings);

        // version semver (only if non-empty — empty already reported)
        if !p.version.is_empty() && semver::Version::parse(&p.version).is_err() {
            findings.push(Finding::BadSemver { id: rid.to_string(), version: p.version.clone() });
        }

        // pubkey hex (only if non-empty)
        if !p.signing_pubkey.is_empty() && hex::decode(&p.signing_pubkey).is_err() {
            findings.push(Finding::BadPubkey {
                id: rid.to_string(),
                reason: format!("not valid hex: {}", p.signing_pubkey),
            });
        }

        // download_url https
        if !p.download_url.is_empty() && !p.download_url.starts_with("https://") {
            findings.push(Finding::BadUrl {
                id: rid.to_string(),
                field: "download_url",
                url: p.download_url.clone(),
            });
        }

        // icon_url https if present
        if let Some(u) = &p.icon_url {
            if !u.is_empty() && !u.starts_with("https://") {
                findings.push(Finding::BadUrl {
                    id: rid.to_string(),
                    field: "icon_url",
                    url: u.clone(),
                });
            }
        }
    }

    if findings.is_empty() {
        Ok(())
    } else {
        Err(findings)
    }
}

/// Network reachability: HEAD each `download_url` (https only). Called by CLI/CI, not unit-tested.
pub fn check_reachability(json: &str) -> Vec<Finding> {
    let catalog: CatalogJson = match serde_json::from_str(json) {
        Ok(c) => c,
        Err(_) => return Vec::new(), // malformed json already reported by validate_catalog
    };
    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => return vec![Finding::MalformedJson { detail: format!("http client: {e}") }],
    };
    let mut out = Vec::new();
    for p in &catalog.plugins {
        if !p.download_url.starts_with("https://") {
            continue; // bad url already reported as BadUrl
        }
        match client.head(&p.download_url).send() {
            Ok(r) if r.status().is_success() => {}
            Ok(r) => out.push(Finding::Unreachable {
                id: p.id.clone(),
                url: p.download_url.clone(),
                status: r.status().to_string(),
            }),
            Err(e) => out.push(Finding::Unreachable {
                id: p.id.clone(),
                url: p.download_url.clone(),
                status: e.to_string(),
            }),
        }
    }
    out
}