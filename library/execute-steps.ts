import { extractJSX } from "../library/extractjsx";
import { prompt } from "../library/prompt";
import { logAndRun } from "../library/log-and-run";
import { getUpdatedFunctions } from "./get-updated-functions";
import * as fs from "fs";
import * as dotenv from "dotenv";
import * as path from "path";
import * as child_process from "child_process";

dotenv.config();

export async function executeSteps(feature: any) {
    let steps = feature["steps"];

    if (feature["autocodeDotenv"]) {
        for (const [key, value] of Object.entries(feature["autocodeDotenv"])) {
            process.env[key] = value as string;
        }
    }

    const parentDir = path.resolve(__dirname, "..");
    const repoURL = feature["repoURL"];
    const repoName = repoURL.match(/\/([^/]+)\.git$/)?.[1];

    if (!repoName) {
        throw new Error("Failed to extract repository name from repoURL");
    }

    const repoDir = path.join(parentDir, repoName);

    const firstStep = steps[0];
    await preliminaryRepoTest(repoName, feature["dotenvContents"], firstStep["testPath"]);
    if (process.env.REACT_STRICT_MODE == "false")
        await modifyNextConfig(repoName);

    let successful = true;
    for (const step of steps) {
        try {
            await executeStep(step, repoDir);
        } catch (err) {
            successful = false;
            console.error("Error executing step: ");
            console.log(err);
            break;
        }
    }

    let status = "Feature completed. Tests passed at each step."
    if (!successful) {
        status = "Unable to get all tests to pass. See console logs for details.";
    }
    return status;
}

async function preliminaryRepoTest(repoName: string, dotenvContents: string, testPath: string) {
    const currentDir = process.cwd();
    const parentDir = path.resolve(currentDir, "..");
    const repoDir = path.join(parentDir, repoName);

    process.chdir(repoDir);
    fs.writeFileSync(".env", dotenvContents);

    // Spawn the terminal process, but don't wait for it
    const terminalProcess = child_process.spawn("gnome-terminal", ["--", "npm", "run", "dev"], {
        detached: true, // Detach the child process
        stdio: "ignore" // Ignore stdio (standard I/O)
    });

    terminalProcess.unref(); // Unreference the child process to allow the parent to exit independently

    await new Promise(resolve => setTimeout(resolve, 6000));
    await logAndRun(testPath, "false");
}

