import * as fs from "fs";
import dotenv from "dotenv";
import { Parser, Node, VariableDeclaration } from 'acorn';
import jsx from 'acorn-jsx';
const acorn = require("acorn");
const walk = require("acorn-walk");
const { extend } = require('acorn-jsx-walk');

dotenv.config();

export function getUpdatedFunctions(existingContents: string, newContents: string): string {
    let updatedFunctions = existingContents;
    const existingFunctions = extract_functions(existingContents);
    const newFunctions = extract_functions(newContents);
    const newFunction = newFunctions[0]
    const newFunctionName = newFunction.name;
    const newFunctionText = newContents.substring(newFunction.start, newFunction.end);
    for (const existingFunction of existingFunctions) {
        if (existingFunction.name == newFunctionName) {
            updatedFunctions = updatedFunctions.substring(0, existingFunction.start)
            + newFunctionText + updatedFunctions.substring(existingFunction.end);
        }
    }
    return updatedFunctions;
}
interface FunctionInfo {
    name: string;
    start: number;
    end: number;
}

function extract_functions(file_contents: string): FunctionInfo[] {
    // Initialize Acorn parser with JSX support
    const parser = Parser.extend(jsx());

    // Parse the file contents
    const ast = parser.parse(file_contents, {
        sourceType: 'module', // Assuming ES modules
        ecmaVersion: 'latest', // Use latest ECMAScript version
    });
    console.log(ast);
    //console.log((ast.body[1] as VariableDeclaration).declarations[0]);

    // Array to store function information
    const functions: FunctionInfo[] = [];
    extend(walk.base);
    walk.simple(ast, {
        FunctionDeclaration(node: { id: { name: string }, start: number, end: number }) {
          // Check if the function has a name
          if (node.id) {
            console.log("Function name:", node.id.name);
            functions.push({ name: node.id.name, start: node.start, end: node.end });
          }
        }
      });
      walk.simple(ast, {
        VariableDeclarator(node: { id: { name: string }, init: { type: string}, start: number, end: number }) {
          // Check if the function has a name
          if (node.id && node.init.type == "ArrowFunctionExpression") {
            console.log("Arrow function name:", node.id.name);
            functions.push({ name: node.id.name, start: node.start, end: node.end });
          }
        }
      });

    return functions;
}

function readFileContents(filePath: string): Promise<string> {
  return new Promise((resolve, reject) => {
    fs.readFile(filePath, "utf8", (err, data) => {
      if (err) {
        reject(err);
      } else {
        resolve(data);
      }
    });
  });
}

async function getFileContents(filePath: string): Promise<string> {
  try {
    const contents = await readFileContents(filePath);
    return contents;
  } catch (error) {
    console.error("Error reading file:", error);
    return ""; // or handle the error in a different way
  }
}

//getFileContents("./api/test-arrow.jsx").then(oldContents => {
    //console.log(oldContents);
    // Example usage
    /*const contents = `
    function sum(a, b) {
        return a + b;
    }

    const square = (x) => {
        return x * x;
    }
    `;

    const extractedFunctions = extract_functions(contents);
    console.log(extractedFunctions);*/

    /*getFileContents("./api/test-arrow-new.jsx").then(newContents => {
        console.log(getUpdatedFunctions(oldContents, newContents));
    })
});*/

