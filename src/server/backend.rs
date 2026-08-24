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

use std::sync::Arc;

use crate::lang::analyser::Analysis;
use crate::lang::syntax::completions_list::*;
use crate::lang::syntax::lexer::*;
use crate::utils::byte_to_pos;
use crate::utils::pos_to_offset;
use dashmap::DashMap;
use ropey::Rope;
use tower_lsp_server::{Client, ls_types::*};
use trustworthiness_checker::{
    VarName,
    lang::dsrv::{
        ast::{DsrvSpecification, ExprView, SyntaxLiteral},
        span::Span,
    },
};

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
    analysis_map: DashMap<Uri, Arc<Analysis>>,
    token_map: DashMap<Uri, Vec<TokenData>>,
    revision_map: DashMap<Uri, u64>,
}

// Backend implementation for the language server
impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            analysis_map: DashMap::new(),
            token_map: DashMap::new(),
            revision_map: DashMap::new(),
        }
    }
    pub async fn change(&self, uri: Uri, text: &str) {
        self.logger(format!("Analyzing document `{:?}`", uri), MessageType::INFO)
            .await;

        // Publish the current document and token state before doing the
        // potentially expensive parse/type-check. Invalidate the previous AST
        // immediately so requests cannot combine it with the new rope.
        let revision = {
            let mut entry = self.revision_map.entry(uri.clone()).or_insert(0);
            *entry = entry.saturating_add(1);
            *entry
        };
        self.document_map.insert(uri.clone(), Rope::from_str(text));
        self.token_map.insert(uri.clone(), tokenize(text));
        self.analysis_map.remove(&uri);

        // Run analysis on a dedicated blocking thread so the tokio runtime stays
        // responsive to hover, completion and other LSP requests while the
        // (potentially slow) parser and type-checker run. `analyze_sync` is a
        // plain synchronous function so it can be called directly from the
        // blocking thread pool without needing `block_on`.
        #[cfg(not(test))]
        let analysis = {
            let text_owned = text.to_string();
            tokio::task::spawn_blocking(move || Analysis::analyze_sync(&text_owned))
                .await
                .unwrap_or_else(|join_err| {
                    eprintln!("[dsrv-lsp] analysis task panicked: {:?}", join_err);
                    Analysis {
                        spec: None,
                        typed: None,
                        diags: vec![],
                        symbol_spans: vec![],
                    }
                })
        };

        #[cfg(test)]
        let analysis = Analysis::analyze_sync(text);
        // A newer didChange may have completed while this analysis was
        // running. Never overwrite its AST or diagnostics with an older result.
        if self
            .revision_map
            .get(&uri)
            .is_none_or(|current| *current != revision)
        {
            return;
        }

        let diags = analysis.diags.clone();

        if analysis.spec.is_some() {
            self.analysis_map.insert(uri.clone(), Arc::new(analysis));
        } else {
            self.analysis_map.remove(&uri);
        }

        self.client
            .publish_diagnostics(uri.clone(), diags, None)
            .await;
    }

    // function to provide completion items based on the current position in the document and the context of the code at that position.
    pub fn get_completion(&self, params: CompletionParams) -> Option<Vec<CompletionItem>> {
        let pos = params.text_document_position;
        let uri_key = pos.text_document.uri;

        let pos_offset = {
            let rope = self.document_map.get(&uri_key)?;
            pos_to_offset(pos.position, &rope).unwrap_or_default()
        };
        let context = {
            let tokens = self.token_map.get(&uri_key)?;
            filter_suggestions(pos_offset as usize, tokens.value())
        };

        let mut items = BUILTIN_REGISTRY
            .iter()
            .filter(|builtin| context.iter().any(|c| builtin.trigger_context.contains(c)))
            .map(create_item)
            .collect::<Vec<_>>();

        let analysis = Arc::clone(self.analysis_map.get(&uri_key)?.value());
        let spec = analysis.spec.as_ref()?;
        let variables = get_all_declared_symbols(spec);
        items.extend(
            variables
                .iter()
                .filter(|var| context.iter().any(|c| var.trigger_context.contains(c)))
                .map(|var| CompletionItem {
                    label: var.label.clone(),
                    kind: Some(var.kind),
                    detail: Some(var.detail.clone()),
                    ..Default::default()
                }),
        );
        Some(items)
    }

    /// Provide hover information from the current owner and transient cursors.
    pub fn provide_hover(&self, params: HoverParams) -> Option<Hover> {
        let pos = params.text_document_position_params;
        let uri_key = pos.text_document.uri;
        let analysis = Arc::clone(self.analysis_map.get(&uri_key)?.value());
        let rope = self.document_map.get(&uri_key)?.clone();
        let pos_offset = pos_to_offset(pos.position, &rope).unwrap_or_default();
        let spec = analysis.spec.as_ref()?;

        // Declaration and assignment-LHS spans are parser-local. They are
        // checked before expression nodes so those editor behaviors remain
        // available without inventing another expression representation.
        if let Some(symbol) = analysis.symbol_at_offset(pos_offset) {
            let variable = VarName::new(&symbol.name);
            return create_variable_hover(spec, &variable, symbol.span, &rope);
        }

        let node = analysis.node_at_offset(pos_offset)?;
        let span = node.span();
        let label = match node.view() {
            ExprView::Var(variable) => {
                return create_variable_hover(spec, variable, span, &rope);
            }
            ExprView::Val(SyntaxLiteral::Bool(value)) => {
                if *value {
                    "true"
                } else {
                    "false"
                }
            }
            ExprView::Dynamic(..) => "dynamic",
            ExprView::Defer(..) => "defer",
            ExprView::Update(..) => "update",
            ExprView::Default(..) => "default",
            ExprView::IsDefined(..) => "is_defined",
            ExprView::When(..) => "when",
            ExprView::Latch(..) => "latch",
            ExprView::Init(..) => "init",
            ExprView::SIndex(..) => "SIndex",
            ExprView::If(..) => "If then else",
            ExprView::MonitoredAt(..) => "Monitored_at",
            ExprView::Dist(..) => "dist",
            ExprView::List(..) => "List.",
            ExprView::LIndex(..) => "List.get",
            ExprView::LAppend(..) => "List.append",
            ExprView::LConcat(..) => "List.concat",
            ExprView::LHead(..) => "List.head",
            ExprView::LTail(..) => "List.tail",
            ExprView::LLen(..) => "List.len",
            ExprView::LMap(..) => "List.map",
            ExprView::LFilter(..) => "List.filter",
            ExprView::LFold(..) => "List.fold",
            ExprView::Map(..) => "Map.",
            ExprView::MGet(..) => "Map.get",
            ExprView::MInsert(..) => "Map.insert",
            ExprView::MRemove(..) => "Map.remove",
            ExprView::MHasKey(..) => "Map.has_key",
            ExprView::Sin(..) => "sin",
            ExprView::Cos(..) => "cos",
            ExprView::Tan(..) => "tan",
            ExprView::Abs(..) => "abs",
            ExprView::Not(..) => "Not",
            ExprView::Neg(..) => "Neg",
            _ => return None,
        };

        let builtin = get_builtin_by_label(label)?;
        Some(create_hover_item(builtin, &span, &rope))
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

    for name in spec.input_vars() {
        let item = Variables {
            label: name.name(),
            kind: CompletionItemKind::VARIABLE,
            trigger_context: &["expr", "input_stream", "variable"],
            type_anno: None,
            detail: "Input Stream".to_string(),
        };
        items.push(item);
    }
    for name in spec.aux_vars() {
        let item = Variables {
            label: name.name(),
            kind: CompletionItemKind::VARIABLE,
            trigger_context: &["expr", "aux_stream", "variable"],
            type_anno: None,
            detail: "Auxiliary internal stream variable".to_string(),
        };
        items.push(item);
    }
    for name in spec.output_vars() {
        // Auxiliary variables have their own completion detail and should not be duplicated as outputs.
        if !spec.aux_vars().contains(name) {
            let item = Variables {
                label: name.name(),
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

fn create_variable_hover(
    spec: &DsrvSpecification,
    variable: &VarName,
    span: Span,
    rope: &Rope,
) -> Option<Hover> {
    let type_str = spec
        .type_annotation(variable)
        .map(|ty| format!(": {ty}"))
        .unwrap_or_default();
    let (stream_kind, stream_text) = if spec.input_vars().contains(variable) {
        ("in", get_builtin_by_label("in")?.documentation)
    } else if spec.aux_vars().contains(variable) {
        ("aux", get_builtin_by_label("aux")?.documentation)
    } else if spec.output_vars().contains(variable) {
        ("out", get_builtin_by_label("out")?.documentation)
    } else {
        ("stream", "stream")
    };
    let info = format!(
        "```dsrv\n{} {}{}\n```\n---\n{}",
        stream_kind,
        variable.name(),
        type_str,
        stream_text
    );
    Some(create_hover_variable(&info, &span, rope))
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

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    use crate::fixtures;

    use super::*;

    #[apply(async_test)]
    async fn test_get_all_declared_symbols() {
        let input = fixtures::input_untyped_valid_simple();
        let analysis = fixtures::analyse_spec(input).await;
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
        let analysis = fixtures::analyse_spec(input).await;
        let spec = analysis.spec.expect("Expected a valid specification");

        let vars = get_all_declared_symbols(&spec);

        println!("Declared symbols in complex spec: {:#?}", vars);

        assert!(
            vars.len() == 11,
            "Expected 11 declared symbols, found {}",
            vars.len()
        );

        assert!(
            vars[10].label == "safeSwap".to_string(),
            "Expected last declared symbol to be `safeSwap`, found `{}`",
            vars[10].label
        );
        assert!(
            vars[3].label == "swapRequestHelper",
            "Expected 4th declared symbol to be `swapRequestHelper`, found `{}`",
            vars[3].label
        );
    }

    // Realised I never made it able to handle typed variables in the backend.
    // #[apply(async_test)]
    // async fn test_get_all_declared_symbols_typed() {
    //     let input = fixtures::input_typed_valid_simple();
    //     let analysis = fixtures::analyse_spec(input).await;
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
            completion.len() == 4,
            "Expected 4 completions (in, out, aux, var) for empty document, found {}",
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
    async fn test_lambdas_and_folds_completion_and_hover() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();
        let uri = fixtures::create_URI_path();
        let text = fixtures::input_lambdas_and_folds();

        backend.change(uri.clone(), text).await;

        let map_line = text.lines().nth(8).unwrap();
        let completion = backend
            .get_completion(CompletionParams {
                text_document_position: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier { uri: uri.clone() },
                    position: Position {
                        line: 8,
                        character: (map_line.find("x *").unwrap() + 3) as u32,
                    },
                },
                work_done_progress_params: Default::default(),
                partial_result_params: Default::default(),
                context: None,
            })
            .expect("valid lambda document should have completion results");
        assert!(
            completion.iter().any(|item| item.label == "samples"),
            "variable completion lost the real specification: {completion:#?}"
        );

        let map_start = map_line.find("List.map").unwrap() as u32;
        let map_hover = backend.provide_hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position::new(8, map_start),
            },
            work_done_progress_params: Default::default(),
        });
        let Some(Hover {
            contents: HoverContents::Markup(contents),
            ..
        }) = map_hover
        else {
            panic!("expected List.map hover information");
        };
        assert!(contents.value.contains("List.map"));

        let samples_start = map_line.find("samples").unwrap() as u32;
        let samples_hover = backend.provide_hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position::new(8, samples_start),
            },
            work_done_progress_params: Default::default(),
        });
        let Some(Hover {
            contents: HoverContents::Markup(contents),
            ..
        }) = samples_hover
        else {
            panic!("expected lambda expression variable hover information");
        };
        assert!(contents.value.contains("in samples: List<Int>"));
    }

    #[apply(async_test)]
    async fn test_parse_failure_replaces_previous_analysis() {
        let service = fixtures::create_LSP_service();
        let backend = service.inner();
        let uri = fixtures::create_URI_path();
        let valid = fixtures::input_untyped_valid_simple();
        let invalid = fixtures::input_untyped_invalid_simple();

        backend.change(uri.clone(), valid).await;
        let valid_hover = backend.provide_hover(HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri: uri.clone() },
                position: Position::new(0, 3),
            },
            work_done_progress_params: Default::default(),
        });
        assert!(valid_hover.is_some());

        backend.change(uri.clone(), invalid).await;
        assert!(!backend.analysis_map.contains_key(&uri));
        assert!(
            backend
                .provide_hover(HoverParams {
                    text_document_position_params: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        position: Position::new(0, 3),
                    },
                    work_done_progress_params: Default::default(),
                })
                .is_none()
        );
        assert!(
            backend
                .get_completion(CompletionParams {
                    text_document_position: TextDocumentPositionParams {
                        text_document: TextDocumentIdentifier { uri: uri.clone() },
                        position: Position::new(2, 3),
                    },
                    work_done_progress_params: Default::default(),
                    partial_result_params: Default::default(),
                    context: None,
                })
                .is_none()
        );
        assert_eq!(backend.document_map.get(&uri).unwrap().to_string(), invalid);
    }
}
