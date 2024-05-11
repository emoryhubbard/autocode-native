use serde_json::{json, Value};
use anyhow::{Context, Result};

pub async fn log_and_run(test_path: &str, show_html: &str) -> Result<String> {
    let url = format!("http://localhost:4000/api/log-and-run?testPath={}&showHTML={}", test_path, show_html);
  
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await?;
    
    let status = response.status();
    let logs = response.text().await?;
  
    if status.is_success() {
        return Ok(logs);
    }
  
    anyhow::bail!("Failed to log and run. Status code: {}", status)
}
