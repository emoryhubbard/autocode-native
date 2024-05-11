/*import { Page } from "puppeteer-core";
const chrome = require('@sparticuz/chromium');
const puppeteer = require('puppeteer-core');*/

import { Page } from "puppeteer";
const puppeteer = require('puppeteer');
const prettier = require('prettier');

export async function logAndRun(url: string, showHTML: string): Promise<string> {
    /*let path = '';
    if (process.env.PROD != 'false')
        path = await chrome.executablePath();
    if (process.env.PROD == 'false')
        path = '/usr/bin/google-chrome-stable';
    
        const browser = await puppeteer.launch({
        args: chrome.args,
        defaultViewport: chrome.defaultViewport,
        executablePath: path,
        headless: 'new',
        ignoreHTTPSErrors: true
      });*/
    
    const browser = await puppeteer.launch({
        headless: false, // Launch non-headless browser
        ignoreHTTPSErrors: true
    });
    
    let consoleLogOutput = '';
    try {
        const testRuns = 2; // multiple test runs in case testing eg. localStorage, see Autocode Design Decisions
        for (let i = 0; i < testRuns; i++) {
        const page = await browser.newPage();
            consoleLogOutput = await (evaluateWithTimeout(page, url, 60000, showHTML)) as string;
            //consoleLogOutput = await (page.evaluate(script)) as string;
        }
        await browser.close();
    }
    catch (e: any) {
        consoleLogOutput = e.message as string;
        if (browser && browser.process() != null) {
            browser.process()!.kill('SIGKILL');
        }
    }

    return consoleLogOutput;
}

async function evaluateWithTimeout(page: Page, url: string, timeout: number, showHTML: string): Promise<any> {
    let allLogs = '';

    // Attach event listener to capture console logs
    page.on('console', message => {
        if (!(message.text().includes("Warning: A future version of React will block javascript: URLs as a security precaution.")
        || message.text().includes("Download the React DevTools for a better development experience")
        || message.text().includes("The value \"product-width\" for key \"width\" is invalid, and has been ignored.")
        || message.text().includes("is found, but is not used because the request credentials mode does not match. Consider taking a look at crossorigin attribute.")
        || message.text().includes("was preloaded using link preload but not used within a few seconds from the window's load event.")
        || message.text().includes("Failed to load resource: the server responded with a status of 500 (Internal Server Error)")))
            allLogs += `${message.text()}\n`;
            //console.log("Logged Puppeteer console message: " + message.text())
    });
    page.on('pageerror', ({ message, stack}) => {allLogs += `${stack}\n`;
})
    let evaluationPromise = page.goto(url);

    await new Promise(r => setTimeout(r, 3000));

    let timeoutPromise = new Promise((resolve, reject) => {
        setTimeout(() => reject(new Error('Evaluation timed out')), timeout);
    });

    try {
        // Wait for either the evaluation or the timeout
        await Promise.race([evaluationPromise, timeoutPromise]);
        if (showHTML == 'true')
            allLogs += '\nLog of current page HTML content:\n' + await prettier.format(await page.content(), { parser: 'html' });
        return allLogs;
    } catch (error) {
        // Handle timeout error
        console.error('Evaluation timed out:', error);
        // Rethrow the error or handle it as needed
        throw error;
    }
}