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

#[derive(Debug, Clone)]
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

    // function to provide completion items based on the current position in the document and the context of the code at that position.
    pub fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let pos = params.text_document_position;
        let uri_key = pos.text_document.uri;

        // To map the offset into byte instead of line and character
        let rope = self.document_map.get(&uri_key)?;
        let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();

        // for the tokens to make the context
        let binding = self.token_map.get(&uri_key)?;
        let context = filter_suggestions(pos_offset as usize, binding.value());
        // log::info!(
        //     "Context for completion at offset {}: {:?}",
        //     pos_offset,
        //     context
        // );

        // Vector to collect the completion item fitting in the context
        let mut items = Vec::new();

        items.extend(
            BUILTIN_REGISTRY
                .iter()
                .filter(|builtin| context.iter().any(|c| builtin.trigger_context.contains(c)))
                .map(|builtin| create_item(builtin)),
        );

        // For the variables
        let analysis_ref = self.analysis_map.get(&uri_key)?;

        // Collects and add input, output, aux variables and stream expressions
        if let Some(spec) = &analysis_ref.value().spec {
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
        //TODO:This will give wrong hover info if the user is hovering over an area that has changed but was never syntactically correct, So the old AST is still present and provides hover information that does not match the current code. Could be solved by comparing with the lexed token map and use that as backup if the AST node does not match the token to still return something
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

#[derive(Debug, Clone, PartialEq)]
pub struct Variables {
    pub label: String,
    pub kind: CompletionItemKind,
    pub trigger_context: &'static [&'static str],
    pub type_anno: Option<String>,
    pub detail: String,
}

