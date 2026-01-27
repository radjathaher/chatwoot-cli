use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::Method;
use serde_json::Value;

pub struct ResponseData {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

pub struct HttpClient {
    base_url: String,
    token: Option<String>,
    client: Client,
}

impl HttpClient {
    pub fn new(base_url: String, token: Option<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("chatwoot-cli")
            .build()
            .context("build http client")?;
        Ok(Self {
            base_url,
            token,
            client,
        })
    }

    pub fn execute_json(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Option<Value>,
    ) -> Result<ResponseData> {
        let url = join_url(&self.base_url, path);
        let method = Method::from_bytes(method.as_bytes())
            .with_context(|| format!("invalid method {method}"))?;

        let mut req = self.client.request(method, url);
        if !query.is_empty() {
            req = req.query(&query);
        }
        if let Some(token) = &self.token {
            req = req.header("api_access_token", token);
        }
        if let Some(body) = body {
            req = req.header("content-type", "application/json").json(&body);
        }

        let resp = req.send().context("send request")?;
        let status = resp.status();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = resp.text().context("read response body")?;

        Ok(ResponseData {
            status: status.as_u16(),
            headers,
            body,
        })
    }

    pub fn execute_form(
        &self,
        method: &str,
        path: &str,
        query: &[(String, String)],
        body: Vec<(String, String)>,
    ) -> Result<ResponseData> {
        let url = join_url(&self.base_url, path);
        let method = Method::from_bytes(method.as_bytes())
            .with_context(|| format!("invalid method {method}"))?;

        let mut req = self.client.request(method, url);
        if !query.is_empty() {
            req = req.query(&query);
        }
        if let Some(token) = &self.token {
            req = req.header("api_access_token", token);
        }
        req = req.header("content-type", "application/x-www-form-urlencoded").form(&body);

        let resp = req.send().context("send request")?;
        let status = resp.status();
        let headers = resp
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = resp.text().context("read response body")?;

        Ok(ResponseData {
            status: status.as_u16(),
            headers,
            body,
        })
    }
}

fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}
