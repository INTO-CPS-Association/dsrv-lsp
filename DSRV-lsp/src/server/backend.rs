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

use crate::lang::analyzer::*;
use crate::lang::syntax::completion_candidates::*;
use crate::lang::syntax::lexer::*;
use crate::utils::pos_to_offset;
use dashmap::DashMap;
use ropey::Rope;
use tower_lsp::Client;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::{DsrvSpecification, VarName};

pub struct Backend {
    pub client: Client,
    pub current_analysis: DashMap<Url, Analysis>,
    analysis_map: DashMap<String, Analysis>,
    document_map: DashMap<String, Rope>,
    token_map: DashMap<String, Vec<TokenData>>,
}

// Backend implementation for the language server
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
    pub async fn change(&self, uri: Url, text: &String) {
        let rope = Rope::from_str(text);
        let mut diags = Vec::new();
        self.document_map.insert(uri.to_string(), rope);
        let tokens = tokenize(text);
        self.token_map.insert(uri.to_string(), tokens);

        match uri.to_file_path() {
            // Try to convert URI to file path, if it fails, log an error message and skip analysis
            Ok(_path) => {
                // If URI is successfully converted to file path, proceed with analysis
                self.logger(format!("Analyzing document `{}`", uri), MessageType::INFO)
                    .await;

                let analysis = Analysis::analyze_2_point_0(&text).await;
                for diag in analysis.clone().diags {
                    diags.push(diag);
                }
                self.current_analysis.insert(uri.clone(), analysis.clone());

                // Only Update the specification if parsing was successful, otherwise keep the previous specification to avoid losing the AST structure and spanned nodes that are needed for providing completion and hover information based on the current position in the document
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
    }

    // TODO: Implement the completion handler to provide autocompletion suggestions based on the current position in the document after the AST structure is updated with spanned nodes
    pub fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let pos = params.text_document_position;
        let uri_key = pos.text_document.uri.to_string();

        let analysis_ref = self.analysis_map.get(&uri_key)?;
        let analysis = analysis_ref.value();
        let binding = self.token_map.get(&uri_key).unwrap();
        let tokens = binding.value();
                
        let rope = self.document_map.get(&uri_key)?;
        let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();
        
        let context = filter_suggestions(pos_offset as usize, tokens);
        
        
        let mut items = Vec::new();

        // Collects and add input, output, aux variables and stream expressions
        if let Some(spec) = &analysis.spec {
            let item = get_all_declared_symbols(&spec);
            items.extend(item);
        }

        
        
        
        // For the built in completion candidates to be available.
        let builtin_items: Vec<CompletionItem> = BUILTIN_REGISTRY
            .iter().filter(|builtin| builtin.trigger_context.contains(&context[0])
            )
            .map(|builtin| CompletionItem {
                label: builtin.label.to_string(),
                kind: Some(builtin.kind),
                detail: Some(builtin.detail.to_string()),
                insert_text: Some(builtin.insert_text.to_string()),
                insert_text_format: Some(builtin.insert_text_format),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: builtin.documentation.to_string(),
                })),
                ..Default::default()
            })
            .collect();

        items.extend(builtin_items);

        return Some(items);
    }

    // TODO: Implement the hover handler to provide information about the symbol under the cursor based on the current position in the document after the AST structure is updated with spanned nodes
    pub fn provide_hover(&self, params: HoverParams) -> Option<Hover> {
      let pos = params.text_document_position_params;
      let uri_key = pos.text_document.uri.to_string();
      
      let analysis_ref = self.analysis_map.get(&uri_key)?;
      let analysis = analysis_ref.value();
      
      let rope = self.document_map.get(&uri_key)?;
      let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();
      
      let node_at_offset = Analysis::node_at_offset(&analysis, pos_offset);
      log::info!("Node at offset {}: {:?}", pos_offset, node_at_offset);
      
      
      
        // let pos = params.text_document_position_params;
        // let uri_key = pos.text_document.uri.to_string();

        // let token_ref = self.token_map.get(&uri_key)?;
        // let tokens = token_ref.value();

        // let mut hovers = Vec::new();
        // // contents: HoverContents::Scalar(MarkedString::String("Hovering Test".to_string())),
        // // range: None,
        // for token in tokens {
        //     let hover = MarkedString::String(format!("Token: {:?} ", token.0));
        //     hovers.push(hover);
        // }
        // Some(Hover {
        //     contents: HoverContents::Array(hovers),
        //     range: None,
        // })
        None
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
fn get_all_declared_symbols(spec: &DsrvSpecification) -> Vec<CompletionItem> {
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
        items.push(create_item(
            name,
            CompletionItemKind::VARIABLE,
            "Stream Variables",
        ));
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
