//! Census over a corpus of real flight logs, downloaded on demand.
//!
//! The logs listed in `tests/corpus/sources.json` are not part of the repository (most of them
//! have no license allowing redistribution): they are downloaded from their source and cached.
//! This test reports, per log and per version, what the parser fails to read. It only fails on
//! panics and parse errors. See `tests/corpus/README.md`.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::{env, fs};

use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::record::Record;
use dji_log_parser::DJILog;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A log of the corpus. Other fields of `sources.json` (version, aircraft, source, license) are
/// documentation only.
#[derive(Deserialize)]
struct Source {
    /// Unique name, also the path of the log in the cache
    name: String,
    url: String,
    /// Path of the log inside the zip archive downloaded from `url`
    archive_entry: Option<String>,
    sha256: String,
}

#[derive(Serialize, Default)]
struct LogReport {
    name: String,
    version: u8,
    aircraft: String,
    records: usize,
    /// Records the parser couldn't read, per record type: unsupported types, or known types
    /// whose content failed to parse
    unknown: BTreeMap<u8, usize>,
    invalid: usize,
    invalid_bytes: usize,
    frames: usize,
    /// The log couldn't be checked: download failure or missing keychains
    skipped: Option<String>,
    /// The parser failed on this log: panic or parse error
    error: Option<String>,
}

fn cache_dir() -> PathBuf {
    env::var_os("DJI_LOG_CORPUS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../target/dji-log-corpus")
        })
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn download(url: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    ureq::get(url)
        .call()
        .map_err(|e| format!("download {url}: {e}"))?
        .into_reader()
        .read_to_end(&mut bytes)
        .map_err(|e| format!("download {url}: {e}"))?;
    Ok(bytes)
}

