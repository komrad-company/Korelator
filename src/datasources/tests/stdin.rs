use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::BufReader;

use crate::datasources::stdin::StdinSource;

#[tokio::test]
async fn stream_from_emits_valid_json_lines() {
    let input: &[u8] = b"{\"a\":1}\n{\"a\":2}\n";
    let events: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();

    StdinSource::stream_from(BufReader::new(input), |event| {
        captured.lock().unwrap().push(event)
    })
    .await
    .unwrap();

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0]["a"], 1);
    assert_eq!(collected[1]["a"], 2);
}

#[tokio::test]
async fn stream_from_skips_empty_lines() {
    let input: &[u8] = b"{\"a\":1}\n\n   \n{\"a\":2}\n";
    let events: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();

    StdinSource::stream_from(BufReader::new(input), |event| {
        captured.lock().unwrap().push(event)
    })
    .await
    .unwrap();

    assert_eq!(events.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn stream_from_skips_invalid_json() {
    let input: &[u8] = b"{\"a\":1}\nnot-json\n{\"a\":2}\n";
    let events: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();

    StdinSource::stream_from(BufReader::new(input), |event| {
        captured.lock().unwrap().push(event)
    })
    .await
    .unwrap();

    assert_eq!(events.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn stream_from_terminates_on_eof() {
    let input: &[u8] = b"";

    let result = StdinSource::stream_from(BufReader::new(input), |_: Value| {}).await;

    assert!(result.is_ok());
}
