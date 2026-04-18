# Language Server
This is the Rust language server for the DSRV language extension. The server is responsible for providing language features such as code completion, diagnostics, and hover information to the client. It utilizes the `tower-lsp-server` crate to facilitate communication with the client and to implement the language server protocol. 

The main entry point for the server is the `main` function, which initializes the server and starts listening for incoming requests from the client. The server implements various methods to handle different types of requests, such as `initialize`, `shutdown`, and `textDocument/didOpen`.

The server manages the state of the language features and interacts with the underlying DSRV language processing logic to provide accurate and efficient responses to the client's requests. To compile the server to run using the extension host, you can use the `cargo build debug` command in the terminal or the npm script defined in the `package.json` file. This will build the server and make it ready for use with the client.

