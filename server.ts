import express, { Express, Request, Response, NextFunction } from "express";
import dotenv from "dotenv";
import fs from "fs"; // Import the fs module
import { promisify } from 'util';
import { resolve } from 'path';
import  dynamicImport from './library/dynamic-import';

dotenv.config();
const app: Express = express();

app.use(express.json());

app.use((req: Request, res: Response, next: NextFunction) => {
    res.header('Access-Control-Allow-Origin', "*");
    res.header('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE');
    res.header('Access-Control-Allow-Headers', 'Content-Type');
    next();
});

function setDotenv(feature: any) {
  if (feature["autocodeDotenv"]) {
    for (const [key, value] of Object.entries(feature["autocodeDotenv"])) {
        process.env[key] = value as string;
    }
  }
}

app.post("/api/execute-steps", async (req, res) => {
    console.log('Inside execute-steps endpoint');
    setDotenv(req.body.feature);
    try {
        // Import and execute dynamic code
        const { executeSteps } = await dynamicImport(resolve(__dirname, 'library', 'execute-steps.ts'));
        const status = await executeSteps(req.body.feature);

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
    fs.readFile(process.env.FEATURE, 'utf8', async (err, data) => {
        if (err) {
            console.error("Error reading file:", err);
            return;
        }
        try {
            const feature = JSON.parse(data);
            console.log("Feature loaded:", feature);
            // Call execute-steps endpoint with the feature
            let response = await fetch(`http://localhost:${port}/api/execute-steps`, {method: 'POST', headers: {'Content-Type': 'application/json'},
                body: JSON.stringify({ feature }) });
            console.log(await response.json());
        } catch (error) {
            console.error("Error parsing JSON:", error);
        }
    });
}
