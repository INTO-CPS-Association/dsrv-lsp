mod lsp_server;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};


pub struct Backend {
  client: Client,
}

impl Backend {
  pub fn new(client: Client) -> Self{
    Self {
      client,
    }
  }
  
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        ..ServerCapabilities::default()
      },
      server_info: Some(ServerInfo {
        name: "DynSRV Language Server".to_string(),
        version: Some("0.1.0".to_string()),
      }),
    })
  }
  
  async fn initialized(&self, _: InitializedParams) {
    self.client.log_message(MessageType::INFO, "DynSRV Language Server initialized!").await;
  }
  
  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }
  
}