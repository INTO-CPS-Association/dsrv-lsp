import { exec } from "child_process";
import { promisify } from "util";
import { stderr } from "process";
import { log } from "./logger";
import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  InitializeParams,
  TextDocumentSyncKind,
  Diagnostic,
  DiagnosticSeverity,
  Range,
  Position
} from 'vscode-languageserver/node';
import { TextDocument } from "vscode-languageserver-textdocument";
const execAsync = promisify(exec);

// Convert Windows path to WSL path
function toWslPath(winPath: string): string {
  return winPath.replace(/\\/g, '/').replace(/^([A-Za-z]):/, (_, drive) => `/mnt/${drive.toLowerCase()}`);
}


// Exported function to be used in the VSCode extension
export async function validateDynSRVFile(filePath: string) {
  const wslFilepath = toWslPath(filePath);
  const dummyInput = "/home/emili/projects/dynsrv-vscode/server/TS-Server/dummy.input";
  const parserLoc = "~/projects/robosapiens-trustworthiness-checker/target/release/trustworthiness_checker";
  const command = `${parserLoc} --parser lalr --language dsrv --input-file ${dummyInput} ${wslFilepath}`;


  try {
    log(`Executing parser with command: ${command}`);
    await execAsync(command);
    return [];
  } catch (error: any) {
    log(`Error executing parser: ${error.message}\nStderr: ${error.stderr}`);
    return getDiagnostics(error.stderr);
  }
}


// Helper function to parse diagnostics from the parser's stderr output
function getDiagnostics(stderr: string): Diagnostic[] {
  const output = process.stdout;
  const diagnostics: Diagnostic[] = [];

  const errorRegex = /\d+:\s+(.*)\s+found at (\d+):(\d+)/g;
  let match;

  while ((match = errorRegex.exec(stderr)) !== null) {
    const line = parseInt(match[1]) - 1;
    const character = parseInt(match[2]) - 1;


    diagnostics.push({
      severity: DiagnosticSeverity.Error,
      range: Range.create(line, character, line, character + 1),
      message: "Syntax error: Failed to parse the file.",
      source: 'RoboSAPIENS'
    });
    log(`Parsed diagnostic: Line ${line + 1}, Character ${character + 1}`);
  }

  return diagnostics;
}