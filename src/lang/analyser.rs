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

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    lang::pattern_matching::{SpannedExpr, extract_from_stmts},
    utils::byte_to_pos,
};
use regex::Regex;
use ropey::Rope;
use tower_lsp_server::ls_types::*;
use trustworthiness_checker::lang::dsrv::{
    DsrvParseError, TypeCheckOptions,
    ast::{DsrvAstError, DsrvSpecification},
    parser::parse_str,
    span::Span,
    type_checker::SemanticError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecificationSnapshot {
    input_vars: BTreeSet<trustworthiness_checker::VarName>,
    output_vars: BTreeSet<trustworthiness_checker::VarName>,
    aux_vars: BTreeSet<trustworthiness_checker::VarName>,
    type_annotations: BTreeMap<trustworthiness_checker::VarName, String>,
}

impl SpecificationSnapshot {
    fn from_checker(spec: &DsrvSpecification) -> Self {
        Self {
            input_vars: spec.input_vars().clone(),
            output_vars: spec.output_vars().clone(),
            aux_vars: spec.aux_vars().clone(),
            type_annotations: spec
                .type_annotations()
                .iter()
                .map(|(name, ty)| (name.clone(), format!("{ty:?}")))
                .collect(),
        }
    }

    pub fn input_vars(&self) -> &BTreeSet<trustworthiness_checker::VarName> {
        &self.input_vars
    }

    pub fn output_vars(&self) -> &BTreeSet<trustworthiness_checker::VarName> {
        &self.output_vars
    }

    pub fn aux_vars(&self) -> &BTreeSet<trustworthiness_checker::VarName> {
        &self.aux_vars
    }

    pub fn type_annotations(&self) -> &BTreeMap<trustworthiness_checker::VarName, String> {
        &self.type_annotations
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckedSpecification;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Analysis {
    pub spec: Option<SpecificationSnapshot>,
    pub typed: Option<TypeCheckedSpecification>,
    pub diags: Vec<Diagnostic>,
    pub spanned_nodes: Vec<SpannedExpr>,
}

impl Analysis {
    /// Synchronous core of the analysis pipeline.
    ///
    /// The checker now owns parsing and expression storage, so the LSP parses
    /// through its public `parse_str` API and keeps only lightweight node
    /// snapshots for editor offset lookups. Strict checking is retained for
    /// documents that contain type annotations; completely untyped documents
    /// continue to use the LSP's syntax-only behavior.
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
        let spec = match parse_str(text) {
            Ok(spec) => spec,
            Err(error) => {
                log::error!("Parsing error: {error:?}");
                return Analysis {
                    spec: None,
                    typed: None,
                    diags: vec![Self::parse_diag(text, error)],
                    spanned_nodes: vec![],
                };
            }
        };

        let spec_snapshot = SpecificationSnapshot::from_checker(&spec);
        let mut nodes = Vec::new();
        extract_from_stmts(&spec, text, &mut nodes);
        log::info!("Extracted spanned nodes: {:#?}", nodes);

        if !spec.type_annotations().is_empty() {
            let type_check_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                spec.clone().type_check(TypeCheckOptions::STRICT)
            }));

            match type_check_result {
                Ok(Ok(_typed)) => {
                    return Analysis {
                        spec: Some(spec_snapshot.clone()),
                        typed: Some(TypeCheckedSpecification),
                        diags: vec![],
                        spanned_nodes: nodes,
                    };
                }
                Ok(Err(errors)) => {
                    let rope = Rope::from_str(text);
                    let diags = errors
                        .iter()
                        .map(|error| {
                            let message = Self::semantic_error_message(error);
                            Self::create_semantic_diag(&rope, &message, error.span())
                        })
                        .collect();
                    return Analysis {
                        spec: Some(spec_snapshot.clone()),
                        typed: None,
                        diags,
                        spanned_nodes: nodes,
                    };
                }
                Err(panic_payload) => {
                    // Keep the server alive if the checker encounters a
                    // feature it cannot yet type-check.
                    eprintln!(
                        "[dsrv-lsp] type_check panicked (unimplemented feature?): {:?}",
                        panic_payload
                    );
                }
            }
        }

        Analysis {
            spec: Some(spec_snapshot),
            typed: None,
            diags: vec![],
            spanned_nodes: nodes,
        }
    }

    fn semantic_error_message(error: &SemanticError) -> String {
        match error {
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

    fn create_semantic_diag(rope: &Rope, msg: &str, span: Option<Span>) -> Diagnostic {
        let range = span
            .map(|span| Range {
                start: byte_to_pos(rope, span.start as usize).unwrap_or_default(),
                end: byte_to_pos(rope, span.end as usize).unwrap_or_default(),
            })
            .unwrap_or_default();
        Self::create_diag(msg, range)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixtures;
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    #[apply(async_test)]
    async fn test_analyse_syntax_valid_input_not_typed() {
        let analysis = fixtures::analyse_spec(fixtures::input_untyped_valid_simple()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spec.is_some());
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
        assert!(analysis.spanned_nodes.is_empty());
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
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spec.is_some());
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

    #[apply(async_test)]
    async fn test_analyse_untyped_with_comments() {
        let analysis = fixtures::analyse_spec(fixtures::input_untyped_simple_with_comments()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spanned_nodes.len() >= 3);
        assert!(analysis.spanned_nodes[2].span.start > analysis.spanned_nodes[1].span.end);
    }

    #[apply(async_test)]
    async fn test_analyse_untyped_complex() {
        let analysis =
            fixtures::analyse_spec(fixtures::input_untyped_complex_with_comments()).await;
        assert!(
            analysis.diags.is_empty(),
            "unexpected diagnostics: {analysis:?}"
        );
        assert!(analysis.spec.is_some());
        assert!(!analysis.spanned_nodes.is_empty());
    }
}
