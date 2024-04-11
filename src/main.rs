use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::{clone, env, fs};
use anyhow::{Context, Result}; // Importing Result and Context from anyhow crate
use dotenvy::dotenv;
mod library;
use library::prompt::prompt;
use library::extract_jsx::extract_jsx;
use library::log_and_run::log_and_run;
use library::get_feature::get_feature;
use library::remove_feature::remove_feature;
use library::get_updated_functions::get_updated_functions;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use std::process::Command;

#[tokio::main]
async fn main() {
    /* Local JSON mode and Remote JSON mode: let mut feature_data = json!({"feature": {...
      Or uncomment the lines below to use json file */
    let feature_data_str = fs::read_to_string("feature.json")
      .with_context(|| format!("Failed to read file: {}", "feature.json"))
      .unwrap();
    let mut feature_data: Value = serde_json::from_str(&feature_data_str)
      .with_context(|| format!("Failed to deserialize JSON data from file: {}", "feature.json"))
      .unwrap();
    let feature_data_immut: Value = serde_json::from_str(&feature_data_str)
      .with_context(|| format!("Failed to deserialize JSON data from file: {}", "feature.json"))
      .unwrap();

    /* API JSON mode: uncomment the lines below, as well the remove
    feature line further down */
    /*let mut feature_data: Value = get_feature().await.unwrap();
    println!("Feature data: {:?}", feature_data);*/
    
    /* Remote JSON mode and API JSON mode: uncomment the line below, as well the remove
    feature line further down */
    let doc_id = feature_data["docId"].as_str().unwrap().to_string();
    
    let steps = &mut feature_data["steps"];
    let steps_immut = feature_data_immut["steps"].as_array().unwrap();

    // Create Autocode's own .env using feature data
    if !Path::new(".env").exists() {
      fs::File::create(".env")
          .expect("Failed to create .env file");
    }
    fs::write(".env", feature_data_immut["autocodeDotenv"].as_str().unwrap()).expect("Failed to write .env file");
    dotenv().ok();

    if std::env::var("CLONING").unwrap() == "true" {
      println!("Cloning reads true.");
      clone_autocode(feature_data_immut["autocodeDotenv"].as_str().unwrap(), feature_data_immut["serviceJSON"].as_str().unwrap()).await;
    }

    // Clone repository if needed
    let first_step = &steps_immut[0];
    let cloned_dir: Option<PathBuf> = if let Ok(cloning) = std::env::var("CLONING") {
      if cloning == "true" {
          if let Some(repo_url) = feature_data_immut["repoURL"].as_str() {
              Some(clone_repository(repo_url, feature_data_immut["dotenvContents"].as_str().unwrap(), first_step["testPath"].as_str().unwrap()).await.unwrap())
          } else {
              eprintln!("CLONING=true but repoURL is not provided in the feature data.");
              None
          }
      } else {
          None
      }
  } else {
      None
  };

  // Check if cloned_dir is uninitialized
  if cloned_dir.is_none() {
      // Handle uninitialized cloned_dir
      eprintln!("cloned_dir is uninitialized.");
      return;
  }
  let cloned_dir = cloned_dir.unwrap(); // Unwrap cloned_dir safely since we've checked it
  let mut successful = true;
  for step in steps.as_array_mut().unwrap() {
      if let Err(err) = execute_step(step, cloned_dir.clone()).await {
          successful = false;
          let error_message = "Custom error message system not yet implemented.";
          eprintln!("Error executing step: {}\n", error_message);
          break;
      }
  }

  if successful {
      println!("Feature completed. Tests passed at each step.\n");
      // Remote JSON mode and API JSON mode: uncomment the line below
      remove_feature(&doc_id).await.unwrap();
  }
}

