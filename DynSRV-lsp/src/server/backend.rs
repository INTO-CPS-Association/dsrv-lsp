use crate::analyzer::Analysis;
use dashmap::DashMap;
use ropey::Rope;
use std::ops::Range;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use trustworthiness_checker::{LOLASpecification, VarName};
use crate::server::lexer::*;
use crate::server::completion_candidates::*;

pub struct Backend {
    pub client: Client,
    pub current_analysis: DashMap<Url, Analysis>,
    analysis_map: DashMap<String, Analysis>,
    document_map: DashMap<String, Rope>,
    token_map: DashMap<String, Vec<(Token, Range<usize>)>>,
    builtins: Vec<BuiltinEntry>,
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
                    trigger_characters: Some(vec![".".to_string()]),
                    all_commit_characters: None,
                    completion_item: None,
                    work_done_progress_options: Default::default(),
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completion = self.get_completion(params);
        Ok(completion.map(CompletionResponse::Array))
        // Ok(Some(CompletionResponse::Array(vec![
        //     CompletionItem::new_simple("test".to_string(), "Some Detail".to_string()),
        //     CompletionItem::new_simple("another".to_string(), "Another Detail".to_string()),
        //     CompletionItem::new_simple("test2".to_string(), "Some Detail2".to_string()),
        // ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
      //Give token based on the position of the hover and return hover information based on the token type (input, output, aux, expr)
        let hover = self.provide_hover(params);
      
      
        Ok(hover)
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
            builtins: load_built_ins(),
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

    fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let pos = params.text_document_position;
        let uri_key = pos.text_document.uri.to_string();

        let analysis_ref = self.analysis_map.get(&uri_key)?;
        let analysis = analysis_ref.value();
        
        let token_ref = self.token_map.get(&uri_key)?;
        let tokens = token_ref.value();
        
        let cursor_char = pos.position.character as usize;
        let token_at_cursor = tokens.iter().find(|(_, range)| {
          range.start <= cursor_char && range.end >= cursor_char
        });
        log::info!("Token at cursor: {:?}", token_at_cursor);
        
        let mut items = Vec::new();
        items.extend(json_to_completionItem(&self.builtins));
        
        
        
        if let Some(spec) = &analysis.spec {
            let item = get_all_declared_symbols(&spec);
            for i in item{
              items.push(i);
            }
          }
          return Some(items);

    }

    
    
    fn provide_hover(&self, params: HoverParams) -> Option<Hover> {
      let pos = params.text_document_position_params;
      let uri_key = pos.text_document.uri.to_string();
      
      let token_ref = self.token_map.get(&uri_key)?;
      let tokens = token_ref.value();
      
      let mut hovers = Vec::new();
// contents: HoverContents::Scalar(MarkedString::String("Hovering Test".to_string())),
// range: None,
      for token in tokens {
        let hover = MarkedString::String(format!("Token: {:?} ", token.0));
        hovers.push(hover);
      };
        Some(Hover{contents: HoverContents::Array(hovers), range: None}) 
    }
  
  
    
    
    // Helper function to create diagnostics from error message and range
    async fn logger(&self, mes: String, level: MessageType) {
        self.client.log_message(level, mes).await;
    }
}

// Helper function to get line from position
fn _pos_to_slice(pos: Position, rope: &Rope) -> Option<String> {
    let line = rope.line(pos.line as usize);
    log::info!("Extracted line at position: `{}`", line);
    Some(line.to_string())
}

// Convert specification items into completion items for autocompletion
fn get_all_declared_symbols(spec: &LOLASpecification) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    for name in &spec.input_vars {
        items.push(create_item(
            name,
            CompletionItemKind::VARIABLE,
            "Input Stream",
        ));
    }
    for name in &spec.output_vars {
        items.push(create_item(
            name,
            CompletionItemKind::VARIABLE,
            "Output Stream",
        ));
    }

    for name in &spec.aux_info {
        items.push(create_item(name, CompletionItemKind::VARIABLE, "Aux/Var"));
    }

    for (name, _) in &spec.exprs {
        if !spec.input_vars.contains(name) && !spec.output_vars.contains(name) {
            items.push(create_item(
                name,
                CompletionItemKind::VARIABLE,
                "Stream Expression",
            ));
        }
    }

    items
}

fn create_item(name: &VarName, kind: CompletionItemKind, detail: &str) -> CompletionItem {
    CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        detail: Some(detail.to_string()),
        insert_text: Some(name.to_string()),
        ..Default::default()
    }
}
