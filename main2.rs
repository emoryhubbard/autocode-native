use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::{clone, env, fs};
use anyhow::{Context, Result}; // Importing Result and Context from anyhow crate
use dotenvy::dotenv;
mod library;
use library::prompt::prompt;
use library::extract_jsx::extract_jsx;
use library::log_and_run::log_and_run;
use library::get_updated_functions::get_updated_functions;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[tokio::main]
async fn execute_steps() {
    let feature_data_str = fs::read_to_string("feature.json")
      .with_context(|| format!("Failed to read file: {}", "feature.json"))
      .unwrap();
    let mut feature_data: Value = serde_json::from_str(&feature_data_str)
      .with_context(|| format!("Failed to deserialize JSON data from file: {}", "feature.json"))
      .unwrap();
    let feature_data_immut: Value = serde_json::from_str(&feature_data_str)
      .with_context(|| format!("Failed to deserialize JSON data from file: {}", "feature.json"))
      .unwrap();
    let steps = &mut feature_data["steps"];
    let steps_immut = feature_data_immut["steps"].as_array().unwrap();

    // Overwrite environment variables with autocodeDotenv values
    if let Some(autocode_dotenv) = feature_data_immut["autocodeDotenv"].as_object() {
        for (key, value) in autocode_dotenv {
            env::set_var(key, value.as_str().unwrap());
        }
    }

    let parent_dir = Path::new(".").canonicalize().unwrap().parent().unwrap().to_owned();

    // URL of a form like "https://github.com/emoryhubbard/tailwindify.git"
    // Extract repository name using regex
    let re = Regex::new(r"/([^/]+)\.git$").unwrap();
    let repo_name = re.captures(feature_data_immut["repoURL"]).unwrap().get(1).unwrap().as_str();

    // Get the directory where the project is stored
    let project_dir = parent_dir.join(repo_name);
    let first_step = &steps_immut[0];
    preliminary_repo_test(repo_name, feature_data_immut["dotenvContents"] ,first_step["testPath"]);

  let mut successful = true;
  for step in steps.as_array_mut().unwrap() {
      if let Err(err) = execute_step(step, project_dir.clone()).await {
          successful = false;
          let error_message = "Custom error message system not yet implemented.";
          eprintln!("Error executing step: {}\n", error_message);
          break;
      }
  }

  if successful {
      println!("Feature completed. Tests passed at each step.\n");
  }
}

async fn preliminary_repo_test(repo_name: $str, dotenv_conents: &str, test_path: &str) -> Result<PathBuf> {
    // Get the current directory
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;

    // Get the parent directory of the current working directory
    let parent_dir = current_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to determine parent directory"))?;

    let project_dir = parent_dir.join(repo_name);

    std::env::set_current_dir(&project_dir)
        .with_context(|| format!("Failed to change directory to {}", clone_dir.display()))?;

    // Write .env file
    let mut env_file = std::fs::File::create(".env")
        .with_context(|| "Failed to create .env file")?;
    env_file.write_all(dotenv_contents.as_bytes())
        .with_context(|| "Failed to write to .env file")?;

    // Run npm run dev in a terminal
    Command::new("gnome-terminal")
        .arg("--wait") // Add the --wait option to keep the terminal open
        .arg("--")
        .arg("npm")
        .arg("run")
        .arg("dev")
        .spawn()
        .with_context(|| "Failed to execute npm run dev in a terminal")?;

    thread::sleep(Duration::from_secs(6)); // giving NextJS time to compile code
    let _ = log_and_run(test_path, "false").await;
    Ok((project_dir))
}

