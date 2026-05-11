import * as vscode from 'vscode';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

function registerCommands(context: vscode.ExtensionContext): void {
    // Register run command
    context.subscriptions.push(
        vscode.commands.registerCommand('zen.run', async (uriArg: vscode.Uri | string, functionName: string, lineNumber?: number) => {
            // Handle URI coming as string from LSP code lens
            const uri = typeof uriArg === 'string' ? vscode.Uri.parse(uriArg) : uriArg;
            const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }

            const outputChannel = vscode.window.createOutputChannel('Zen Run');
            outputChannel.show();
            outputChannel.appendLine(`▶ Running ${functionName}${lineNumber !== undefined ? ` (line ${lineNumber + 1})` : ''}...`);
            outputChannel.appendLine('---');

            try {
                const { stdout, stderr } = await execAsync(`zen "${uri.fsPath}"`, {
                    cwd: workspaceFolder.uri.fsPath,
                    maxBuffer: 1024 * 1024 * 10
                });
                if (stdout) outputChannel.appendLine(stdout);
                if (stderr) outputChannel.appendLine('STDERR:\n' + stderr);
                outputChannel.appendLine('---');
                outputChannel.appendLine(`✓ ${functionName} completed successfully`);
            } catch (error: unknown) {
                const message = error instanceof Error ? error.message : String(error);
                outputChannel.appendLine(`✗ Error running ${functionName}:`);
                outputChannel.appendLine(message);
                outputChannel.appendLine('---');
                vscode.window.showErrorMessage(`Failed to run ${functionName}: ${message}`);
            }
        })
    );

    // Register build command
    context.subscriptions.push(
        vscode.commands.registerCommand('zen.build', async (uriArg: vscode.Uri | string, functionName: string, lineNumber?: number) => {
            // Handle URI coming as string from LSP code lens
            const uri = typeof uriArg === 'string' ? vscode.Uri.parse(uriArg) : uriArg;
            const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }

            const outputChannel = vscode.window.createOutputChannel('Zen Build');
            outputChannel.show();
            outputChannel.appendLine(`🔨 Building ${functionName}${lineNumber !== undefined ? ` (line ${lineNumber + 1})` : ''}...`);
            outputChannel.appendLine('---');

            try {
                const filename = uri.fsPath.split('/').pop() || 'output';
                const output = filename.replace('.zen', '');
                const { stdout, stderr } = await execAsync(`zen "${uri.fsPath}" -o "${output}"`, {
                    cwd: workspaceFolder.uri.fsPath,
                    maxBuffer: 1024 * 1024 * 10
                });
                if (stdout) outputChannel.appendLine(stdout);
                if (stderr) outputChannel.appendLine('STDERR:\n' + stderr);
                outputChannel.appendLine('---');
                outputChannel.appendLine(`✓ Build completed successfully`);
            } catch (error: unknown) {
                const message = error instanceof Error ? error.message : String(error);
                outputChannel.appendLine(`✗ Build failed:`);
                outputChannel.appendLine(message);
                outputChannel.appendLine('---');
                vscode.window.showErrorMessage(`Build failed: ${message}`);
            }
        })
    );

    // Register test command
    context.subscriptions.push(
        vscode.commands.registerCommand('zen.runTest', async (uriArg: vscode.Uri | string, testName: string) => {
            // Handle URI coming as string from LSP code lens
            const uri = typeof uriArg === 'string' ? vscode.Uri.parse(uriArg) : uriArg;
            const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
            if (!workspaceFolder) {
                vscode.window.showErrorMessage('No workspace folder found');
                return;
            }

            const outputChannel = vscode.window.createOutputChannel('Zen Tests');
            outputChannel.show();
            outputChannel.appendLine(`▶ Running test: ${testName}...`);
            outputChannel.appendLine('---');

            try {
                const { stdout, stderr } = await execAsync(`zen test "${uri.fsPath}" --filter "${testName}"`, {
                    cwd: workspaceFolder.uri.fsPath,
                    maxBuffer: 1024 * 1024 * 10
                });
                if (stdout) outputChannel.appendLine(stdout);
                if (stderr) outputChannel.appendLine('STDERR:\n' + stderr);
                outputChannel.appendLine('---');
                outputChannel.appendLine(`✓ Test ${testName} passed`);
            } catch (error: unknown) {
                const message = error instanceof Error ? error.message : String(error);
                outputChannel.appendLine(`✗ Test ${testName} failed:`);
                outputChannel.appendLine(message);
                outputChannel.appendLine('---');
                vscode.window.showErrorMessage(`Test failed: ${message}`);
            }
        })
    );
}

export async function activate(context: vscode.ExtensionContext) {
    registerCommands(context);
    vscode.window.showInformationMessage('Zen rewrite extension active with syntax and command support.');
}

export async function deactivate(): Promise<void> {
}
