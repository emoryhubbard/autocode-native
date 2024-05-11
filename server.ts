import express, { Express, Request, Response, NextFunction } from "express";
import dotenv from "dotenv";
import fs from "fs"; // Import the fs module
import { executeSteps } from "./library/execute-steps";
const bodyParser = require('body-parser');

dotenv.config();

const app: Express = express();

app.use(bodyParser.json());
app.use((req: Request, res: Response, next: NextFunction) => {
    res.header('Access-Control-Allow-Origin', "*");
    res.header('Access-Control-Allow-Methods', 'GET,PUT,POST,DELETE');
    res.header('Access-Control-Allow-Headers', 'Content-Type');
    next();
});

// Define execute-steps API endpoint which will change the port if feature demands it
app.post("/api/execute-steps", async (req, res) => {
  console.log('Inside execute-steps endpoint');
  let status = "Not set";
  status = await executeSteps(req.body.feature);
  res.status(200).json({ status: status });
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
      console.log(await response.json())
    } catch (error) {
      console.error("Error parsing JSON:", error);
    }
  });
}