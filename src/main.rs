use serde_json::{json, Value};
use std::fs;
use anyhow::{Context, Result}; // Importing Result and Context from anyhow crate
use dotenvy::dotenv;
mod library;
use library::prompt::prompt;
use library::extract_jsx::extract_jsx;
use library::log_and_run::log_and_run;
use library::get_feature::get_feature;
use library::remove_feature::remove_feature;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let src_path = "/home/emoryhubbardiv/Documents/tailwindify/src/app/";

    //let mut feature_data = json!({"feature": {...
    /*let feature_data_str = fs::read_to_string("long-test-feature.json")
      .with_context(|| format!("Failed to read file: {}", "long-test-feature.json"))
      .unwrap();
    let mut feature_data: Value = serde_json::from_str(&feature_data_str)
      .with_context(|| format!("Failed to deserialize JSON data from file: {}", "quick-test-feature.json"))
      .unwrap();*/

    let mut feature_data: Value = get_feature().await.unwrap();
    println!("Feature data: {:?}", feature_data);
    let doc_id = feature_data["docId"].as_str().unwrap().to_string();

    /*remove_feature(doc_id).await.unwrap();
    std::process::exit(0);*/

    let steps = &mut feature_data["steps"];

    for step in steps.as_array_mut().unwrap() {
      if let Err(err) = execute_step(step).await {
          let error_message = "Custom error message system not yet implemented.";
          eprintln!("Error executing step: {}", error_message);
          break;
      }
    }
    remove_feature(&doc_id).await.unwrap();
}
async fn execute_step_no_debug(step: &mut Value) -> Result<()> {
  if let Some(files) = step["files"].as_array_mut() {
      for file in files {
          add_file_contents(file);
      }
  }

  let full_prompt = get_prompt(&step);
  let api_key = std::env::var("API_KEY").context("API_KEY environment variable not found")?;
  let response = prompt(&full_prompt, &api_key).await?;
  println!("Response: {}", response);
  
  let js_content = extract_jsx(&response).await?;
  println!("Extracted code: {}", js_content);
  create_or_modify(&step, &js_content);

  Ok(())
}
async fn execute_step(step: &mut Value) -> Result<()> {
  if let Some(files) = step["files"].as_array_mut() {
      for file in files {
          add_file_contents(file);
      }
  }

  let mut passing = false;
  let mut code_attempts = Vec::new();
  let mut logs = Vec::new();
  let mut passing_responses = Vec::new();
  let mut curr_prompt = get_prompt(&step);
  let mut trimmed_code = String::new();
  let api_key = std::env::var("API_KEY").context("API_KEY environment variable not found")?;

  let max_attempts = 2;
  for i in 0..max_attempts {
      let code_attempt = prompt(&curr_prompt, &api_key).await?;
      code_attempts.push(code_attempt.clone());
      trimmed_code = extract_jsx(&code_attempt).await?;
      println!("trimmed_code: {}", trimmed_code);
      create_or_modify(&step, &trimmed_code)?;
      thread::sleep(Duration::from_secs(3));
      if let Some(test_path) = step["testPath"].as_str() {
        logs.push(log_and_run(test_path).await?);
      }
      passing_responses.push(get_passing_response(&trimmed_code, &logs[i], &curr_prompt, &api_key).await?);
      passing = is_passing(&passing_responses[i]);

      if !passing {
          curr_prompt = get_next_prompt(&trimmed_code, &logs[i], &curr_prompt, &passing_responses[i], &step);
      } else {
          break;
      }
  }
  if !passing {
    println!("{}", get_debug_details(&trimmed_code, &code_attempts, &logs, &passing_responses)?);
    anyhow::bail!("Debugging attempts failed. Aborting execution.");
  }
  Ok(())
}

fn add_file_contents(file: &mut Value) {
  let file_path = file["filePath"].as_str().unwrap();
  if let Ok(contents) = fs::read_to_string(file_path) {
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
          let file_path = file["filePath"].as_str().unwrap();
          let file_contents = file["fileContents"].as_str().unwrap_or("No file contents");
          prompt.push_str(&format!(" Here is the current {} located at {}: \"{}\"", file_name, file_path, file_contents));
      }
  }
  
  prompt
}
fn create_or_modify(step: &Value, new_contents: &String) -> Result<()> {
  let target_file_name = step["target"].as_str().context("Target file name not found in step")?;
  let files = step["files"].as_array().context("Files array not found in step")?;
  
  let target_file_path = files.iter()
      .find(|file| file["isTarget"].as_bool() == Some(true))
      .and_then(|file| file["filePath"].as_str())
      .context("Target file path not found in files")?;
  
  let existing_contents = match fs::read_to_string(target_file_path) {
      Ok(contents) => contents,
      Err(_) => String::new(), // File doesn't exist yet
  };
  
  fs::write(target_file_path, new_contents)
      .with_context(|| format!("Failed to write to file: {}", target_file_path))?;
  
  println!("File {} {}.", target_file_name,
           if existing_contents.is_empty() { "created" } else { "modified" });
  
  Ok(())
}

async fn get_passing_response(code: &str, logs: &str, user_prompt: &str, api_key: &String) -> Result<String> {
  let logs = if logs.is_empty() {
      "[no console log output was produced]".to_string()
  } else {
      logs.to_string()
  };
  println!("Logs from running the file: {}", logs);

  let response_prompt = format!("Here is the code: {}\n\nNote that it should be doing exactly what the user wanted, which was '{}'. Based on the following logs, does this code look like it ran properly? Console logs:\n{}\n[end of logs]\n\nIMPORTANT: Please include the word yes, or no, in your response for clarity, and explain why.", code, user_prompt, logs);
  let response = prompt(&response_prompt, &api_key).await?;

  println!("ChatGPT evaluation of logs: {}", response);
  Ok(response)
}

fn get_next_prompt(code: &str, logs: &str, user_prompt: &str, passing_response: &str, step: &Value) -> String {
  let logs = if logs.is_empty() {
      "[no console log output was produced]".to_string()
  } else {
      logs.to_string()
  };

  format!("There is a problem with this code:\n{}\n\nNote that it should be doing exactly what the user wanted, which was '{}'. Based on the following logs, the code didn't look like it ran properly: Console logs:\n{}\n\nIt was explained to me that '{}'. Could you write a new, corrected {}? Please include the whole file in your response.", code, user_prompt, logs, passing_response, step["target"])
}

fn is_passing(response: &str) -> bool {
  response.to_lowercase().contains("yes")
}
fn get_debug_details(trimmed_code: &str, code_attempts: &[String], logs: &[String], passing_responses: &[String]) -> Result<String> {
  let mut debug_details = String::from("Unable to generate properly working code. Debugging details:");
  for i in 0..code_attempts.len() {
      debug_details += &format!(
          "\n\nChatGPT Response {}:\n{}\n\nConsole logs from test run {}:\n{}\n\nChatGPT evaluation of logs {}:\n\nBased on the following logs, does this code look like it ran properly?\n\n{}",
          i + 1,
          code_attempts[i],
          i + 1,
          logs[i],
          i + 1,
          passing_responses[i]
      );
  }
  Ok(debug_details)
}