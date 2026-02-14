use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::analyzer::{Analysis, analyze};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    pub client: Client,
    pub current_analysis: RwLock<HashMap<Url, Analysis>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            current_analysis: RwLock::new(HashMap::new()),
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
    
    async fn did_open(&self, params: DidOpenTextDocumentParams){
      let uri = params.text_document.uri;
      
        if let Ok(_path) = uri.to_file_path(){

        self.client.log_message(MessageType::INFO, format!("Analyzing document `{}`", uri)).await;
        let uri = uri;
        let text = params.text_document.text;
        let analysis = analyze(&text).await;
        
        self.current_analysis.write().unwrap().insert(uri.clone(), analysis.clone());
        
        self.client.publish_diagnostics(uri, analysis.diags, None).await;
        self.client.log_message(MessageType::INFO, "Document opened and analyzed").await;      
        }
  }
}


// struct TextDocumentChange<'a> {
//   uri: String,
//   text: &'a str,
// }

