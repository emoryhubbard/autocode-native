import express, { Express, Request, Response, NextFunction } from "express";
import dotenv from "dotenv";
import fs from "fs"; // Import the fs module
import { promisify } from 'util';
import { resolve } from 'path';

dotenv.config();
const app: Express = express();

app.use(express.json());

app.use((req: Request, res: Response, next: NextFunction) => {
    res.header('Access-Control-Allow-Origin', "*");
    res.header('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE');
    res.header('Access-Control-Allow-Headers', 'Content-Type');
    next();
});

const readFileAsync = promisify(fs.readFile);
const writeFileAsync = promisify(fs.writeFile);

async function generateExecuteStepsFile(): Promise<string> {
    try {
        // Read the contents of execute-steps.ts from the library directory
        const templateFilePath = resolve(__dirname, 'library', 'execute-steps.ts');
        const fileContent = await readFileAsync(templateFilePath, 'utf8');
        // Replace the placeholder with the customPrompt
        const updatedContent = fileContent.replace(/let PROMPT = '';/, `let PROMPT = \`${process.env[process.env.PROMPT as string]}\`;`);

        // Write the updated content to execute-steps-temp.ts
        const tempFilePath = resolve(__dirname, 'library', 'execute-steps-temp.ts');
        await writeFileAsync(tempFilePath, updatedContent);

        return tempFilePath;
    } catch (error) {
        console.error('Error generating execute-steps-temp.ts:', error);
        return '';
    }
}

async function importAndUseExecuteSteps(tempFilePath: string, feature: any): Promise<any> {
    try {
        // Dynamically import the executeSteps function from the generated file
        const { executeSteps } = await import(tempFilePath);

        // Call the imported function
        const status = await executeSteps(feature);
        return status;
    } catch (error) {
        console.error('Error importing and using executeSteps:', error);
        return 'Error executing steps';
    }
}

app.post("/api/execute-steps", async (req, res) => {
    console.log('Inside execute-steps endpoint');
    if (req.body.feature["autocodeDotenv"]) {
      for (const [key, value] of Object.entries(req.body.feature["autocodeDotenv"])) {
          process.env[key] = value as string;
      }
  }
    //const customPrompt = 'Could you write a new ${step["target"]} with this modification: "${step["description"]}". In addition, could you write a simple console log statement(s) within its code to verify the change is working, which is highly likely to run (not lost in a function that isn\'t called)?';

    try {
        // Generate execute-steps-temp.ts
        const executeStepsTempFilePath = await generateExecuteStepsFile();
        console.log(`Generated ${executeStepsTempFilePath}`);

        // Import and use executeSteps function
        const status = await importAndUseExecuteSteps(executeStepsTempFilePath, req.body.feature);

        res.status(200).json({ status });
    } catch (error) {
        console.error('Error:', error);
        res.status(500).json({ error: 'Internal server error' });
    }
});

// Start the server initially
const port = parseInt(process.env.PORT || "3000");

app.listen(port, () => {
    console.log(`Server is running on port ${port}`);
});

// Run Autocode with the default feature setting if set
if (process.env.FEATURE) {
    // Read the filename from the FEATURE environment variable
    const fileName = process.env.FEATURE;

    // Read the JSON from the file and save it to a variable called feature
    fs.readFile(fileName, 'utf8', async (err, data) => {
        if (err) {
            console.error("Error reading file:", err);
            return;
        }
        try {
            const feature = JSON.parse(data);
            console.log("Feature loaded:", feature);
            // Call execute-steps endpoint with the feature
            let response = await fetch(`http://localhost:${port}/api/execute-steps`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ feature })
            });
            console.log(await response.json());
        } catch (error) {
            console.error("Error parsing JSON:", error);
        }
    });
}