async fn clone_autocode(dotenv_contents: &str, service_json: &str) -> Result<PathBuf> {
  let repo_url = "https://github.com/emoryhubbard/express-autocode-api.git";
  // Get the current directory
  let original_dir = std::env::current_dir()
      .with_context(|| "Failed to get current directory")?;

  // Get the parent directory of the current working directory
  let parent_dir = original_dir
      .parent()
      .ok_or_else(|| anyhow::anyhow!("Failed to determine parent directory"))?;

  // Define the directory path for cloning the repository
  let clone_dir = parent_dir.join("express-autocode-api");

  // Clone the repository into the parent directory
  let output = Command::new("git")
      .arg("clone")
      .arg(repo_url)
      .arg(&clone_dir)
      .output()
      .with_context(|| "Failed to execute git clone")?;
  
  if !output.status.success() {
      return Err(anyhow::anyhow!("Failed to clone repository"));
  }

  // Change current directory to the cloned repository
  std::env::set_current_dir(&clone_dir)
      .with_context(|| format!("Failed to change directory to {}", clone_dir.display()))?;
  // Write .env file
  let mut env_file = std::fs::File::create(".env")
      .with_context(|| "Failed to create .env file")?;
  env_file.write_all(dotenv_contents.as_bytes())
      .with_context(|| "Failed to write to .env file")?;

  // Write serviceAccountKey.json file
  let mut service_file = std::fs::File::create("serviceAccountKey.json")
    .with_context(|| "Failed to create service file")?;
  service_file.write_all(service_json.as_bytes())
    .with_context(|| "Failed to write to service file")?;

  // Run npm install
  let output = Command::new("npm")
      .arg("install")
      .output()
      .with_context(|| "Failed to execute npm install")?;
  
  if !output.status.success() {
      return Err(anyhow::anyhow!("Failed to install npm dependencies"));
  }

  /*let _output = tokio::process::Command::new("npm")
        .arg("run")
        .arg("dev")
        .output();*/
  // Run npm run dev in a terminal
  Command::new("gnome-terminal")
      .arg("--wait") // Add the --wait option to keep the terminal open
      .arg("--")
      .arg("npm")
      .arg("run")
      .arg("dev")
      .spawn()
      .with_context(|| "Failed to execute npm run dev in a terminal")?;
  thread::sleep(Duration::from_secs(6)); // giving TypeScript time to compile code

  env::set_current_dir(original_dir.clone())
      .with_context(|| format!("Failed to change directory to {}", original_dir.display()))?;
  Ok((clone_dir))
}

async fn clone_repository(repo_url: &str, dotenv_contents: &str, test_path: &str) -> Result<PathBuf> {
    // Get the current directory
    let current_dir = std::env::current_dir()
        .with_context(|| "Failed to get current directory")?;

    // Get the parent directory of the current working directory
    let parent_dir = current_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to determine parent directory"))?;

    // Define the directory path for cloning the repository
    let clone_dir = parent_dir.join("cloned_repo");

    // Clone the repository into the parent directory
    let output = Command::new("git")
        .arg("clone")
        .arg(repo_url)
        .arg(&clone_dir)
        .output()
        .with_context(|| "Failed to execute git clone")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to clone repository"));
    }

    // Change current directory to the cloned repository
    std::env::set_current_dir(&clone_dir)
        .with_context(|| format!("Failed to change directory to {}", clone_dir.display()))?;

    // Write .env file
    let mut env_file = std::fs::File::create(".env")
        .with_context(|| "Failed to create .env file")?;
    env_file.write_all(dotenv_contents.as_bytes())
        .with_context(|| "Failed to write to .env file")?;

    // Run npm install
    let output = Command::new("npm")
        .arg("install")
        .output()
        .with_context(|| "Failed to execute npm install")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to install npm dependencies"));
    }
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
    Ok((clone_dir))
}

async fn execute_step_test_edit(step: &mut Value) -> Result<()> {
  if let Some(files) = step["files"].as_array_mut() {
      for file in files {
          add_file_contents(file);
      }
  }
  let new_function_contents= r#"function Home() {
      // this should appear
      return (
          <>
          <title>Tailwindify Home Page</title>
          <Header />
          <Footer />
          <Script type='module' src="/js/home.js" />
          </>
      )
  }"#.to_string();
  println!("Extracted code: {}", new_function_contents);
  let _ = create_or_modify(&step, &new_function_contents).await;

  Ok(())
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
  let _ = create_or_modify(&step, &js_content).await;

  Ok(())
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
      //println!("\npassing_response: {}", passing_response);
      passing_responses.push(code_attempt.clone());
      passing = is_passing(&passing_responses[i]);
      println!("\ncode_attempt: {}", code_attempt);
      if !passing {
        //println!("\nlogs going to get_next_prompt: {}", &logs[i]);
        //curr_prompt = get_next_prompt(&trimmed_code, &logs[i], &user_prompt, &passing_responses[i], &step);
      } else {
          break;
      }
  }
  //println!("{}", get_debug_details(&trimmed_code, &code_attempts, &logs, &passing_responses)?);
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
fn extract_functions(file_contents: &str) -> Vec<String> {
  println!("Inside extract functions");
  let re = regex::Regex::new(r"function\s+(\w+)\s*\(([^)]*)\)\s*\{(.*?)\}")
      .unwrap();
  let mut functions = Vec::new();

  for capture in re.captures_iter(file_contents) {
      let function_name = capture.get(1).unwrap().as_str().to_string();
      let function_params = capture.get(2).unwrap().as_str().to_string();
      let function_body = capture.get(3).unwrap().as_str().to_string();

      let function_definition = format!(
          "function {}({}) {{\n{}\n}}",
          function_name, function_params, function_body
      );
      println!("Function definition: {}\n", function_definition);
      functions.push(function_definition);
  }

  functions
}

fn get_function_name(function_definition: &str) -> Option<String> {
  let re = regex::Regex::new(r"function\s+(\w+)\s*\(").unwrap();
  if let Some(capture) = re.captures(function_definition) {
      let function_name = capture.get(1).unwrap().as_str().to_string();
      Some(function_name)
  } else {
      None
  }
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