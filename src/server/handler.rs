/*
 * Copyright (c) 2026 Emilie Bang Holmberg (github.com/EmmiPigen).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License.
 *
 * This project utilizes the 'trustworthiness-checker' crate, which is
 * property of the INTO-CPS Association and used under the ICAPL (GPL Mode).
 */

use crate::server::backend::Backend;
use tower_lsp_server::{LanguageServer, jsonrpc::Result, ls_types::*};

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "DSRV Language Server".to_string(),
                version: Some("1.0".to_string()),
            }),

            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    all_commit_characters: None,
                    trigger_characters: Some(vec![".".to_string()]),
                    completion_item: None,
                    work_done_progress_options: Default::default(),
                    ..Default::default()
                }),

                hover_provider: Some(HoverProviderCapability::Options(HoverOptions {
                    ..Default::default()
                })),

                // TODO: Add support for definition and declaration providers in the future to enable jumping to definitions and declarations of input and output streams
                // definition_provider: Some(OneOf::Left(true)),
                // declaration_provider: Some(DeclarationCapability::Options(DeclarationOptions {
                //     work_done_progress_options: Default::default(),
                // })),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "DynSRV Language Server initialized!")
            .await;
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // Handle the `textDocument/didOpen` notification
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.change(uri, &params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        self.change(uri, &params.content_changes[0].text).await;
    }

    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
        log::debug!("File Saved");
    }

    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
        log::debug!("File Closed");
    }

    //Done: Added the trigger character "." to provide suggestions for fields and methods when the user types a dot after an expression, added all the built in functions and variables to the completion list, and added the ability to provide suggestions based on the current scope and context of the code being edited.
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completion = self.get_completion(params);
        Ok(completion.map(CompletionResponse::Array))
    }

    // Used the backend to create the hover items for the token at the position of the hover and return it to the client to be displayed in the editor.
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        //Give token based on the position of the hover and return hover information based on the token type (input, output, aux, expr)
        let hover = self.provide_hover(params);
        Ok(hover)
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    use crate::fixtures;

    use super::*;

    #[apply(async_test)]
    async fn test_initialize() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let result = backend
            .initialize(InitializeParams::default())
            .await
            .unwrap();

        let server_infox = result.server_info.unwrap();

        assert_eq!(
            server_infox.name, "DSRV Language Server",
            "Expected server name to be 'DSRV Language Server'"
        );
        assert_eq!(
            server_infox.version,
            Some("1.0".to_string()),
            "Expected version to be '1.0'"
        );

        // Check the server capabilities to ensure that the the implemented capabilities are correctly advertised and that the unimplemented capabilities are not advertised, this is important to ensure that the client can correctly interact with the server and that the server can provide the expected functionality to the client.
        let server_cap = result.capabilities;
        println!("Server Capabilities: {:#?}", server_cap);
        assert!(
            server_cap.hover_provider.is_some(),
            "Expected hover provider to be supported"
        );
        assert!(
            server_cap.completion_provider.is_some(),
            "Expected completion provider to be supported"
        );
        assert!(
            server_cap.text_document_sync.is_some(),
            "Expected text document sync to be supported"
        ); // This is where the diagnostic is also supported, not through the diagnostic provider as it don't need pull based diagnostics

        assert!(
            server_cap.definition_provider.is_none(),
            "Expected definition provider to not be supported"
        );
        assert!(
            server_cap.signature_help_provider.is_none(),
            "Expected signature help provider to not be supported"
        );
    }

    #[apply(async_test)]
    async fn test_shutdown() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let result = backend.shutdown().await;

        assert!(result.is_ok(), "Expected shutdown to complete successfully");
    }

    #[apply(async_test)]
    async fn test_completion() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Uri::from_file_path(
                        "/home/emili/projects/dsrv-vscode/assets/test/test.dsrv",
                    )
                    .unwrap(),
                },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let result = backend.completion(params).await.unwrap();
        println!("Completion Result: {:#?}", result);

        //Test that the completion result gives a response, but as there is no actual code in the test file, it should not give any completion items, so the result should be None
        assert!(result.is_none())
    }

    #[apply(async_test)]
    async fn test_hover() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier {
                    uri: Uri::from_file_path(
                        "/home/emili/projects/dsrv-vscode/assets/test/test.dsrv",
                    )
                    .unwrap(),
                },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
        };

        let result = backend.hover(params).await.unwrap();

        print!("Hover Result: {:#?}", result);

        // As there is no actual code in the test file, there should be no token at the position of the hover, so the result should be None
        assert!(result.is_none())
    }
}
