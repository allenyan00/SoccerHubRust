use anyhow::{anyhow, Error};
use futures::TryStreamExt;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Response, StatusCode};
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::io::BufRead;
use url::form_urlencoded::parse as parse_qs;

const REQUEST_MAX_RETRY: u32 = 3;

pub async fn get_data_from_url(
    url: &str,
    headers: &[(String, String)],
    params: &[(String, String)],
) -> Result<Value, Error> {
    let client = reqwest::Client::new();
    let mut request_retry_count = 0;

    let mut header_map = HeaderMap::new();
    for (k, v) in headers {
        if k.is_empty() || v.is_empty() {
            return Err(anyhow!(StatusCode::BAD_REQUEST).context("Invalid header"));
        }
        let header_name = HeaderName::from_bytes(k.as_bytes())
            .map_err(|_| anyhow!(StatusCode::BAD_REQUEST).context("Invalid header name"))?;
        let header_value = HeaderValue::from_str(v)
            .map_err(|_| anyhow!(StatusCode::BAD_REQUEST).context("Invalid header value"))?;
        header_map.insert(header_name, header_value);
    }
    // 将切片转换为 HashMap
    let params: HashMap<String, String> = params.iter().cloned().collect();
    use futures::StreamExt;

    loop {
        let response = client
            .get(url)
            .query(&params)
            .headers(header_map.clone())
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        print!("Response status: {}", response.status());
        print!("The response status is {}", response.status().is_success());
        if response.status().is_success() {
            // 获取响应头
            let content_type = response
                .headers()
                .get("Content-Type")
                .and_then(|ct| ct.to_str().ok())
                .unwrap_or("")
                .to_string();

            // 读取整个响应体为字节流，便于后续处理
            let body_bytes = response.bytes().await?;
            let body_str = String::from_utf8_lossy(&body_bytes);

            let data = match content_type.split(';').next() {
                Some("application/json") => from_str::<Value>(&body_str)?,
                Some("application/xml") => from_str::<Value>(&body_str)?,
                Some("application/x-www-form-urlencoded") => {
                    let parsed: HashMap<_, _> =
                        parse_qs(body_str.as_bytes()).into_owned().collect();
                    serde_json::to_value(parsed)?
                }
                Some("text/plain") => serde_json::to_value(body_str.to_string())?,
                Some("text/csv") => {
                    let reader = csv::Reader::from_reader(body_str.as_bytes());
                    let records: Vec<HashMap<String, String>> = reader
                        .into_records()
                        .filter_map(|r| r.ok())
                        .map(|r| r.deserialize(None).unwrap_or_default())
                        .collect();
                    serde_json::to_value(records)?
                }
                Some("text/html") => serde_json::to_value(body_str.to_string())?,
                _ => return Ok(Value::Null),
            };

            // if !data.is_null() && !data.as_str().map_or(false, |s| s.trim().is_empty()) {
            //     return Ok(data);
            // }
            if !data.is_null() {
                return Ok(data);
            }
        }

        if request_retry_count >= REQUEST_MAX_RETRY {
            return Err(anyhow!(StatusCode::INTERNAL_SERVER_ERROR).context("Max retries exceeded"));
        }

        request_retry_count += 1;
    }
}
