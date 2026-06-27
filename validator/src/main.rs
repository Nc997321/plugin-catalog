use std::path::PathBuf;
use std::process::ExitCode;

use catalog_validator::{check_reachability, validate_catalog, Finding};

const USAGE: &str = "usage: catalog-check <catalog.json> [--no-reachability]";

fn main() -> ExitCode {
    let mut no_reachability = false;
    let mut path: Option<PathBuf> = None;
    for a in std::env::args().skip(1) {
        if a == "--no-reachability" {
            no_reachability = true;
        } else if a == "--help" || a == "-h" {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        } else if path.is_none() {
            path = Some(PathBuf::from(a));
        } else {
            eprintln!("{USAGE}\nunexpected argument: {a}");
            return ExitCode::from(2);
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    let json = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {}: {e}", path.display());
            return ExitCode::from(2);
        }
    };

    let mut findings: Vec<Finding> = match validate_catalog(&json) {
        Ok(()) => Vec::new(),
        Err(fs) => fs,
    };
    if !no_reachability {
        findings.extend(check_reachability(&json));
    }

    if findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        for f in &findings {
            eprintln!("{f}");
        }
        ExitCode::from(1)
    }
}