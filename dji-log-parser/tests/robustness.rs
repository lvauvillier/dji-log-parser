//! Corrupted or truncated inputs must return errors (or partial results), never panic.

use std::path::PathBuf;

use dji_log_parser::keychain::KeychainFeaturePoint;
use dji_log_parser::{DJILog, Error};

fn fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("unable to read {}: {e}", path.display()))
}

fn keychains(name: &str) -> Vec<Vec<KeychainFeaturePoint>> {
    serde_json::from_slice(&fixture(name)).unwrap()
}

fn v14_keychains() -> Vec<Vec<KeychainFeaturePoint>> {
    keychains("v14_mini4pro.keychains.json")
}

/// Parses everything that can be parsed, ignoring errors: only panics make the test fail.
fn parse_all(bytes: Vec<u8>, keychains: Option<Vec<Vec<KeychainFeaturePoint>>>) {
    if let Ok(parser) = DJILog::from_bytes(bytes) {
        let _ = parser.keychains_request();
        let _ = parser.records(keychains.clone());
        let _ = parser.frames(keychains);
    }
}

/// Cuts the log at a few fixed offsets (header, details, first records) and at regular
/// intervals across the file.
fn truncation_offsets(len: usize) -> Vec<usize> {
    let mut offsets = vec![0, 1, 10, 50, 99, 100, 101, 200, 500, 1000];
    offsets.extend((1..20).map(|i| len * i / 20));
    offsets.retain(|&offset| offset < len);
    offsets
}

#[test]
fn truncated_logs_do_not_panic() {
    for (name, keychains) in [
        ("v4.txt", None),
        ("v8.txt", None),
        ("v8_phantom3.txt", None),
        ("v8_mavicpro.txt", None),
        ("v8_mavicpro_2017.txt", None),
        ("v8_mavic2enterprise.txt", None),
        ("v9_inspire1.txt", None),
        ("v10_phantom4pro.txt", None),
        ("v11_mavic2.txt", None),
        ("v11_mavic2enterprise.txt", None),
        ("v11_spark.txt", None),
        ("v14_mini4pro.txt", Some(v14_keychains())),
    ] {
        let bytes = fixture(name);
        for offset in truncation_offsets(bytes.len()) {
            parse_all(bytes[..offset].to_vec(), keychains.clone());
        }
    }
}

#[test]
fn out_of_bounds_detail_offset_is_an_error() {
    let mut bytes = fixture("v4.txt");
    // detail_offset is the first u64 of the prefix (used as is for versions < 12)
    bytes[..8].copy_from_slice(&u64::MAX.to_le_bytes());

    assert!(DJILog::from_bytes(bytes).is_err());
}

#[test]
fn wrong_aes_keys_do_not_panic() {
    let mut keychains = v14_keychains();
    for feature_point in keychains.iter_mut().flatten() {
        // Valid base64 and lengths, wrong values
        feature_point.aes_key = "A".repeat(43) + "=";
        feature_point.aes_iv = "A".repeat(22) + "==";
    }

    parse_all(fixture("v14_mini4pro.txt"), Some(keychains));
}

#[test]
fn invalid_base64_keychain_is_an_error() {
    let mut keychains = v14_keychains();
    keychains[0][0].aes_key = "not base64!".to_owned();

    let parser = DJILog::from_bytes(fixture("v14_mini4pro.txt")).unwrap();
    assert!(matches!(
        parser.records(Some(keychains)),
        Err(Error::InvalidKeychain(_))
    ));
}

#[test]
fn wrong_length_keychain_is_an_error() {
    let mut keychains = v14_keychains();
    // 16 bytes instead of 32
    keychains[0][0].aes_key = "A".repeat(22) + "==";

    let parser = DJILog::from_bytes(fixture("v14_mini4pro.txt")).unwrap();
    assert!(matches!(
        parser.records(Some(keychains)),
        Err(Error::InvalidKeychain(_))
    ));
}