async fn execute_step(step: &mut Value, cloned_dir: PathBuf) -> Result<()> {
  if let Some(files) = step["files"].as_array_mut() {
      for file in files {
        if std::env::var("CLONING").unwrap() == "true" {
          add_full_path(file, cloned_dir.clone());
        }
        add_file_contents(file);
      }
  }

  let mut passing = false;
  let mut code_attempts = Vec::new();
  let mut logs = Vec::new();
  let mut passing_responses = Vec::new();
  let mut curr_prompt = get_prompt(&step);
  let mut user_prompt = get_prompt(&step);
  let mut trimmed_code = String::new();
  let api_key = std::env::var("CHATGPT_APIKEY").context("API_KEY environment variable not found")?;

  println!("\ncurr_prompt: {}", &curr_prompt);
  let mut code_attempt = prompt(&curr_prompt, &api_key).await?;
  println!("\ncode_attempt: {}", code_attempt);

  let max_attempts = 3;
  for i in 0..max_attempts {
      code_attempts.push(code_attempt.clone());
      trimmed_code = extract_jsx(&code_attempt).await?;
      println!("\ntrimmed_code: {}", trimmed_code);
      create_or_modify(&step, &trimmed_code).await?;
      thread::sleep(Duration::from_secs(3)); // giving NextJS time to compile changed code
      if let Some(test_path) = step["testPath"].as_str() {
        let curr_logs = log_and_run(test_path, &step["showHTML"].as_str().unwrap().to_lowercase()).await.unwrap();
        println!("\ncurr_logs: {}", curr_logs);
        logs.push(curr_logs);
      }
      code_attempt = get_passing_response(&trimmed_code, &logs[i], &curr_prompt, &api_key, step["target"].as_str().unwrap()).await?;
      passing_responses.push(code_attempt.clone());
      passing = is_passing(&passing_responses[i]);
      println!("\ncode_attempt: {}", code_attempt);
      if passing {
        break;
      }
  }
  if !passing {
    //println!("{}", get_debug_details(&trimmed_code, &code_attempts, &logs, &passing_responses)?);
    anyhow::bail!("Debugging attempts failed. Aborting execution.");
  }
  Ok(())
}

fn add_full_path(file: &mut Value, cloned_dir: PathBuf) {
  let file_path = file["filePath"].as_str().unwrap();
  let updated_file_path = cloned_dir.join(file_path);
  // Update the "filePath" field in the file object
  file["filePath"] = json!(updated_file_path.to_string_lossy());
  println!("Updated filePath: {}", file["filePath"]);
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
  let mut prompt = format!("Could you write a new {} with this modification: \"{}\". In addition, could you write a simple console log statement(s) within its code to verify the change is working, which is highly likely to run (not lost in a function that isn't called)?", step["target"], description);

  if let Some(files) = step["files"].as_array() {
      for file in files {
          let file_name = file["fileName"].as_str().unwrap();
          let file_path = file["filePath"].as_str().unwrap();
          let file_contents = file["fileContents"].as_str().unwrap_or("No file contents");
          prompt.push_str(&format!(" Here is the current {} located at {}: \"{}\"\n", file_name, file_path, file_contents));
      }
  }
  
  prompt
}
async fn create_or_modify(step: &Value, new_contents: &String) -> Result<()> {
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

    // Check if the new_contents is less than 50% of the existing_contents
    let new_lines = new_contents.lines().count();
    let existing_lines = existing_contents.lines().count();
    if new_lines < existing_lines / 2 {
        // Replace the existing function with the new one
        fs::write(target_file_path, get_updated_functions(&existing_contents, new_contents).await.unwrap())
            .with_context(|| format!("Failed to write to file: {}", target_file_path))?;
    } else {
        // Write the new_contents to the file
        fs::write(target_file_path, new_contents)
            .with_context(|| format!("Failed to write to file: {}", target_file_path))?;
    }

    println!(
        "File {} {}.",
        target_file_name,
        if existing_contents.is_empty() { "created" } else { "modified" }
    );

    Ok(())
}
async fn get_passing_response(code: &str, logs: &str, user_prompt: &str, api_key: &String, target: &str) -> Result<String> {
  let logs = if logs.is_empty() {
      "[no console log output was produced]".to_string()
  } else {
      logs.to_string()
  };
  //println!("Logs from running the file: {}", logs);

  let response_prompt = format!("Here is the code: {}\n\nNote that it should be doing exactly what the user wanted, which was '{}'. Based on the following logs, does this code look like it ran properly? (Note in React it is normal if logs repeat twice on component initialization) Console logs:\n{}\n[end of logs]\n\nIMPORTANT: Please include the word yes, or no, in your response for clarity, explain why, and provide a corrected \"{}\", if necessary (include any missing function calls, especially if the logs are empty yet functions are defined, in your corrected \"{}\").", code, user_prompt, logs, target, target);
  let response = prompt(&response_prompt, &api_key).await?;

  //println!("ChatGPT evaluation of logs: {}", response);
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