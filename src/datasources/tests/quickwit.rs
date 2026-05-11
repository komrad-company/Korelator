use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use super::fixtures;
use crate::datasources::quickwit::QuickwitSource;
use crate::errors::DatasourceError;

#[tokio::test]
async fn poll_emits_hits_and_extracts_cursor_from_last_hit() {
    let mut server = mockito::Server::new_async().await;
    let _mock = fixtures::mock_two_hits(&mut server).await;

    let source = QuickwitSource::new(server.url(), "logs".into());
    let result = source.poll(&None).await.unwrap();

    assert_eq!(result.hits.len(), 2);
    assert_eq!(result.next_cursor, Some(vec![json!(2000), json!("b2")]));
}

#[tokio::test]
async fn poll_sends_search_after_when_cursor_is_set() {
    let mut server = mockito::Server::new_async().await;
    let mock = fixtures::mock_search_after_b2(&mut server).await;

    let source = QuickwitSource::new(server.url(), "logs".into());
    source
        .poll(&Some(vec![json!(2000), json!("b2")]))
        .await
        .unwrap();

    mock.assert_async().await;
}

#[tokio::test]
async fn poll_returns_no_cursor_on_empty_response() {
    let mut server = mockito::Server::new_async().await;
    let _mock = fixtures::mock_empty_hits(&mut server).await;

    let source = QuickwitSource::new(server.url(), "logs".into());
    let result = source.poll(&None).await.unwrap();

    assert!(result.hits.is_empty());
    assert!(result.next_cursor.is_none());
}

#[tokio::test]
async fn poll_propagates_http_error_on_server_error() {
    let mut server = mockito::Server::new_async().await;
    let _mock = fixtures::mock_server_error(&mut server).await;

    let source = QuickwitSource::new(server.url(), "logs".into());
    assert!(matches!(
        source.poll(&None).await,
        Err(DatasourceError::Http(_))
    ));
}

#[tokio::test]
async fn stream_calls_on_event_for_each_hit_then_stops_on_error() {
    let mut server = mockito::Server::new_async().await;
    let (_hits, _err) = fixtures::mock_two_hits_then_503(&mut server).await;

    let events: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
    let captured = events.clone();

    let source = QuickwitSource::new(server.url(), "logs".into());
    let _ = source
        .stream(|event| captured.lock().unwrap().push(event))
        .await;

    let collected = events.lock().unwrap();
    assert_eq!(collected.len(), 2);
    assert_eq!(collected[0]["v"], 1);
    assert_eq!(collected[1]["v"], 2);
}