async function modifyNextConfig(repoName: string) {
    const currentDir = process.cwd();
    const parentDir = path.resolve(currentDir, "..");
    const repoDir = path.join(parentDir, repoName);
    const nextConfigPath = path.join(repoDir, "next.config.js");

    try {
        // Read the content of next.config.js
        let nextConfigContent = fs.readFileSync(nextConfigPath, "utf-8");

        // Regex to find reactStrictMode: false
        const reactStrictModeFalseRegex = /reactStrictMode:\s*false/;
        // Regex to find reactStrictMode: true
        const reactStrictModeTrueRegex = /reactStrictMode:\s*true/;
        // Regex to find const nextConfig = {
        const nextConfigStartRegex = /const\s+nextConfig\s*=\s*{/;

        // Check if reactStrictMode: false is present
        if (!reactStrictModeFalseRegex.test(nextConfigContent)) {
            // If reactStrictMode: true is present, replace it with reactStrictMode: false
            if (reactStrictModeTrueRegex.test(nextConfigContent)) {
                nextConfigContent = nextConfigContent.replace(reactStrictModeTrueRegex, "reactStrictMode: false");
            } else {
                // If neither reactStrictMode: true nor reactStrictMode: false is present,
                // add reactStrictMode: false after const nextConfig = {
                nextConfigContent = nextConfigContent.replace(nextConfigStartRegex, "const nextConfig = { reactStrictMode: false ");
            }

            // Write the modified content back to next.config.js
            fs.writeFileSync(nextConfigPath, nextConfigContent);
            console.log("next.config.js modified successfully.");
        } else {
            console.log("reactStrictMode: false already exists in next.config.js. No modification needed.");
        }
    } catch (err) {
        console.error("Error modifying next.config.js:", err);
    }
}

async function executeStep(step: any, repoDir: string) {
    const files = step["files"];
    for (const file of files) {
        if (process.env.CLONING === "true") {
            addFullPath(file, repoDir);
        }
        addFileContents(file);
    }

    let passing = false;
    const codeAttempts: string[] = [];
    const logs: string[] = [];
    const passingResponses: string[] = [];
    let currPrompt = getPrompt(step);
    let trimmedCode = "";

    console.log("\ncurr_prompt:", currPrompt);
    let codeAttempt = await prompt(currPrompt, process.env.CHATGPT_APIKEY as string);

    console.log("\ncode_attempt:", codeAttempt);

    const maxAttempts = 3;
    for (let i = 0; i < maxAttempts; i++) {
        codeAttempts.push(codeAttempt);
        trimmedCode = await extractJSX(codeAttempt);
        console.log("\ntrimmed_code:", trimmedCode);
        await createOrModify(step, trimmedCode);
        await new Promise(resolve => setTimeout(resolve, 3000));
        const testPath = step["testPath"];
        const currLogs = await logAndRun(testPath, step["showHTML"].toLowerCase());
        console.log("\ncurr_logs:", currLogs);
        logs.push(currLogs);
        codeAttempt = await getPassingResponse(trimmedCode, logs[i], currPrompt, step["target"]);
        passingResponses.push(codeAttempt);
        passing = isPassing(passingResponses[i]);
        console.log("\ncode_attempt:", codeAttempt);
        if (passing) {
            break;
        }
    }
    if (!passing) {
        throw new Error("Debugging attempts failed. Aborting execution.");
    }
}

function addFullPath(file: any, repoDir: string) {
    const filePath = file["filePath"];
    const updatedFilePath = path.join(repoDir, filePath);
    file["filePath"] = updatedFilePath;
    console.log("Updated filePath:", file["filePath"]);
}

function addFileContents(file: any) {
    const filePath = file["filePath"];
    try {
        const contents = fs.readFileSync(filePath, "utf-8");
        file["fileContents"] = contents;
    } catch (err) {
        console.log("Error reading file:", filePath);
    }
}

function getPrompt(step: any) {
    let prompt = `Could you write a new ${step["target"]} with this modification: "${step["description"]}". In addition, could you write a simple console log statement(s) within its code to verify the change is working, which is highly likely to run (not lost in a function that isn't called)?`;

    const files = step["files"];
    for (const file of files) {
        const fileName = file["fileName"];
        const filePath = file["filePath"];
        const fileContents = file["fileContents"] ?? "No file contents";
        prompt += ` Here is the current ${fileName} located at ${filePath}: "${fileContents}"\n`;
    }
  
    return prompt;
}

async function createOrModify(step: any, newContents: string) {
    const targetFileName = step["target"];
    const files = step["files"];
    const targetFile = files.find((file: any) => file["isTarget"]);

    if (!targetFile) {
        throw new Error("Target file not found in step");
    }

    const targetFilePath = targetFile["filePath"];
    let existingContents = "";

    try {
        existingContents = fs.readFileSync(targetFilePath, "utf-8");
    } catch (err) {
        // File doesn't exist yet
    }

    const newLines = newContents.split("\n").length;
    const existingLines = existingContents.split("\n").length;
    if (newLines < existingLines / 2) {
        const updatedFunctions = await getUpdatedFunctions(existingContents, newContents);
        fs.writeFileSync(targetFilePath, updatedFunctions);
    } else {
        fs.writeFileSync(targetFilePath, newContents);
    }

    console.log(`File ${targetFileName} ${existingContents ? "modified" : "created"}.`);
}

async function getPassingResponse(code: string, logs: string, userPrompt: string, target: string) {
    logs = logs || "[no console log output was produced]";

    const responsePrompt = `Here is the code: ${code}\n\nNote that it should be doing exactly what the user wanted, which was '${userPrompt}'. Based on the following logs, does this code look like it ran properly? Console logs:\n${logs}\n[end of logs]\n\nIMPORTANT: Please include the word yes, or no, in your response for clarity, explain why, and provide a corrected "${target}", if necessary (include any missing function calls, especially if the logs are empty yet functions are defined, in your corrected "${target}").`;

    const response = await prompt(responsePrompt, process.env.CHATGPT_APIKEY as string);

    return response;
}

function isPassing(response: string) {
    return response.toLowerCase().includes("yes");
}
