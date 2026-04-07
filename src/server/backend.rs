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
use crate::lang::syntax::completions_list::*;
use crate::lang::syntax::lexer::*;
use crate::utils::byte_to_pos;
use crate::utils::pos_to_offset;
use dashmap::DashMap;
use ropey::Rope;
// use tower_lsp::Client;
// use tower_lsp::lsp_types::*;
use tower_lsp_server::{Client, ls_types::*};
use trustworthiness_checker::DsrvSpecification;
use trustworthiness_checker::lang::dsrv::{ast::SExpr, span::Span};

macro_rules! documentation {
    ($value:expr) => {
        Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: $value.to_string(),
        }))
    };
}
macro_rules! hoverDoc {
    ($value:expr) => {
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: $value.to_string(),
        })
    };
}

pub struct Backend {
    pub client: Client,
    pub current_analysis: DashMap<Uri, Analysis>,
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
    pub async fn change(&self, uri: Uri, text: &String) {
        let rope = Rope::from_str(text);
        let mut diags = Vec::new();
        self.document_map.insert(uri.to_string(), rope);
        let tokens = tokenize(text);
        self.token_map.insert(uri.to_string(), tokens);

        match uri.to_file_path() {
            // Try to convert URI to file path, if it fails, log an error message and skip analysis
            Some(_path) => {
                // If URI is successfully converted to file path, proceed with analysis
                self.logger(format!("Analyzing document `{:?}`", uri), MessageType::INFO)
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
            None => {
                // If URI conversion fails, log an error message and skip analysis
                self.logger(
                    format!("Failed to convert URI `{:?}` to file path", uri),
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

        // For the variables
        let analysis_ref = self.analysis_map.get(&uri_key)?;
        let analysis = analysis_ref.value();

        // for the tokens to make the context
        let binding = self.token_map.get(&uri_key).unwrap();
        let tokens = binding.value();

        // For the rope to get the position offset for the context
        let rope = self.document_map.get(&uri_key)?;
        let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();

        // For the context
        let context = filter_suggestions(pos_offset as usize, tokens);
        log::info!(
            "Context for completion at offset {}: {:?}",
            pos_offset,
            context
        );
        let mut items = Vec::new();

        // For the built in completion candidates to be available.
        let builtin_items: Vec<CompletionItem> = BUILTIN_REGISTRY
            .iter()
            .filter(|builtin| context.iter().any(|c| builtin.trigger_context.contains(c)))
            .map(|builtin| create_item(builtin))
            .collect();
        items.extend(builtin_items);

        // Collects and add input, output, aux variables and stream expressions
        if let Some(spec) = &analysis.spec {
            let variables = get_all_declared_symbols(&spec);
            let vars: Vec<CompletionItem> = variables
                .iter()
                .filter(|var| context.iter().any(|c| var.trigger_context.contains(c)))
                .map(|var| CompletionItem {
                    label: var.label.to_string(),
                    kind: Some(var.kind),
                    detail: Some(var.detail.to_string()),
                    ..Default::default()
                })
                .collect();
            items.extend(vars);
        }

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
        
        if node_at_offset.is_none() {
            log::info!("No node found at offset {}, cannot provide hover information", pos_offset);
            return None;
        }
        
        // Match the node at the current offset with the corresponding built-in function to provide hover information. If the node is not a built-in function, return None to indicate that no hover information is available for that symbol.
        let builtin: &DsrvBuiltIn;
        match node_at_offset.unwrap().node {
            SExpr::RestrictedDynamic(..) | SExpr::Dynamic(..) => {
                builtin = get_builtin_by_label("dynamic")?;
            }
            SExpr::Defer(..) => {
                builtin = get_builtin_by_label("defer")?;
            }
            SExpr::Update(..) => {
                builtin = get_builtin_by_label("update")?;
            }
            SExpr::Default(..) => {
                builtin = get_builtin_by_label("default")?;
            }
            SExpr::IsDefined(..) => {
                builtin = get_builtin_by_label("is_defined")?;
            }
            SExpr::When(..) => {
                builtin = get_builtin_by_label("when")?;
            }
            SExpr::Latch(..) => {
                builtin = get_builtin_by_label("latch")?;
            }
            SExpr::Init(..) => {
                builtin = get_builtin_by_label("init")?;
            }
            SExpr::SIndex(..) => {
                builtin = get_builtin_by_label("SIndex")?;
            }
            SExpr::If(..) => {
                builtin = get_builtin_by_label("If then else")?;
            }
            SExpr::MonitoredAt(..) => {
                builtin = get_builtin_by_label("Monitored_at")?;
            }
            SExpr::Dist(..) => {
                builtin = get_builtin_by_label("dist")?;
            }
            SExpr::List(..) => {
                builtin = get_builtin_by_label("List.")?;
            }
            SExpr::LIndex(..) => {
                builtin = get_builtin_by_label("get")?;
            }
            SExpr::LAppend(..) => {
                builtin = get_builtin_by_label("append")?;
            }
            SExpr::LConcat(..) => {
                builtin = get_builtin_by_label("concat")?;
            }
            SExpr::LHead(..) => {
                builtin = get_builtin_by_label("head")?;
            }
            SExpr::LTail(..) => {
                builtin = get_builtin_by_label("tail")?;
            }
            SExpr::LLen(..) => {
                builtin = get_builtin_by_label("len")?;
            }
            SExpr::Map(..) => {
                builtin = get_builtin_by_label("Map.")?;
            }
            SExpr::MGet(..) => {
                builtin = get_builtin_by_label("get")?;
            }
            SExpr::MInsert(..) => {
                builtin = get_builtin_by_label("insert")?;
            }
            SExpr::MRemove(..) => {
                builtin = get_builtin_by_label("remove")?;
            }
            SExpr::MHasKey(..) => {
                builtin = get_builtin_by_label("has_key")?;
            }
            SExpr::Sin(..) => {
                builtin = get_builtin_by_label("sin")?;
            }
            SExpr::Cos(..) => {
                builtin = get_builtin_by_label("cos")?;
            }
            SExpr::Tan(..) => {
                builtin = get_builtin_by_label("tan")?;
            }
            SExpr::Abs(..) => {
                builtin = get_builtin_by_label("abs")?;
            }
            SExpr::Not(..) => {
                builtin = get_builtin_by_label("Not")?;
            }

            SExpr::Var(ref v_name) => {
                let spec = analysis.spec.clone()?; 
                let t = analysis.typed.clone()?.type_annotations.get(v_name);
                
                //check if the var is an input, output or aux variable and provide hover information accordingly
                
                if spec.aux_info.contains(v_name) {
                  log::info!("Providing hover information for aux variable `{}`", v_name);
                } else if spec.input_vars.contains(v_name) {
                  log::info!("Providing hover information for input variable `{}`", v_name);

                } else if spec.output_vars.contains(v_name) {
                  log::info!("Providing hover information for output variable `{}`", v_name);

                } else {
                  log::info!("Variable `{}` is not an input, output or aux variable, no hover information available", v_name);
                }
                
                
                
                return None;
            }

            _ => return None,
        }
        

        
        return Some(create_hover_item(
            builtin,
            &node_at_offset.unwrap().span,
            &rope,
        ));
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

#[derive(Debug, Clone)]
pub struct Variables {
    pub label: String,
    pub kind: CompletionItemKind,
    pub trigger_context: &'static [&'static str],
    pub type_anno: Option<String>,
    pub detail: String,
}

// Convert specification items into completion items for autocompletion
fn get_all_declared_symbols(spec: &DsrvSpecification) -> Vec<Variables> {
    let mut items = Vec::new();

    for name in &spec.input_vars {
        let item = Variables {
            label: name.into(),
            kind: CompletionItemKind::VARIABLE,
            trigger_context: &["expr", "input_stream", "variable"],
            type_anno: None,
            detail: "Input Stream".to_string(),
        };
        items.push(item);
    }
    for name in &spec.output_vars {
        let item = Variables {
            label: name.into(),
            kind: CompletionItemKind::VARIABLE,
            trigger_context: &["expr", "output_stream", "variable"],
            type_anno: None,
            detail: "Output Stream".to_string(),
        };
        items.push(item);
    }
    for name in &spec.aux_info {
        let item = Variables {
            label: name.into(),
            kind: CompletionItemKind::VARIABLE,
            trigger_context: &["expr", "aux_stream", "variable"],
            type_anno: None,
            detail: "Auxiliary internal stream variable".to_string(),
        };
        items.push(item);
    }
    items
}

fn create_item(item: &DsrvBuiltIn) -> CompletionItem {
    CompletionItem {
        label: item.label.to_string(),
        kind: Some(item.kind),
        detail: Some(item.detail.to_string()),
        insert_text: Some(item.insert_text.to_string()),
        insert_text_format: Some(item.insert_text_format),
        documentation: documentation!(item.documentation),
        ..Default::default()
    }
}

fn create_hover_item(item: &DsrvBuiltIn, span: &Span, rope: &Rope) -> Hover {
    let content = hoverDoc!(format!(
        "```l\n{}\n```\n{}",
        item.detail,
        item.documentation.trim()
    ));

    Hover {
        contents: content,
        range: Some(Range::new(
            byte_to_pos(&rope, span.start as usize).unwrap_or_default(),
            byte_to_pos(&rope, span.end as usize).unwrap_or_default(),
        )),
    }
}
