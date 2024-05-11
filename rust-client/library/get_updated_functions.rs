use serde_json::{json, Value};
use anyhow::{Context, Result}; // Importing Result and Context from anyhow crate

pub async fn get_updated_functions(existing_contents: &str, new_contents:&str) -> Result<String> {
    let body = json!({ "existingContents": existing_contents, "newContents": new_contents });
  
    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:4000/api/get-updated-functions")
        .json(&body)
        .send()
        .await?;
    
    let status = response.status();
    let text = response.text().await?;
  
    if status.is_success() {
        let json_response: serde_json::Value = serde_json::from_str(&text)?;
        if let Some(js_str) = json_response.get("JSX").and_then(|s| s.as_str()) {
            return Ok(js_str.to_string());
        }
    }
  
    anyhow::bail!("Failed to get updated functions using API. Status code: {}", status)
  }