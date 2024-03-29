use anyhow::{Context, Result}; // Importing Result and Context from anyhow crate

pub async fn prompt(prompt: &str, api_key: &String) -> Result<String> {
    let client = reqwest::Client::new();
  
    let request_data = serde_json::json!({
        "model": "gpt-3.5-turbo",
        "messages": [{ "role": "user", "content": prompt }],
        "temperature": 0.7,
    });
  
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_data)
        .send()
        .await?
        .text()
        .await?;
    
    let response_data: serde_json::Value = serde_json::from_str(&response)?;
    let code = response_data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Response data does not contain expected content"))?;
  
    Ok(code.to_string())
  }