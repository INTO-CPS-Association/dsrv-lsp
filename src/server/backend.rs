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
macro_rules! hover_doc {
    ($value:expr) => {
        HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: $value.to_string(),
        })
    };
}

pub struct Backend {
    pub client: Client,
    // Store the analysis, rope and lexed tokens for each document URI.
    document_map: DashMap<Uri, Rope>,
    analysis_map: DashMap<Uri, Analysis>,
    token_map: DashMap<Uri, Vec<TokenData>>,
}

// Backend implementation for the language server
impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            analysis_map: DashMap::new(),
            token_map: DashMap::new(),
        }
    }
    pub async fn change(&self, uri: Uri, text: &str) {
        match uri.to_file_path() {
            // Try to convert URI to file path, if it fails, log an error message and skip analysis to avoid trying to analyse not file windows such as the output or terminal window
            Some(_path) => {
                self.logger(format!("Analyzing document `{:?}`", uri), MessageType::INFO)
                    .await;

                // If URI is successfully converted to file path, proceed with analysis
                self.document_map.insert(uri.clone(), Rope::from_str(text));
                self.token_map.insert(uri.clone(), tokenize(text));

                let analysis = Analysis::analyze_specification(&text).await;
                let diags = analysis.diags.clone(); // Clone diagnostics to avoid ownership issues when inserting analysis into the map later.

                // Only Update the specification if parsing was successful, otherwise keep the previous specification to avoid losing the AST structure and spanned nodes that are needed for providing completion and hover information based on the current position in the document
                if analysis.spec.is_some() {
                    self.analysis_map.insert(uri.clone(), analysis);
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

    // function to provide completion items based on the current position in the document and the context of the code at that position.
    pub fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let pos = params.text_document_position;
        let uri_key = pos.text_document.uri;

        // For the variables
        let analysis_ref = self.analysis_map.get(&uri_key)?;
        let analysis = analysis_ref.value();

        // for the tokens to make the context
        let binding = self.token_map.get(&uri_key)?;
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

        items.extend(
            BUILTIN_REGISTRY
                .iter()
                .filter(|builtin| context.iter().any(|c| builtin.trigger_context.contains(c)))
                .map(|builtin| create_item(builtin)),
        );

        // Collects and add input, output, aux variables and stream expressions
        if let Some(spec) = &analysis.spec {
            let variables = get_all_declared_symbols(&spec);
            items.extend(
                variables
                    .iter()
                    .filter(|var| context.iter().any(|c| var.trigger_context.contains(c)))
                    .map(|var| CompletionItem {
                        label: var.label.to_string(),
                        kind: Some(var.kind),
                        detail: Some(var.detail.to_string()),
                        ..Default::default()
                    }),
            );
        }
        Some(items)
    }

    // Uses the spanned nodes in the AST to provide hover information for the symbol at the current position in the document. Including variable and built-in functions.
    pub fn provide_hover(&self, params: HoverParams) -> Option<Hover> {
        let pos = params.text_document_position_params;
        let uri_key = pos.text_document.uri;

        let analysis_ref = self.analysis_map.get(&uri_key)?;
        let analysis = analysis_ref.value();

        let rope = self.document_map.get(&uri_key)?;
        let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();

        let node = Analysis::node_at_offset(&analysis, pos_offset)?;
        log::info!("Node at offset {}: {:?}", pos_offset, node);

        // Match the node at the current offset with the corresponding built-in function to provide hover information. If the node is not a built-in function, return None to indicate that no hover information is available for that symbol.
        if let Some(label) = node.builtin_label() {
            let builtin = get_builtin_by_label(label)?;
            return Some(create_hover_item(builtin, &node.span, &rope));
        }

        match node.node {
            SExpr::Var(ref var_name) => {
                let spec = analysis.spec.clone()?;
                let t = spec.type_annotations.get(var_name);

                // log::info!("Providing hover information for variable `{}`", var_name);

                let type_str = match t {
                    Some(ty) => format!(": {:?}", ty),
                    None => String::new(),
                };

                let (stream_kind, stream_text) = if spec.input_vars.contains(var_name) {
                    ("in", get_builtin_by_label("in")?.documentation)
                } else if spec.aux_info.contains(var_name) {
                    ("aux", get_builtin_by_label("aux")?.documentation)
                } else if spec.output_vars.contains(var_name) {
                    ("out", get_builtin_by_label("out")?.documentation)
                } else {
                    ("stream", "stream")
                };

                let info = format!(
                    "```dsrv\n{} {}{}\n```\n---\n{}",
                    stream_kind, var_name, type_str, stream_text
                );
                // log::info!("\n{}\n", info);
                Some(create_hover_variable(&info, &node.span, &rope))
            }

            _ => None,
        }
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
    let content = hover_doc!(format!(
        "```dsrv\n{}\n```\n---\n{}",
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

fn create_hover_variable(s: &str, span: &Span, rope: &Rope) -> Hover {
    let content = hover_doc!(s);
    Hover {
        contents: content,
        range: Some(Range::new(
            byte_to_pos(&rope, span.start as usize).unwrap_or_default(),
            byte_to_pos(&rope, span.end as usize).unwrap_or_default(),
        )),
    }
}

pub trait SExprHoverExt {
    fn builtin_label(&self) -> Option<&'static str>;
}
impl SExprHoverExt for SExpr {
    fn builtin_label(&self) -> Option<&'static str> {
        match self {
            SExpr::RestrictedDynamic(..) | SExpr::Dynamic(..) => Some("dynamic"),
            SExpr::Defer(..) => Some("defer"),
            SExpr::Update(..) => Some("update"),
            SExpr::Default(..) => Some("default"),
            SExpr::IsDefined(..) => Some("is_defined"),
            SExpr::When(..) => Some("when"),
            SExpr::Latch(..) => Some("latch"),
            SExpr::Init(..) => Some("init"),
            SExpr::SIndex(..) => Some("SIndex"),
            SExpr::If(..) => Some("If then else"),
            SExpr::MonitoredAt(..) => Some("Monitored_at"),
            SExpr::Dist(..) => Some("dist"),
            SExpr::List(..) => Some("List."),
            SExpr::LIndex(..) => Some("List.get"),
            SExpr::LAppend(..) => Some("List.append"),
            SExpr::LConcat(..) => Some("List.concat"),
            SExpr::LHead(..) => Some("List.head"),
            SExpr::LTail(..) => Some("List.tail"),
            SExpr::LLen(..) => Some("List.len"),
            SExpr::Map(..) => Some("Map."),
            SExpr::MGet(..) => Some("Map.get"),
            SExpr::MInsert(..) => Some("Map.insert"),
            SExpr::MRemove(..) => Some("Map.remove"),
            SExpr::MHasKey(..) => Some("Map.has_key"),
            SExpr::Sin(..) => Some("sin"),
            SExpr::Cos(..) => Some("cos"),
            SExpr::Tan(..) => Some("tan"),
            SExpr::Abs(..) => Some("abs"),
            SExpr::Not(..) => Some("Not"),
            _ => None,
        }
    }
}
