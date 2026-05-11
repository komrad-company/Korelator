use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::time::sleep;

use crate::errors::DatasourceError;

#[derive(Deserialize)]
struct SearchResponse {
    hits: Vec<Value>,
}

pub(crate) struct PollResult {
    pub hits: Vec<Value>,
    pub next_cursor: Option<Vec<Value>>,
}

pub struct QuickwitSource {
    client: Client,
    url: String,
    index: String,
}

impl QuickwitSource {
    pub fn new(url: String, index: String) -> Self {
        Self {
            client: Client::new(),
            url,
            index,
        }
    }

    pub(crate) async fn poll(
        &self,
        cursor: &Option<Vec<Value>>,
    ) -> Result<PollResult, DatasourceError> {
        let mut body = json!({
            "query": "*",
            "max_hits": 500,
            "sort_by": [{"field": "_timestamp", "order": "Asc"}, {"field": "_id", "order": "Asc"}],
        });

        if let Some(c) = cursor {
            body["search_after"] = json!(c);
        }

        let resp = self
            .client
            .post(format!("{}/api/v1/{}/search", self.url, self.index))
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json::<SearchResponse>()
            .await?;

        let next_cursor = resp.hits.last().and_then(|h| {
            let ts = h.get("_timestamp").cloned()?;
            let id = h.get("_id").cloned()?;
            Some(vec![ts, id])
        });

        Ok(PollResult {
            hits: resp.hits,
            next_cursor,
        })
    }

    pub async fn stream<F>(&self, mut on_event: F) -> Result<(), DatasourceError>
    where
        F: FnMut(Value),
    {
        let mut cursor: Option<Vec<Value>> = None;

        loop {
            let result = self.poll(&cursor).await?;
            let is_empty = result.hits.is_empty();

            if let Some(next) = result.next_cursor {
                cursor = Some(next);
            }

            for hit in result.hits {
                on_event(hit);
            }

            if is_empty {
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
