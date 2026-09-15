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

use std::{cmp::Reverse, str::FromStr};

use crate::{
    lang::syntax::lexer::{Token, tokenize},
    utils::byte_to_pos,
};
use regex::Regex;
use ropey::Rope;
use tower_lsp_server::ls_types::*;
use trustworthiness_checker::lang::dsrv::{
    DsrvParseError, TypeCheckMode,
    ast::{
        CheckedDsrvSpecification, DsrvAstError, DsrvSpecification, ExprRef, Local, SemanticEntry,
    },
    parser::parse_str,
    span::Span,
    type_checker::SemanticError,
};

/// A source symbol whose span is not exposed by the checker AST.
///
/// The checker owns expression spans. Declaration and assignment-LHS spans are
/// parser-local, so this index keeps only the source name and its span for
/// those two editor cases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolSpan {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Analysis {
    /// The checker-owned parsed specification. Expression cursors are created
    /// from this owner only for the duration of an LSP request.
    pub spec: Option<DsrvSpecification>,
    /// Keep the real checked specification so type-check status is represented
    /// by checker data rather than a local marker type.
    pub typed: Option<CheckedDsrvSpecification<Local>>,
    pub diags: Vec<Diagnostic>,
    pub symbol_spans: Vec<SymbolSpan>,
}

impl Analysis {
    /// Synchronous core of the analysis pipeline.
    pub fn analyze_sync(text: &str) -> Analysis {
        Self::analyse_specification_inner(text)
    }

    pub async fn analyse_specification(text: &str) -> Analysis {
        Self::analyse_specification_inner(text)
    }

    pub async fn analyze_specification(text: &str) -> Analysis {
        Self::analyse_specification_inner(text)
    }