// TODO: Add support for typed variables to be able to provide type information in the completion items.
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
    for name in &spec.output_vars {
        // Check if the variable is already in, as aux vars is both parsed as output and aux variables, so they will be in both lists, but we only want to add them once with the aux variable information as that is more specific.
        if !spec.aux_info.contains(name) {
            let item = Variables {
                label: name.into(),
                kind: CompletionItemKind::VARIABLE,
                trigger_context: &["expr", "output_stream", "variable"],
                type_anno: None,
                detail: "Output Stream".to_string(),
            };
            items.push(item);
        }
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

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    use crate::fixtures;

    use super::*;

    #[apply(async_test)]
    async fn test_get_all_declared_symbols() {
        let input = fixtures::input_untyped_valid_simple();
        let analysis = fixtures::analyze_spec(input).await;
        let spec = analysis.spec.expect("Expected a valid specification");

        let vars = get_all_declared_symbols(&spec);

        println!("Declared symbols: {:#?}", vars);

        assert!(
            vars.len() == 3,
            "Expected 3 declared symbols, found {}",
            vars.len()
        );

        let result = vec![
            Variables {
                label: "x".to_string(),
                kind: CompletionItemKind::VARIABLE,
                trigger_context: &["expr", "input_stream", "variable"],
                type_anno: None,
                detail: "Input Stream".to_string(),
            },
            Variables {
                label: "y".to_string(),
                kind: CompletionItemKind::VARIABLE,
                trigger_context: &["expr", "input_stream", "variable"],
                type_anno: None,
                detail: "Input Stream".to_string(),
            },
            Variables {
                label: "z".to_string(),
                kind: CompletionItemKind::VARIABLE,
                trigger_context: &["expr", "output_stream", "variable"],
                type_anno: None,
                detail: "Output Stream".to_string(),
            },
        ];

        assert_eq!(
            vars, result,
            "Declared symbols do not match expected result"
        );
    }

    #[apply(async_test)]
    async fn test_get_all_declared_symbols_complex() {
        let input = fixtures::input_untyped_complex_with_comments();
        let analysis = fixtures::analyze_spec(input).await;
        let spec = analysis.spec.expect("Expected a valid specification");

        let vars = get_all_declared_symbols(&spec);

        println!("Declared symbols in complex spec: {:#?}", vars);

        assert!(
            vars.len() == 11,
            "Expected 11 declared symbols, found {}",
            vars.len()
        )
    }
    // Realised I never made it able to handle typed variables in the backend.
    // #[apply(async_test)]
    // async fn test_get_all_declared_symbols_typed() {
    //     let input = fixtures::input_typed_valid_simple();
    //     let analysis = fixtures::analyze_spec(input).await;
    //     let spec = analysis.typed.expect("Expected a valid specification");
    //     let vars = get_all_declared_symbols(&spec);
    //     println!("Declared symbols: {:#?}", vars);
    // }

    #[test]
    fn test_create_item() {
        let dsrv = DsrvBuiltIn {
            label: "in",
            kind: CompletionItemKind::KEYWORD,
            trigger_context: &["toplevel"],
            insert_text: "in $1",
            insert_text_format: InsertTextFormat::SNIPPET,
            detail: "in <label> [: <Type>]",
            documentation: "Declares an input stream that provides a sequence of event values to the monitor. The label acts as a variable name in the input namespace in(ϕ)",
        };

        let item = create_item(&dsrv);

        println!("Created completion item: {:#?}", item);

        assert!(
            item.label == "in",
            "Expected label to be `out`, found `{}`",
            item.label
        );

        assert!(
            item.kind == Some(CompletionItemKind::KEYWORD),
            "Expected kind to be `FUNCTION`, found `{:?}`",
            item.kind
        );
    }

    #[test]
    fn test_create_hover_item() {
        let dsrv = DsrvBuiltIn {
            label: "in",
            kind: CompletionItemKind::KEYWORD,
            trigger_context: &["toplevel"],
            insert_text: "in $1",
            insert_text_format: InsertTextFormat::SNIPPET,
            detail: "in <label> [: <Type>]",
            documentation: "Declares an input stream that provides a sequence of event values to the monitor. The label acts as a variable name in the input namespace in(ϕ)",
        };

        let rope = Rope::from_str(fixtures::input_untyped_valid_simple());

        let item = create_hover_item(&dsrv, &Span { start: 1, end: 2 }, &rope);

        println!("Created hover item: {:#?}", item);

        assert!(
            item.contents
                == hover_doc!(format!(
                    "```dsrv\n{}\n```\n---\n{}",
                    dsrv.detail,
                    dsrv.documentation.trim()
                )),
            "Hover contents do not match expected value"
        );

        assert!(
            item.range.unwrap().start
                == Position {
                    line: 0,
                    character: 1
                },
            "Hover range start does not match expected value"
        );
        assert!(
            item.range.unwrap().end
                == Position {
                    line: 0,
                    character: 2
                },
            "Hover range end does not match expected value"
        );
    }

    #[test]
    fn test_create_variable() {
        let rope = Rope::from_str(fixtures::input_untyped_valid_simple());
        let var = format!(
            "```dsrv\n{} {}{}\n```\n---\n{}",
            "in",
            "x",
            "",
            get_builtin_by_label("in").unwrap().documentation
        );

        let item = create_hover_variable(&var, &Span { start: 1, end: 4 }, &rope);

        println!("Created variable hover item: {:#?}", item);

        assert!(
            item.contents == hover_doc!(var),
            "Variable hover contents do not match expected value"
        );

        assert!(
            item.range.unwrap().start
                == Position {
                    line: 0,
                    character: 1
                },
            "Variable hover range start does not match expected value"
        );

        assert!(
            item.range.unwrap().end
                == Position {
                    line: 0,
                    character: 4
                },
            "Variable hover range end does not match expected value"
        );
    }

    #[apply(async_test)]
    async fn test_backend_change_untyped() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_untyped_valid_simple();

        backend.change(uri.clone(), text).await;

        println!("Backend: {:?}", backend);

        assert!(
            backend.document_map.contains_key(&uri),
            "Document map does not contain the URI after change"
        );

        assert!(
            backend.analysis_map.contains_key(&uri),
            "Analysis map does not contain the URI after change"
        );

        assert!(
            backend.token_map.contains_key(&uri),
            "Token map does not contain the URI after change"
        );
    }

    #[apply(async_test)]
    async fn test_change_complex() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_untyped_complex_with_comments();

        backend.change(uri.clone(), text).await;

        println!("Backend after complex change: {:?}", backend);

        assert!(
            backend.document_map.contains_key(&uri),
            "Document map does not contain the URI after complex change"
        );

        assert!(
            backend.analysis_map.contains_key(&uri),
            "Analysis map does not contain the URI after complex change"
        );

        assert!(
            backend.token_map.contains_key(&uri),
            "Token map does not contain the URI after complex change"
        );
    }

    #[apply(async_test)]
    async fn test_change_typed() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_typed_valid_complex();

        backend.change(uri.clone(), text).await;

        println!("Backend after typed change: {:?}", backend);

        assert!(
            backend.document_map.contains_key(&uri),
            "Document map does not contain the URI after typed change"
        );
        assert!(
            backend.analysis_map.contains_key(&uri),
            "Analysis map does not contain the URI after typed change"
        );
        assert!(
            backend.token_map.contains_key(&uri),
            "Token map does not contain the URI after typed change"
        );
    }

    #[apply(async_test)]
    async fn test_backend_new() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        println!("Backend: {:?}", backend);

        assert!(
            backend.document_map.is_empty(),
            "Document map should be empty on new backend"
        );
        assert!(
            backend.analysis_map.is_empty(),
            "Analysis map should be empty on new backend"
        );
        assert!(
            backend.token_map.is_empty(),
            "Token map should be empty on new backend"
        );
    }

    #[apply(async_test)]
    async fn test_get_completion() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();
        let uri = fixtures::create_URI_path();

        // Test completion with  valid text
        let text = fixtures::input_untyped_valid_simple();

        backend.change(uri.clone(), text).await;
        // println!("Backend: {:?}", backend.token_map);

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 4,
                    character: 10,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let completion = backend.get_completion(params).unwrap();
        assert!(!completion.is_empty(), "Expected completions");
    }

    #[apply(async_test)]
    async fn test_provide_hover() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_untyped_valid_simple(); // "in x\nin y\nout z\nz = x + y"

        backend.change(uri.clone(), text).await;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 3,
                }, // 'x'
            },
            work_done_progress_params: Default::default(),
        };

        let hover = backend.provide_hover(params);

        println!("Hover result: {:#?}", hover);

        assert!(
            hover.is_some(),
            "Expected hover information for variable 'x'"
        );
    }

    #[apply(async_test)]
    async fn test_hover_typed() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_typed_valid_simple();

        backend.change(uri.clone(), text).await;
        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 3,
                    character: 6,
                }, // 'x'
            },
            work_done_progress_params: Default::default(),
        };

        let hover = backend.provide_hover(params);
        println!("Hover result for typed variable: {:#?}", hover);

        assert!(
            hover.is_some(),
            "Expected hover information for typed variable 'x'"
        );

        assert!(
            hover.unwrap().contents
                == hover_doc!(format!(
                    "```dsrv\nin x: Int\n```\n---\nDeclares an input stream that provides a sequence of event values to the monitor. The label acts as a variable name in the input namespace in(ϕ)"
                )),
            "Hover contents do not match expected value for typed variable 'x'"
        );
    }

    #[apply(async_test)]
    async fn test_hover_empty() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        let uri = fixtures::create_URI_path();
        let text = fixtures::input_empty();

        backend.change(uri.clone(), text).await;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
        };

        let hover = backend.provide_hover(params);

        println!("Hover result for empty document: {:#?}", hover);

        assert!(
            hover.is_none(),
            "Expected no hover information for empty document"
        );
    }

    #[apply(async_test)]
    async fn test_get_completion_empty() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();
        let uri = fixtures::create_URI_path();

        // Test completion with empty text
        let text = fixtures::input_empty();

        backend.change(uri.clone(), text).await;

        let params = CompletionParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position {
                    line: 0,
                    character: 0,
                },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: None,
        };

        let completion = backend.get_completion(params).unwrap();

        println!("Completion result for empty document: {:#?}", completion);

        assert!(
            !completion.is_empty(),
            "Expected completions for empty document"
        );

        assert!(
            completion.len() == 3,
            "Expected 3 completions (in, out, aux) for empty document, found {}",
            completion.len()
        );
        assert!(
            completion[0].label == "in".to_string(),
            "Expected first completion to be `in`, found `{}`",
            completion[0].label
        );
    }

    #[apply(async_test)]
    async fn test_logger() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();

        // Test logging an info message
        backend
            .logger("This is an info message".to_string(), MessageType::INFO)
            .await;

        // Test logging a warning message
        backend
            .logger(
                "This is a warning message".to_string(),
                MessageType::WARNING,
            )
            .await;

        // Test logging an error message
        backend
            .logger("This is an error message".to_string(), MessageType::ERROR)
            .await;

        // Can't really assert anything here, but at least we can check that the function runs without panicking and logs the messages to the client.
        assert!(true, "Logger function executed without panicking")
    }

    #[apply(async_test)]
    async fn test_get_builtin() {
        let builtin = get_builtin_by_label("in");
        assert!(
            builtin.is_some(),
            "Expected to find built-in function with label `in`"
        );
        let builtin = builtin.unwrap();
        assert!(
            builtin.label == "in",
            "Expected built-in label to be `in`, found `{}`",
            builtin.label
        );
        assert!(
            builtin.documentation.contains("Declares an input stream"),
            "Expected documentation to contain 'Declares an input stream', found `{}`",
            builtin.documentation
        );
    }
}
