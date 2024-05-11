use serde_json::{json, Value};
use anyhow::{Context, Result};

pub async fn remove_feature(doc_id: &str) -> Result<()> {
    let url = format!("http://localhost:4000/api/remove-feature?doc-id={}", doc_id);
  
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .send()
        .await?;
    
    let status = response.status();
  
    if status.is_success() {
        println!("Feature removed successfully");
        return Ok(());
    }
  
    anyhow::bail!("Failed to remove feature. Status code: {}", status)
}
