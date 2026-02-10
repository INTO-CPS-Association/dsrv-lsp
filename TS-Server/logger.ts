import * as vscode from "vscode";
let channel: vscode.OutputChannel | undefined;

export function initLogger(name = 'DynSRV') {
  if (!channel){
    channel = vscode.window.createOutputChannel(name);
  }
}

// Helper function to log messages to the output channel
export function log(message: string){
  channel?.appendLine(message); // Append the message to the output channel
}


// Helper function to log errors to the output channel
export function logError(error: unknown) {
  if (error instanceof Error) {
    channel?.appendLine(`[Error] ${error.message}`);
  } else {
    channel?.appendLine(`[Error] ${String(error)}`);
  }
}

export function show(){
  channel?.show(); // Show the output channel and preserve focus
}