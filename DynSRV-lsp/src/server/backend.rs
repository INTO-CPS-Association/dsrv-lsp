use crate::analyzer::Analysis;
use dashmap::DashMap;
use ropey::Rope;
use std::ops::Range;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    pub client: Client,
    pub current_analysis: RwLock<HashMap<Url, Analysis>>,
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

    // Handle the `textDocument/didOpen` notification
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.change(uri, &params.text_document.text).await;
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams){
        let uri = params.text_document.uri;
        self.change(uri, &params.content_changes[0].text).await;
    }
    
    async fn did_save(&self, _params: DidSaveTextDocumentParams) {
      log::debug!("File Saved");
    }
    
    async fn did_close(&self, _params: DidCloseTextDocumentParams) {
    log::debug!("File Closed");
    } 
} 

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            current_analysis: RwLock::new(HashMap::new()),
        }
    }
    async fn change(&self, uri: Url, text: &String) {
        match uri.to_file_path() {
            Ok(_path) => {
              self.client.log_message(MessageType::INFO, format!("Analyzing document `{}`", uri)).await;
              
              let analysis = Analysis::analyze_2_point_0(&text).await;
              if !analysis.diags.is_empty() {
                // self.client.log_message(MessageType::INFO, format!("Diagnostics for line: {:?}", analysis.diags)).await;
              
              self.current_analysis.write().unwrap().insert(uri.clone(), analysis.clone());
            }
            self.client.publish_diagnostics(uri.clone(), analysis.diags, None).await;
              
            }
            Err(_path) => {
                self.client
                    .log_message(
                        MessageType::INFO,
                        format!("Failed to convert URI `{}` to file path", uri),
                    )
                    .await;
            }
        }

        //     //Log diagnostics in output console
        //     self.client
        //         .log_message(MessageType::INFO, "Document opened and analyzed")
        //         .await;
        // }
    }
}
