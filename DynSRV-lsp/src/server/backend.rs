use std::sync::{Arc, RwLock};

use crate::analyzer::Analysis;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    pub client: Client,
    pub current_analysis: Arc<RwLock<Option<Analysis>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            current_analysis: Arc::new(RwLock::new(None)),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "DynSRV Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),

            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
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
}

