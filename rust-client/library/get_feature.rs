use serde_json::{json, Value};
use anyhow::{Context, Result};

pub async fn get_feature() -> Result<Value> {
    let url = "http://localhost:4000/api/get-feature";
  
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await?;
    
    let status = response.status();
    let feature_text = response.text().await?;
    println!("Logs: {}", feature_text);
  
    if status.is_success() {
        let feature_json: Value = serde_json::from_str(&feature_text)
            .context("Failed to parse JSON response")?;
        return Ok(feature_json);
    }
  
    anyhow::bail!("Failed to get feature. Status code: {}", status)
}
