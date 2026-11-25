//! Snapshot tests over real flight logs, one per log format family.
//!
//! Each snapshot captures a summary of the parsed log rather than the full output, so that
//! diffs stay reviewable: details, record counts per type, frame count and a few sample frames.
//!
//! To update snapshots after an intended change: `cargo insta review` (or `INSTA_UPDATE=always`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use dji_log_parser::frame::Frame;
use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::layout::details::Details;
use dji_log_parser::DJILog;
use serde::Serialize;

#[derive(Serialize)]
struct Summary {
    version: u8,
    details: Details,
    /// Number of records per record type
    record_counts: BTreeMap<String, usize>,
    frame_count: usize,
    /// First, middle and last frames
    sample_frames: Vec<SampleFrame>,
}

#[derive(Serialize)]
struct SampleFrame {
    index: usize,
    frame: Frame,
}

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("unable to read {}: {e}", path.display()))
}

/// Loads keychains previously fetched from the DJI API, so tests never hit the network.
fn keychains(name: &str) -> Vec<Vec<KeychainFeaturePoint>> {
    serde_json::from_slice(&fixture(name)).expect("unable to parse keychains")
}

fn summarize(bytes: Vec<u8>, keychains: Option<Vec<Vec<KeychainFeaturePoint>>>) -> Summary {
    let parser = DJILog::from_bytes(bytes).expect("unable to parse log");

    let records = parser
        .records(keychains.clone())
        .expect("unable to parse records");
    let mut record_counts = BTreeMap::new();
    for record in &records {
        let value = serde_json::to_value(record).expect("unable to serialize record");
        let record_type = value["type"].as_str().unwrap_or("?").to_owned();
        *record_counts.entry(record_type).or_insert(0) += 1;
    }

    let frames = parser.frames(keychains).expect("unable to parse frames");
    let frame_count = frames.len();
    let mut indexes = vec![0, frame_count / 2, frame_count.saturating_sub(1)];
    indexes.dedup();
    let sample_frames = indexes
        .into_iter()
        .filter_map(|index| {
            frames.get(index).map(|frame| SampleFrame {
                index,
                frame: frame.clone(),
            })
        })
        .collect();

    Summary {
        version: parser.version,
        details: parser.details,
        record_counts,
        frame_count,
        sample_frames,
    }
}

#[test]
fn v4() {
    insta::assert_json_snapshot!(summarize(fixture("v4.txt"), None));
}

#[test]
fn v8() {
    insta::assert_json_snapshot!(summarize(fixture("v8.txt"), None));
}

#[test]
fn v8_phantom3() {
    insta::assert_json_snapshot!(summarize(fixture("v8_phantom3.txt"), None));
}

#[test]
fn v11_mavic2() {
    insta::assert_json_snapshot!(summarize(fixture("v11_mavic2.txt"), None));
}

#[test]
fn v11_spark() {
    insta::assert_json_snapshot!(summarize(fixture("v11_spark.txt"), None));
}

#[test]
fn v14_mini4pro() {
    insta::assert_json_snapshot!(summarize(
        fixture("v14_mini4pro.txt"),
        Some(keychains("v14_mini4pro.keychains.json"))
    ));
}
