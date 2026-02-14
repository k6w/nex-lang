import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // The server is implemented in Rust
    const serverModule = context.asAbsolutePath(
        path.join('bin', 'nex-lsp.exe')
    );

    // If the extension is launched in debug mode then the debug server options are used
    // Otherwise the run options are used
    const serverOptions: ServerOptions = {
        command: serverModule,
        args: []
    };

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for NEX documents
        documentSelector: [{ scheme: 'file', language: 'nex' }],
        synchronize: {
            // Notify the server about file changes to '.nex' files contained in the workspace
            fileEvents: workspace.createFileSystemWatcher('**/*.nex')
        }
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'nexLanguageServer',
        'NEX Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}