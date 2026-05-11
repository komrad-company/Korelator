use mockito::{Mock, Server};

pub async fn mock_two_hits(server: &mut Server) -> Mock {
    server
        .mock("POST", "/api/v1/logs/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hits":[{"_timestamp":1000,"_id":"a1","msg":"a"},{"_timestamp":2000,"_id":"b2","msg":"b"}]}"#)
        .create_async()
        .await
}

pub async fn mock_search_after_b2(server: &mut Server) -> Mock {
    server
        .mock("POST", "/api/v1/logs/search")
        .match_body(mockito::Matcher::PartialJson(
            serde_json::json!({"search_after": [2000, "b2"]}),
        ))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hits":[]}"#)
        .create_async()
        .await
}

pub async fn mock_empty_hits(server: &mut Server) -> Mock {
    server
        .mock("POST", "/api/v1/logs/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"hits":[]}"#)
        .create_async()
        .await
}

pub async fn mock_server_error(server: &mut Server) -> Mock {
    server
        .mock("POST", "/api/v1/logs/search")
        .with_status(500)
        .create_async()
        .await
}

pub async fn mock_two_hits_then_503(server: &mut Server) -> (Mock, Mock) {
    let hits = server
        .mock("POST", "/api/v1/logs/search")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"hits":[{"_timestamp":1,"_id":"e1","v":1},{"_timestamp":2,"_id":"e2","v":2}]}"#,
        )
        .expect(1)
        .create_async()
        .await;
    let err = server
        .mock("POST", "/api/v1/logs/search")
        .with_status(503)
        .expect(1)
        .create_async()
        .await;
    (hits, err)
}
