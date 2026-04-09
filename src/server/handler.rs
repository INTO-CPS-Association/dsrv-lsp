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

use tower_lsp_server::{LanguageServer, ls_types::*, jsonrpc::Result};
use crate::server::backend::Backend;


impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "DynSRV Language Server".to_string(),
                version: Some("0.1.0".to_string()),
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

                definition_provider: Some(OneOf::Left(true)),
                declaration_provider: Some(DeclarationCapability::Options(DeclarationOptions {
                    work_done_progress_options: Default::default(),
                })),
                execute_command_provider: Some(ExecuteCommandOptions {
                    ..Default::default()
                }),

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

    // TODO: Implement the hover handler to provide information about the token under the cursor, such as its type and documentation, based on the AST structure with spanned nodes½
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
      //Give token based on the position of the hover and return hover information based on the token type (input, output, aux, expr)
        let hover = self.provide_hover(params);
      
      
        Ok(hover)
    }
}