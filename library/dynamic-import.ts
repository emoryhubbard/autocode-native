import fs from "fs"; // Import the fs module
import { promisify } from 'util';
import { resolve } from 'path';

const readFileAsync = promisify(fs.readFile);
const writeFileAsync = promisify(fs.writeFile);

export default async function dynamicImport(filePath: string): Promise<any> {
    try {
        // Read the contents of the file
        const variableNames = Object.keys(process.env);

        function isConstant(str: string): boolean { return variableNames.includes(str); }
        
        function containsConstant(str: string): boolean {
            return variableNames.some(constant => str.includes(constant));
        }
        
        function getConstants(str: string): string[] {
            const constants = str.match(/[A-Z][A-Z0-9_]*/g) || [];
            return constants.filter(constant => variableNames.includes(constant));
        }
        function recursiveReplace(envValue: string): string {
            if (envValue !== undefined && !isConstant(envValue) && !containsConstant(envValue)) 
                return envValue;
            if (envValue !== undefined && isConstant(envValue))
                return recursiveReplace(process.env[envValue] as string);
            if (envValue !== undefined && !isConstant(envValue) && containsConstant(envValue)) {
                const constants = getConstants(envValue);
                let updatedString = envValue;
                constants.forEach((constant: string) => {
                    updatedString = updatedString.replace(new RegExp(`${constant}`),
                        recursiveReplace(process.env[constant] as string));
                });
                return updatedString;
            }
            return "";
        }

        // A simpler recursion system that doesn't replace env variables within text strings
        /*function recursiveReplace(envValue: string) {
            if (envValue !== undefined && !isConstant(envValue)) 
                return envValue;
            if (envValue !== undefined && isConstant(envValue))
                return recursiveReplace(process.env[envValue] as string);
        }*/
        // Replace the variables with corresponding environment variable values
        const fileContent = await readFileAsync(filePath, 'utf8');
        let updatedContent = fileContent;
        variableNames.forEach((variableName: string) => {
            let finalValue = recursiveReplace(process.env[variableName] as string);

            // For replacing environment variables that are in template literal form
            updatedContent = updatedContent.replace(
                new RegExp(`const ${variableName} = '';`),
                `const ${variableName} = \`${finalValue}\`;`);

            function validJson(str: string): boolean {
                try {JSON.parse(str); return true;} catch {} return false;
            }
            
            // For replacing environment variables that are in expression form
            /*if (!validJson(finalValue))
                return;
            finalValue = JSON.parse(finalValue);*/
            updatedContent = updatedContent.replace(
                new RegExp(`const ${variableName} = {} as any;`),
                `const ${variableName} = ${finalValue};`);
        });

        /*function addDir(filePath: string, directory: string): string {
            const lastSlashIndex = filePath.lastIndexOf('/');
            if (lastSlashIndex === -1) {
                // If there is no directory path, just prepend the given directory to the file name
                return `${directory}/${filePath}`;
            } else {
                // Insert the directory before the last slash
                return `${filePath.slice(0, lastSlashIndex)}/${directory}/${filePath.slice(lastSlashIndex + 1)}`;
            }
        }*/

        // Write the updated content to a temporary file
        //const tempFilePath = resolve(__dirname, `temp.ts`);
        //let tempFilePath = addDir(filePath, 'temp');
        let tempFilePath = filePath.slice(0,-3) + '-temp.ts';
        await writeFileAsync(tempFilePath, updatedContent);

        // Import and return the dynamic code
        const dynamicModule = await import(tempFilePath);
        return dynamicModule;
    } catch (error) {
        console.error('Error during dynamic import:', error);
        throw error;
    }
}