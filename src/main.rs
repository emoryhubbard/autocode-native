use serde_json::{json, Value};
use std::fs;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let src_path = "/home/emoryhubbardiv/Documents/tailwindify/src/app/";
    let mut job_data = json!({
        "job": {
          "description": "Add a Last-Visited indicator on the home page.",
          "steps": [
            {
              "description": "Add a function called setLastVisit to page.jsx that uses the setLocalStorage function in utils.mjs. to set lastVisit as the current time.",
              "target": "page.jsx",
              "files": [
                {
                  "fileName": "page.jsx",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/page.jsx",
                  "fileContents": "// Content of page.jsx file",
                  "isTarget": true
                },
                {
                  "fileName": "utils.mjs",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/components/utils.mjs",
                  "fileContents": "// Content of utils.mjs file",
                  "isTarget": false
                }
              ]
            },
            {
              "description": "Add a function called getLastVisit to page.jsx that uses the getLocalStorage function in utils.mjs.",
              "target": "page.jsx",
              "files": [
                {
                  "fileName": "page.jsx",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/page.jsx",
                  "fileContents": "// Content of page.jsx file",
                  "isTarget": true
                },
                {
                  "fileName": "utils.mjs",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/components/utils.mjs",
                  "fileContents": "// Content of utils.mjs file",
                  "isTarget": false
                }
              ]
            },
            {
              "description": "Add a prop called lastVisit to the useState in page.jsx and set its initial value with getLastVisit, and add that same lastVisit to the setAllValues function in useEffect as well.",
              "target": "page.jsx",
              "files": [
                {
                  "fileName": "page.jsx",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/page.jsx",
                  "fileContents": "// Content of page.jsx file",
                  "isTarget": true
                }
              ]
            },
            {
              "description": "In the useEffect hook in page.jsx, call setLastVisit with the current time.",
              "target": "page.jsx",
              "files": [
                {
                  "fileName": "page.jsx",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/page.jsx",
                  "fileContents": "// Content of page.jsx file",
                  "isTarget": true
                }
              ]
            },
            {
              "description": "Modify the h1 in page.jsx to say Transform Your Styles: Last Visited [insert time here]. Instead of insert time here, use the lastVisit prop.",
              "target": "page.jsx",
              "files": [
                {
                  "fileName": "page.jsx",
                  "filePath": "/home/emoryhubbardiv/Documents/tailwindify/src/app/page.jsx",
                  "fileContents": "// Content of page.jsx file",
                  "isTarget": true
                }
              ]
            }
          ]
        }
      }
      );

    let steps = &mut job_data["job"]["steps"];

    for step in steps.as_array_mut().unwrap() {
        execute_step(step).await;
    }
    //println!("{}", job_data);
    //prompt_test().await;
}
async fn execute_step(step: &mut Value) {
    //println!("Executing step: {}", step["description"]);

    if let Some(files) = step["files"].as_array_mut() {
        for file in files {
            add_file_contents(file);
        }
    }

    let full_prompt = get_prompt(&step);
    println!("Prompt: {}", full_prompt);
    let api_key = std::env::var("API_KEY").unwrap();
    let response = prompt(&full_prompt, api_key).await;
    println!("Response: {}", response);

}
fn add_file_contents(file: &mut Value) {
    let file_path = file["filePath"].as_str().unwrap();
    if let Ok(contents) = fs::read_to_string(file_path) {
        // Note: This modifies the original JSON data in memory
        file.as_object_mut().unwrap().insert("fileContents".to_string(), json!(contents));
    } else {
        println!("Error reading file: {}", file_path);
    }
}
fn get_prompt(step: &Value) -> String {
    let description = step["description"].as_str().unwrap();
    let mut prompt = format!("Could you write a new {} with this modification: \"{}\". In addition, could you write a simple console log statement within its code to verify the change is working?", step["target"], description);

    if let Some(files) = step["files"].as_array() {
        for file in files {
            let file_name = file["fileName"].as_str().unwrap();
            let file_contents = file["fileContents"].as_str().unwrap_or("No file contents");
            prompt.push_str(&format!(" Here is the current {}: \"{}\"", file_name, file_contents));
        }
    }
    
    prompt
}
async fn prompt_test() {
    let user_prompt = "Could you write a function that finds the first ten primes?";
    let api_key = std::env::var("API_KEY").unwrap();
    prompt(user_prompt, api_key).await;
}

async fn prompt(prompt: &str, api_key: String) -> String {
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
        .await
        .unwrap();
    let response_data: serde_json::Value = response.json().await.unwrap();
    let code = response_data["choices"][0]["message"]["content"].as_str().unwrap();
    //println!("Code: {}", code);

    code.to_string()
}