use crate::analyzer::Analysis;
use dashmap::DashMap;
use ropey::Rope;
use std::ops::Range;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use trustworthiness_checker::{LOLASpecification, VarName};
use crate::server::lexer::*;

pub struct Backend {
    pub client: Client,
    pub current_analysis: DashMap<Url, Analysis>,
    analysis_map: DashMap<String, Analysis>,
    document_map: DashMap<String, Rope>,
    token_map: DashMap<String, (Vec<Token>, Vec<Range<usize>>)>,
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
} 

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            current_analysis: DashMap::new(),
            document_map: DashMap::new(),
            analysis_map: DashMap::new(),
            token_map: DashMap::new(),
        }
    }
    async fn change(&self, uri: Url, text: &String) {
        let rope = Rope::from_str(text);
        let mut diags = Vec::new();
        self.document_map.insert(uri.to_string(), rope);
        self.token_map.insert(uri.to_string(), tokenize(text, &mut diags));

        match uri.to_file_path() {
            // Try to convert URI to file path, if it fails, log an error message and skip analysis
            Ok(_path) => {
                // If URI is successfully converted to file path, proceed with analysis
                self.logger(format!("Analyzing document `{}`", uri), MessageType::INFO)
                    .await;

                let analysis = Analysis::analyze_2_point_0(&text).await;
                for diag in analysis.clone().diags{
                  diags.push(diag);
                }
                self.current_analysis.insert(uri.clone(), analysis.clone());

                // Only Update the symbol map if AST is valid
                if analysis.spec.is_some() {
                    self.analysis_map.insert(uri.to_string(), analysis.clone());
                }

                self.client
                    .publish_diagnostics(uri.clone(), diags, None)
                    .await;
            }
            Err(_path) => {
                // If URI conversion fails, log an error message and skip analysis
                self.logger(
                    format!("Failed to convert URI `{}` to file path", uri),
                    MessageType::ERROR,
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
    // Helper function to create diagnostics from error message and range
    async fn logger(&self, mes: String, level: MessageType) {
        self.client.log_message(level, mes).await;
    }
}
}