    fn analyse_specification_inner(text: &str) -> Analysis {
        let lexer_symbol_spans = Self::symbol_spans(text);
        let spec = match parse_str(text) {
            Ok(spec) => spec,
            Err(error) => {
                log::error!("Parsing error: {error:?}");
                return Analysis {
                    spec: None,
                    typed: None,
                    diags: vec![Self::parse_diag(text, error)],
                    symbol_spans: lexer_symbol_spans,
                };
            }
        };

        // The editor always admits the local language semantically. Preserve
        // the established strict contract for complete annotations, while
        // allowing gradual inference to retain a checked AST during authoring.
        let all_streams_annotated = spec
            .input_vars()
            .iter()
            .chain(spec.roots().map(|(name, _)| name))
            .all(|name| spec.type_annotation(name).is_some());
        let mode = if all_streams_annotated {
            TypeCheckMode::Strict
        } else {
            TypeCheckMode::Gradual
        };
        let symbol_spans = Self::ast_symbol_spans(&spec, &lexer_symbol_spans);
        let type_check_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            spec.clone()
                .validate::<Local>()
                .and_then(|validated| validated.type_check(mode))
        }));

        match type_check_result {
            Ok(Ok(typed)) => Analysis {
                spec: Some(spec),
                typed: Some(typed),
                diags: vec![],
                symbol_spans,
            },
            Ok(Err(errors)) => {
                let rope = Rope::from_str(text);
                let diags = errors
                    .iter()
                    .map(|error| Self::create_semantic_diag(&rope, error))
                    .collect();
                Analysis {
                    spec: Some(spec),
                    typed: None,
                    diags,
                    symbol_spans,
                }
            }
            Err(panic_payload) => {
                // Parsing succeeded, so retain the raw owner and let editor
                // requests continue to use its syntax.
                eprintln!(
                    "[dsrv-lsp] type_check panicked (unimplemented feature?): {:?}",
                    panic_payload
                );
                Analysis {
                    spec: Some(spec),
                    typed: None,
                    diags: vec![],
                    symbol_spans,
                }
            }
        }
    }

    fn ast_symbol_spans(
        spec: &DsrvSpecification,
        lexer_symbol_spans: &[SymbolSpan],
    ) -> Vec<SymbolSpan> {
        spec.semantic_entries()
            .iter()
            .map(|entry| {
                let name = entry.name().name();
                // Declaration spans are authoritative in the checker AST.
                // Assignment entries currently cover the whole equation, so
                // retain the lexer-derived LHS span to avoid swallowing hover
                // requests for expressions on the right-hand side.
                let span = if matches!(entry, SemanticEntry::Assignment { .. }) {
                    lexer_symbol_spans
                        .iter()
                        .find(|symbol| {
                            symbol.name == name
                                && entry.span().contains_offset(symbol.span.start)
                                && entry.span().contains_offset(symbol.span.end)
                        })
                        .map_or_else(|| entry.span(), |symbol| symbol.span)
                } else {
                    entry.span()
                };
                SymbolSpan { name, span }
            })
            .collect()
    }

    fn symbol_spans(source: &str) -> Vec<SymbolSpan> {
        let tokens = tokenize(source);
        let mut symbols = Vec::new();

        for (index, token) in tokens.iter().enumerate() {
            let is_declaration = matches!(
                token.token,
                Token::In | Token::Out | Token::Aux | Token::Var
            );
            if is_declaration {
                if let Some(name) = tokens
                    .get(index + 1)
                    .filter(|next| next.token == Token::Identifier)
                {
                    symbols.push(SymbolSpan {
                        name: name.content.clone(),
                        span: Span {
                            start: token.span.start as u32,
                            end: name.span.end as u32,
                        },
                    });
                }
            } else if token.token == Token::Identifier
                && tokens
                    .get(index + 1)
                    .is_some_and(|next| next.token == Token::Eq)
            {
                symbols.push(SymbolSpan {
                    name: token.content.clone(),
                    span: Span {
                        start: token.span.start as u32,
                        end: token.span.end as u32,
                    },
                });
            }
        }

        symbols.sort_by_key(|symbol| (symbol.span.start, Reverse(symbol.span.end)));
        symbols
    }

    /// Return the smallest checker-owned expression span containing `offset`.
    /// The inclusive boundary behavior matches [`Span::contains_offset`].
    pub fn node_at_offset(&self, offset: u32) -> Option<ExprRef<'_>> {
        self.spec
            .as_ref()?
            .nodes()
            .filter(|node| node.span().contains_offset(offset))
            .min_by_key(|node| node.span().len())
    }

    pub fn symbol_at_offset(&self, offset: u32) -> Option<&SymbolSpan> {
        self.symbol_spans
            .iter()
            .filter(|symbol| symbol.span.contains_offset(offset))
            .min_by_key(|symbol| symbol.span.len())
    }

    fn semantic_error_message(error: &SemanticError) -> String {
        match error {
            SemanticError::DuplicateDeclaration { variable, .. } => {
                format!("Duplicate declaration: `{}`", variable.name())
            }
            SemanticError::TypeError(error) => format!("Type error: {}", error.message()),
            SemanticError::DeferredError(message, _) => format!("Deferred error: {message}"),
            SemanticError::UndeclaredVariable(message, _) => {
                format!("Undeclared variable: {message}")
            }
            SemanticError::MissingTypeAnnotation(message, _) => {
                format!("Missing type annotation: {message}")
            }
            SemanticError::MissingTypeAscription(message, _) => {
                format!("Missing type ascription: {message}")
            }
            SemanticError::UnsupportedDistributionConstraint(message, _) => {
                format!("Unsupported distribution constraint: {message}")
            }
            SemanticError::UnsupportedLiteral(message, _) => {
                format!("Unsupported literal: {message}")
            }
            SemanticError::UnsupportedExpression(message, _) => {
                format!("Unsupported expression: {message}")
            }
            SemanticError::InvalidRuntimeScope(message, _) => {
                format!("Invalid runtime scope: {message}")
            }
            SemanticError::UnresolvedType(error) => {
                format!("Unresolved type: {}", error.message())
            }
        }
    }

    fn parse_diag(text: &str, error: DsrvParseError) -> Diagnostic {
        let details = format!("{error:?}\n{error}");
        let position = Self::parse_error_position(&details);
        let (message, end) =
            if details.contains("UnrecognizedEof") || details.contains("Unrecognized EOF") {
                ("Syntax error: Unexpected EOF", position)
            } else if details.contains("InvalidToken") {
                ("Syntax error: Invalid Token", position)
            } else if details.contains("Unrecognized token") {
                // The current LALR parser reports invalid characters as an
                // unrecognized token. Preserve the old, more useful diagnostic
                // for characters that cannot be part of DSRV syntax.
                if Self::source_char_at(text, position) == Some('\\') {
                    ("Syntax error: Invalid Token", position)
                } else {
                    (
                        "Syntax error: Unrecognized token",
                        Position::new(position.line, position.character.saturating_add(1)),
                    )
                }
            } else if details.contains("ExtraToken") {
                (
                    "Syntax error: Extra token:",
                    Position::new(position.line, position.character.saturating_add(1)),
                )
            } else if details.contains("UnrecognizedToken") {
                (
                    "Syntax error: Unrecognized token",
                    Position::new(position.line, position.character.saturating_add(1)),
                )
            } else if let DsrvParseError::Ast(ast_error) = &error {
                let range = Self::ast_error_span(ast_error)
                    .map(|span| Self::span_range(text, span))
                    .unwrap_or_else(|| Range::new(position, position));
                return Self::create_diag(&format!("Syntax error: {ast_error}"), range);
            } else {
                ("Syntax error: Invalid DSRV syntax", position)
            };

        Self::create_diag(message, Range::new(position, end))
    }

    fn source_char_at(text: &str, position: Position) -> Option<char> {
        text.lines()
            .nth(position.line as usize)
            .and_then(|line| line.as_bytes().get(position.character as usize))
            .map(|byte| *byte as char)
    }

    fn parse_error_position(details: &str) -> Position {
        let pattern = Regex::new(r"line\s+(\d+),\s*column\s+(\d+)")
            .expect("parse error location regex is valid");
        let Some(captures) = pattern.captures(details) else {
            return Position::new(0, 0);
        };
        let line = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<u32>().ok())
            .unwrap_or(1)
            .saturating_sub(1);
        let character = captures
            .get(2)
            .and_then(|value| value.as_str().parse::<u32>().ok())
            .unwrap_or(1)
            .saturating_sub(1);
        Position::new(line, character)
    }

    fn ast_error_span(error: &DsrvAstError) -> Option<Span> {
        match error {
            DsrvAstError::DuplicateAssignment { duplicate, .. } => Some(*duplicate),
            DsrvAstError::DuplicateExpressionField { .. }
            | DsrvAstError::InvalidExpressionForest(_)
            | DsrvAstError::InvalidExpressionMap(_) => None,
        }
    }

    fn span_range(text: &str, span: Span) -> Range {
        let rope = Rope::from_str(text);
        Range::new(
            byte_to_pos(&rope, span.start as usize).unwrap_or_default(),
            byte_to_pos(&rope, span.end as usize).unwrap_or_default(),
        )
    }

    fn create_diag(msg: &str, range: Range) -> Diagnostic {
        Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("DSRV".into()),
            message: msg.into(),
            ..Default::default()
        }
    }

    fn create_semantic_diag(rope: &Rope, error: &SemanticError) -> Diagnostic {
        let range = error
            .span()
            .map(|span| Range {
                start: byte_to_pos(rope, span.start as usize).unwrap_or_default(),
                end: byte_to_pos(rope, span.end as usize).unwrap_or_default(),
            })
            .unwrap_or_default();
        let mut diagnostic = Self::create_diag(&Self::semantic_error_message(error), range);
        if let SemanticError::DuplicateDeclaration {
            variable, first, ..
        } = error
        {
            diagnostic.related_information = Some(vec![DiagnosticRelatedInformation {
                location: Location {
                    // `Backend::change` replaces this placeholder with the
                    // document URI before publishing the diagnostic.
                    uri: Uri::from_str("file:///").expect("placeholder URI is valid"),
                    range: Range {
                        start: byte_to_pos(rope, first.start as usize).unwrap_or_default(),
                        end: byte_to_pos(rope, first.end as usize).unwrap_or_default(),
                    },
                },
                message: format!("First declaration of `{}`", variable.name()),
            }]);
        }
        diagnostic
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixtures;
    use macro_rules_attribute::apply;
    use trustworthiness_checker::{async_test, lang::dsrv::ast::ExprView};

    #[apply(async_test)]
    async fn test_analyse_syntax_valid_input_gets_gradual_types() {
        let analysis = fixtures::analyse_spec(fixtures::input_untyped_valid_simple()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_some());
    }

    #[apply(async_test)]
    async fn test_analyse_syntax_valid_input_typed() {
        let analysis = fixtures::analyse_spec(fixtures::input_typed_valid_simple()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert_eq!(analysis.spec.as_ref().unwrap().type_annotations().len(), 3);
        assert!(analysis.typed.is_some());
    }

    #[apply(async_test)]
    async fn test_analyse_empty_input() {
        let analysis = fixtures::analyse_spec(fixtures::input_empty()).await;
        let spec = analysis.spec.as_ref().unwrap();
        assert!(spec.input_vars().is_empty());
        assert!(spec.output_vars().is_empty());
        assert!(spec.aux_vars().is_empty());
        assert!(spec.type_annotations().is_empty());
        assert!(spec.nodes().next().is_none());
    }

    #[apply(async_test)]
    async fn test_analyse_syntax_invalid_input() {
        let analysis = fixtures::analyse_spec(fixtures::input_untyped_invalid_simple()).await;
        assert!(!analysis.diags.is_empty());
        assert!(analysis.spec.is_none());
    }

    #[apply(async_test)]
    async fn test_analyse_unformatted_input() {
        let analysis =
            fixtures::analyse_spec(fixtures::input_untyped_long_valid_unformatted()).await;
        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_none());
        assert!(!analysis.diags.is_empty());
    }

    #[apply(async_test)]
    async fn test_very_long_input() {
        let analysis = fixtures::analyse_spec(fixtures::input_long()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spec.is_some());
    }

    #[apply(async_test)]
    async fn test_analyse_syntax_error_invalid_token() {
        let analysis = fixtures::analyse_spec(fixtures::input_parseError_invalid_token()).await;
        assert!(!analysis.diags.is_empty());
        assert_eq!(analysis.diags[0].message, "Syntax error: Invalid Token");
        assert_eq!(
            analysis.diags[0].range,
            Range::new(Position::new(4, 7), Position::new(4, 7))
        );
    }

    #[apply(async_test)]
    async fn test_analyse_syntax_error_unrecognized_eof() {
        let analysis = fixtures::analyse_spec(fixtures::input_parseError_unrecognizedEOF()).await;
        assert!(!analysis.diags.is_empty());
        assert_eq!(analysis.diags[0].message, "Syntax error: Unexpected EOF");
    }

    #[apply(async_test)]
    async fn test_analyse_syntax_error_unrecognized_token() {
        let analysis =
            fixtures::analyse_spec(fixtures::input_parseError_unrecognized_token()).await;
        assert!(!analysis.diags.is_empty());
        assert_eq!(
            analysis.diags[0].message,
            "Syntax error: Unrecognized token"
        );
    }

    #[apply(async_test)]
    async fn test_analyse_type_error() {
        let analysis = fixtures::analyse_spec(fixtures::input_typed_invalid_simple()).await;
        assert!(!analysis.diags.is_empty());
        assert!(analysis.typed.is_none());
    }

    #[apply(async_test)]
    async fn test_analyse_semantic_undeclared_variable() {
        let analysis = fixtures::analyse_spec(fixtures::input_semantic_undeclared_var()).await;
        assert!(!analysis.diags.is_empty());
        assert!(analysis.diags[0].message.contains("Undeclared variable:"));
    }

    #[apply(async_test)]
    async fn test_analyse_semantic_type_error() {
        let analysis = fixtures::analyse_spec(fixtures::input_semantic_type_error()).await;
        assert!(!analysis.diags.is_empty());
        assert!(analysis.diags[0].message.contains("Type error:"));
    }

    #[test]
    fn duplicate_declaration_uses_duplicate_as_primary_and_first_as_related() {
        let source = "in x\nin x\nout y\ny = x";
        let analysis = Analysis::analyze_sync(source);

        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_none());
        let diagnostic = analysis
            .diags
            .iter()
            .find(|diagnostic| diagnostic.message.contains("Duplicate declaration"))
            .expect("duplicate declaration diagnostic");
        assert_eq!(
            diagnostic.range,
            Range::new(Position::new(1, 0), Position::new(1, 4))
        );
        let related = diagnostic
            .related_information
            .as_ref()
            .and_then(|information| information.first())
            .expect("first declaration related information");
        assert_eq!(
            related.location.range,
            Range::new(Position::new(0, 0), Position::new(0, 4))
        );
    }

    #[test]
    fn every_duplicate_points_to_the_original_declaration() {
        let source = "in x\nin x\nin x\nout y\ny = x";
        let analysis = Analysis::analyze_sync(source);
        let duplicate_diagnostics = analysis
            .diags
            .iter()
            .filter(|diagnostic| diagnostic.message.contains("Duplicate declaration"))
            .collect::<Vec<_>>();

        assert_eq!(duplicate_diagnostics.len(), 2);
        assert_eq!(
            duplicate_diagnostics[0].range,
            Range::new(Position::new(1, 0), Position::new(1, 4))
        );
        assert_eq!(
            duplicate_diagnostics[1].range,
            Range::new(Position::new(2, 0), Position::new(2, 4))
        );
        for diagnostic in duplicate_diagnostics {
            let related = diagnostic
                .related_information
                .as_ref()
                .and_then(|information| information.first())
                .expect("original declaration related information");
            assert_eq!(
                related.location.range,
                Range::new(Position::new(0, 0), Position::new(0, 4))
            );
        }
    }

    #[test]
    fn assignment_only_model_uses_gradual_checking() {
        let analysis = Analysis::analyze_sync("x = 1");

        assert!(analysis.diags.is_empty(), "{:?}", analysis.diags);
        let typed = analysis
            .typed
            .expect("gradually checked assignment-only model");
        assert_eq!(
            typed.type_annotation(&"x".into()).map(ToString::to_string),
            Some("Int".to_owned())
        );
    }

    #[test]
    fn cross_role_declaration_is_rejected_but_retains_raw_ast() {
        let analysis = Analysis::analyze_sync("in shared\nout shared\nshared = 1");

        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_none());
        assert!(
            analysis
                .diags
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Duplicate declaration"))
        );
    }

    #[test]
    fn test_parse_new_upstream_expression_forms() {
        for source in [
            "out z\nz = -1",
            "out z\nz = Tuple(1, 2)",
            "out z\nz = {value: 1}",
            r#"out z
z = Struct("value": 1).value"#,
        ] {
            let analysis = Analysis::analyze_sync(source);
            assert!(
                analysis.diags.is_empty(),
                "failed to parse {source:?}: {analysis:?}"
            );
            assert!(analysis.spec.is_some());
        }
    }

    #[test]
    fn test_node_at_offset_uses_real_ast_and_inclusive_boundaries() {
        let source = "out z\nz = x + y";
        let analysis = Analysis::analyze_sync(source);
        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_none());
        assert!(!analysis.diags.is_empty());

        let x_start = source.find('x').unwrap() as u32;
        let x_end = x_start + 1;
        for offset in [x_start, x_end] {
            let node = analysis.node_at_offset(offset).unwrap();
            assert!(matches!(node.view(), ExprView::Var(name) if name.name() == "x"));
        }

        let y_start = source.find('y').unwrap() as u32;
        let node = analysis.node_at_offset(y_start).unwrap();
        assert!(matches!(node.view(), ExprView::Var(name) if name.name() == "y"));
    }

    #[test]
    fn test_lambdas_and_folds_example() {
        let source = r#"in samples: List<Int>
in bias: Int
out doubled: List<Int>
out positives: List<Int>
out sum: Int
out adjustedSum: Int
out allPositive: Bool

doubled = List.map(\x: Int -> x * 2, samples)
positives = List.filter(\x: Int -> x > 0, samples)
sum = List.fold(\acc: Int, x: Int -> acc + x, 0, samples)
adjustedSum = (\total: Int -> total + bias)(sum)
allPositive = List.fold(\acc: Bool, x: Int -> acc && (x > 0), true, samples)
"#;

        let analysis = Analysis::analyze_sync(source);
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {:#?}",
            analysis.diags
        );
        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_some());
    }

    #[apply(async_test)]
    async fn test_analyse_untyped_with_comments() {
        let analysis = fixtures::analyse_spec(fixtures::input_untyped_simple_with_comments()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        let symbols = &analysis.symbol_spans;
        assert!(symbols.len() >= 3);
        assert!(symbols[2].span.start > symbols[1].span.end);
    }

    #[apply(async_test)]
    async fn test_analyse_untyped_complex() {
        let analysis =
            fixtures::analyse_spec(fixtures::input_untyped_complex_with_comments()).await;
        assert!(analysis.spec.is_some());
        assert!(analysis.typed.is_none());
        assert!(!analysis.diags.is_empty());
        assert!(analysis.spec.as_ref().unwrap().nodes().next().is_some());
    }
}