fn write(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

/// Returns the log, from the cache or downloaded from its source.
fn fetch(source: &Source, cache: &Path) -> Result<Vec<u8>, String> {
    let path = cache.join("logs").join(&source.name);
    if let Ok(bytes) = fs::read(&path) {
        if sha256_hex(&bytes) == source.sha256 {
            return Ok(bytes);
        }
    }

    let bytes = match &source.archive_entry {
        None => download(&source.url)?,
        Some(entry) => {
            let archive_path = cache
                .join("archives")
                .join(format!("{}.zip", sha256_hex(source.url.as_bytes())));
            let archive = match fs::read(&archive_path) {
                Ok(archive) => archive,
                Err(_) => {
                    let archive = download(&source.url)?;
                    write(&archive_path, &archive);
                    archive
                }
            };
            let mut bytes = Vec::new();
            zip::ZipArchive::new(Cursor::new(archive))
                .and_then(|mut archive| Ok(archive.by_name(entry)?.read_to_end(&mut bytes)?))
                .map_err(|e| format!("extract {entry}: {e}"))?;
            bytes
        }
    };

    if sha256_hex(&bytes) != source.sha256 {
        return Err(format!("sha256 mismatch for {}", source.url));
    }
    write(&path, &bytes);
    Ok(bytes)
}

/// Returns the keychains of an encrypted log, from the cache or fetched from the DJI API.
fn keychains(
    parser: &DJILog,
    source: &Source,
    cache: &Path,
) -> Result<Option<Vec<Vec<KeychainFeaturePoint>>>, String> {
    if parser.version < 13 {
        return Ok(None);
    }

    let path = cache
        .join("keychains")
        .join(format!("{}.json", source.name));
    if let Ok(bytes) = fs::read(&path) {
        return Ok(Some(serde_json::from_slice(&bytes).unwrap()));
    }

    let api_key =
        env::var("DJI_API_KEY").map_err(|_| "no keychains, set DJI_API_KEY".to_owned())?;
    let keychains = parser
        .fetch_keychains(&api_key)
        .map_err(|e| format!("keychains: {e}"))?;
    write(&path, &serde_json::to_vec_pretty(&keychains).unwrap());
    Ok(Some(keychains))
}

fn census(source: &Source, cache: &Path) -> LogReport {
    let mut report = LogReport {
        name: source.name.clone(),
        ..Default::default()
    };

    let bytes = match fetch(source, cache) {
        Ok(bytes) => bytes,
        Err(e) => {
            report.skipped = Some(e);
            return report;
        }
    };

    let parser = match DJILog::from_bytes(bytes) {
        Ok(parser) => parser,
        Err(e) => {
            report.error = Some(format!("from_bytes: {e}"));
            return report;
        }
    };
    report.version = parser.version;
    report.aircraft = format!("{:?}", parser.details.product_type);

    let keychains = match keychains(&parser, source, cache) {
        Ok(keychains) => keychains,
        Err(e) => {
            report.skipped = Some(e);
            return report;
        }
    };

    let records = match parser.records(keychains.clone()) {
        Ok(records) => records,
        Err(e) => {
            report.error = Some(format!("records: {e}"));
            return report;
        }
    };
    report.records = records.len();
    for record in &records {
        match record {
            Record::Unknown(record_type, _) => {
                *report.unknown.entry(*record_type).or_default() += 1
            }
            Record::Invalid(data) => {
                report.invalid += 1;
                report.invalid_bytes += data.len();
            }
            _ => {}
        }
    }

    match parser.frames(keychains) {
        Ok(frames) => report.frames = frames.len(),
        Err(e) => report.error = Some(format!("frames: {e}")),
    }
    report
}

fn print_summary(reports: &[LogReport]) {
    let mut by_version: BTreeMap<u8, Vec<&LogReport>> = BTreeMap::new();
    for report in reports
        .iter()
        .filter(|r| r.skipped.is_none() && r.error.is_none())
    {
        by_version.entry(report.version).or_default().push(report);
    }

    println!(
        "\n{:<4} {:>5} {:>9} {:>8} {:>8} {:>9} {:>10}  most unknown record types",
        "", "logs", "records", "unknown", "invalid", "frames", "no frames"
    );
    for (version, reports) in by_version {
        let records: usize = reports.iter().map(|r| r.records).sum();
        let mut unknown: BTreeMap<u8, usize> = BTreeMap::new();
        for (record_type, count) in reports.iter().flat_map(|r| &r.unknown) {
            *unknown.entry(*record_type).or_default() += count;
        }
        let unknown_total: usize = unknown.values().sum();
        let mut unknown: Vec<_> = unknown.into_iter().collect();
        unknown.sort_by_key(|&(_, count)| std::cmp::Reverse(count));
        let top: Vec<_> = unknown
            .iter()
            .take(6)
            .map(|(record_type, count)| format!("{record_type}:{count}"))
            .collect();

        println!(
            "v{:<3} {:>5} {:>9} {:>7.1}% {:>8} {:>9} {:>10}  {}",
            version,
            reports.len(),
            records,
            100.0 * unknown_total as f64 / records.max(1) as f64,
            reports.iter().map(|r| r.invalid).sum::<usize>(),
            reports.iter().map(|r| r.frames).sum::<usize>(),
            reports.iter().filter(|r| r.frames == 0).count(),
            top.join(" "),
        );
    }

    for report in reports {
        if let Some(reason) = report.skipped.as_ref().or(report.error.as_ref()) {
            println!("{}: {reason}", report.name);
        }
    }
}

#[test]
#[ignore = "downloads ~70 MB of flight logs: run with `--ignored --nocapture`, in release"]
fn corpus() {
    let sources: Vec<Source> = serde_json::from_slice(
        &fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/sources.json"))
            .unwrap(),
    )
    .unwrap();
    let cache = cache_dir();

    let reports: Vec<LogReport> = sources
        .iter()
        .map(|source| {
            catch_unwind(AssertUnwindSafe(|| census(source, &cache))).unwrap_or_else(|_| {
                LogReport {
                    name: source.name.clone(),
                    error: Some("panic".to_owned()),
                    ..Default::default()
                }
            })
        })
        .collect();

    print_summary(&reports);
    let report_path = cache.join("report.json");
    write(&report_path, &serde_json::to_vec_pretty(&reports).unwrap());
    let report_path = report_path.canonicalize().unwrap_or(report_path);
    println!("\nFull report: {}", report_path.display());

    let errors: Vec<_> = reports.iter().filter(|r| r.error.is_some()).collect();
    assert!(errors.is_empty(), "{} logs failed to parse", errors.len());
}
