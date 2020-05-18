use anyhow::{Result, Error};
use reqwest::header;

use crate::notion::{PublicPageData};

pub fn fetch(endpoint: String, body: String) -> Result<PublicInfo, Error> {
    let client = reqwest::Client::new();
    let url = format!("https://www.notion.so/api/v3/{}", endpoint);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
    smol::run(async {
        let resp = client.post(&url).headers(headers).body(body).send().await?;
        let res_body = resp.json::<PublicPageData>().await?;
        Ok(res_body)
    })
}

#[allow(dead_code)]
pub fn get() -> Result<String, Error> {
    smol::run(async {
        let resp = reqwest::get("https://www.notion.so/teukka/Notion-b59819a3270d477fb9d6073f09456b8e").await?;
        let body = resp.text().await?;
        Ok(body)
    })
}
