use async_trait::async_trait;
use serde_json::json;
use crate::capabilities::Capability;
use crate::tools::{Tool, ToolResult, HttpResponse};

pub struct HttpGetTool {
    allowed_domains: Vec<String>,
}

impl HttpGetTool {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self { allowed_domains }
    }
}

#[async_trait]
impl Tool for HttpGetTool {
    fn name(&self) -> &'static str {
        "http_get"
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        vec![Capability::HttpGet]
    }

    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let url = args.get("url")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'url' argument")?;

        // Check domain allowlist
        if let Some(domain) = url.strip_prefix("https://").or(url.strip_prefix("http://")) {
            let host = domain.split('/').next().unwrap_or(domain);
            if !self.allowed_domains.is_empty() && !self.allowed_domains.iter().any(|d| host.contains(d)) {
                return Err(format!("Domain not allowed: {}", host));
            }
        }

        let client = reqwest::Client::new();
        let response = client.get(url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.text().await.map_err(|e| e.to_string())?;

        Ok(json!(HttpResponse {
            status,
            headers,
            body,
        }))
    }
}

pub struct HttpPostTool {
    allowed_domains: Vec<String>,
}

impl HttpPostTool {
    pub fn new(allowed_domains: Vec<String>) -> Self {
        Self { allowed_domains }
    }
}

#[async_trait]
impl Tool for HttpPostTool {
    fn name(&self) -> &'static str {
        "http_post"
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        vec![Capability::HttpPost]
    }

    async fn execute(&self, args: serde_json::Value) -> Result<serde_json::Value, String> {
        let url = args.get("url")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'url' argument")?;
        
        let body = args.get("body")
            .map(|v| v.to_string())
            .unwrap_or_default();

        // Check domain allowlist
        if let Some(domain) = url.strip_prefix("https://").or(url.strip_prefix("http://")) {
            let host = domain.split('/').next().unwrap_or(domain);
            if !self.allowed_domains.is_empty() && !self.allowed_domains.iter().any(|d| host.contains(d)) {
                return Err(format!("Domain not allowed: {}", host));
            }
        }

        let client = reqwest::Client::new();
        let response = client.post(url)
            .body(body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response.headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        let body = response.text().await.map_err(|e| e.to_string())?;

        Ok(json!(HttpResponse {
            status,
            headers,
            body,
        }))
    }
}