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

use crate::{lang::pattern_matching::extract_from_stmts, utils::*};
use lalrpop_util::ParseError;
use ropey::Rope;
// use tower_lsp::lsp_types::*;
use tower_lsp_server::ls_types::*;
use trustworthiness_checker::lang::dsrv::{
    ast::{DsrvSpecification, SpannedExpr},
    lalr::TopDeclsParser,
    lalr_parser::create_dsrv_spec,
    span::Span,
    type_checker::{SemanticError, TypedDsrvSpecification, type_check},
};

#[derive(Clone, Debug)]
pub struct Analysis {
    pub spec: Option<DsrvSpecification>, // The parsed specification, if parsing was successful
    pub typed: Option<TypedDsrvSpecification>, //For future use, when type checker is implemented
    pub diags: Vec<Diagnostic>,          // Diagnostics from both syntax and semantic analysis
    pub spanned_nodes: Vec<SpannedExpr>, // A vector of all expressions in the spec annotated with their spans
}

impl Analysis {
    // Create Clone function for Analysis struct
    pub async fn analyze_specification(text: &str) -> Analysis {
        match TopDeclsParser::new().parse(text) {
            Ok(stmts) => {
                log::info!("stmts: {:#?}", stmts);
                // log::info!("stmts: {:?}", stmts[0]);
                // log::info!("lenth: {:?}", stmts.len());
                // log::info!("Parsed specification: {:#?}", spec);

                // Use the pattern matching function to extract all spanned nodes into a flat vector.
                let mut nodes = Vec::new();
                extract_from_stmts(&stmts, &mut nodes);
                // log::info!("Extracted spanned nodes: {:#?}", nodes);

                // Create the DSRV specification from the parsed statements for type_checker and semantic errors
                let spec = create_dsrv_spec(&stmts);
                if !(spec.type_annotations.is_empty()) {
                    match type_check(spec.clone()) {
                        Ok(s) => {
                            // log::info!("Type checked specification: {:?}", s);
                            return Analysis {
                                spec: Some(spec.clone()),
                                typed: Some(s.clone()),
                                diags: vec![],
                                spanned_nodes: nodes.clone(),
                            };
                        }
                        Err(errs) => {
                            // log::error!("Type checking errors: {:#?}", errs);

                            let mut diags_vec: Vec<Diagnostic> = Vec::new();

                            for error in errs {
                                let rope = Rope::from_str(text);
                                match error {
                                    SemanticError::DeferredError(msg, span) => {
                                        diags_vec
                                            .push(Self::create_semantic_diag(&rope, &msg, span));
                                    }
                                    SemanticError::TypeError(msg, span) => {
                                        diags_vec
                                            .push(Self::create_semantic_diag(&rope, &msg, span));
                                    }
                                    SemanticError::UndeclaredVariable(msg, span) => {
                                        diags_vec
                                            .push(Self::create_semantic_diag(&rope, &msg, span));
                                    }
                                }
                            }
                            return Analysis {
                                spec: Some(spec.clone()),
                                typed: None,
                                diags: diags_vec,
                                spanned_nodes: nodes.clone(),
                            };
                        }
                    }
                }
                Analysis {
                    spec: Some(spec.clone()),
                    typed: None,
                    diags: vec![],
                    spanned_nodes: nodes,
                }
            }

            Err(error) => {
                log::error!("Parsing error: {:#?}", error);
                // Map the error's byte positions to line and column positions in the text_document immediately.
                let error = error.map_location(|byte| byte_to_pos(&Rope::from_str(text), byte));

                // Convert the parse error into a diagnostic message with a range indicating where the error occurred in the source code
                let diags = match error {
                    ParseError::InvalidToken { location } => {
                        let range =
                            Range::new(location.unwrap_or_default(), location.unwrap_or_default());
                        Self::create_diag("Invalid Token", range)
                    }
                    ParseError::UnrecognizedEof {
                        location,
                        expected: _,
                    } => {
                        let range = Range {
                            start: location.unwrap_or_default(),
                            end: location.unwrap_or_default(),
                        };

                        Self::create_diag("Syntax error: Unexpected EOF", range)
                    }

                    ParseError::UnrecognizedToken { token, expected: _ } => {
                        let (start, _tok, end) = token;
                        Self::create_diag(
                            "Syntax error: Unrecognized token",
                            Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
                        )
                    }
                    ParseError::ExtraToken { token } => {
                        let (start, _tok, end) = token;

                        Self::create_diag(
                            "Syntax error: Extra token:",
                            Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
                        )
                    }

                    ParseError::User { error } => {
                        let p = Position::new(1, 1);
                        Self::create_diag(&format!("User error: {:?}", error), Range::new(p, p))
                    }
                };

                // Return the analysis result with the diagnostic message
                Analysis {
                    spec: None,
                    typed: None,
                    diags: vec![diags],
                    spanned_nodes: vec![],
                }
            }
        }
    }
    fn create_diag(msg: &str, range: Range) -> Diagnostic {
        Diagnostic {
            range: range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("DSRV".into()),
            message: msg.into(),
            ..Default::default()
        }
    }

    fn create_semantic_diag(rope: &Rope, msg: &str, span: Span) -> Diagnostic {
        let range = Range {
            start: byte_to_pos(&rope, span.start as usize).unwrap_or_default(),
            end: byte_to_pos(&rope, span.end as usize).unwrap_or_default(),
        };

        log::info!("msg: {:?}", &msg);
        // let msg_formatted = regex_format(&msg);

        Self::create_diag(&msg, range)
    }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;
    use crate::test_utils;
    use super::*;

    #[apply(async_test)]
    async fn test_analyze_syntax_valid_input_not_typed() {
        let input = test_utils::input_valid_simple();
        let analysis = test_utils::analyze_spec(input).await;

        assert!(
            analysis.diags.is_empty(),
            "Expected no diagnostics for valid input, but got: {:?}",
            analysis
        );

        assert!(
            analysis.spec.is_some(),
            "Expected spec to be Some for valid input, but got: {:?}",
            analysis
        );
    }

    #[apply(async_test)]
    async fn test_analyze_syntax_valid_input_typed() {
        let input = test_utils::input_valid_typed();
        let analysis = test_utils::analyze_spec(input).await;

        // println!("Analysis result: {:#?}", analysis.clone());

        assert!(
            analysis.diags.is_empty(),
            "Expected no diagnostics for valid input, but got: {:?}",
            analysis.diags
        );

        assert!(
            analysis.spec.as_ref().unwrap().type_annotations.len() == 3,
            "Expected 3 type annotations, but got: {:?}",
            analysis.spec.as_ref().unwrap().type_annotations.len()
        );

        assert!(
            analysis.typed.is_some(),
            "Expected typed to be Some for valid input with type annotations, got: {:?}",
            analysis.typed
        );
    }

    #[apply(async_test)]
    async fn test_analyze_empty_input() {
        let input = test_utils::input_empty();
        let analysis = test_utils::analyze_spec(input).await;
        let spec = analysis.spec.as_ref().unwrap();

        // println!("{:#?}", analysis);

        assert!(
            spec.input_vars.is_empty(),
            "Expected no input variables, but got: {:?}",
            spec.input_vars
        );
        assert!(
            spec.output_vars.is_empty(),
            "Expected no output variables, but got: {:?}",
            spec.output_vars
        );
        assert!(
            spec.aux_info.is_empty(),
            "Expected no auxiliary variables, but got: {:?}",
            spec.aux_info
        );
        assert!(
            spec.type_annotations.is_empty(),
            "Expected no type annotations, but got: {:?}",
            spec.type_annotations
        );

        assert!(
            spec.exprs.is_empty(),
            "Expected no expressions, but got: {:?}",
            spec.exprs
        );
    }

    #[apply(async_test)]
    async fn test_analyze_syntax_invalid_input() {
        let input = test_utils::input_invalid_simple();
        let analysis =test_utils::analyze_spec(input).await;

        println!("Analysis result: {:#?}", analysis);

        assert!(
            !analysis.diags.is_empty(),
            "Expected diagnostics for invalid syntax, but got none"
        );
        assert_eq!(
            analysis.diags.first().unwrap().message,
            "Syntax error: Unexpected EOF",
            "Expected diagnostic message for unexpected EOF, but got: {:?}",
            analysis.diags.first().unwrap().message
        );
        assert!(
            analysis.spec.is_none(),
            "Expected spec to be None for invalid syntax, but got: {:?}",
            analysis.spec
        )
    }

    #[apply(async_test)]
    async fn test_analyze_semantic_type_error() {
        let input = test_utils::input_invalid_typed();
        let analysis = test_utils::analyze_spec(input).await;

        println!("Analysis result: {:#?}", analysis);
        assert!(
            !analysis.diags.is_empty(),
            "Expected diagnostics for type error, but got none"
        );
        assert!(
            analysis.diags[0].message.contains("x"),
            "Expected diagnostic message to mention 'x', but got: {:?}",
            analysis.diags[0].message
        );
        assert!(
            analysis.diags[0].message.contains("y"),
            "Expected diagnostic message to mention 'y', but got: {:?}",
            analysis.diags[0].message
        );
        assert!(
            analysis.typed.is_none(),
            "Expected typed spec to be None for input with type errors, got: {:?}",
            analysis.typed
        );
    }

    #[apply(async_test)]
    async fn test_analyze_semantic_undeclared_variable() {
        let input = test_utils::input_semantic_undeclared_var();
        let analysis = test_utils::analyze_spec(input).await;

        println!("Analysis result: {:#?}", analysis);

        assert!(
            !analysis.diags.is_empty(),
            "Expected diagnostics for undeclared variable 'y', but got none"
        );

        // assert_eq!(analysis.diags[0].message, "Usage of undeclared variable: y", "Expected diagnostic message for undeclared variable 'y', but got: {:?}", analysis.diags[0].message);
        assert!(
            analysis.diags[0].message.contains("y"),
            "Expected diagnostic message to mention undeclared variable 'y', but got: {:?}",
            analysis.diags[0].message
        );
    }

    #[apply(async_test)]
    async fn test_create_diags_function() {
        let range = Range::new(Position::new(1, 1), Position::new(1, 5));
        let diag = Analysis::create_diag("Test error message", range.clone());

        println!("Diagnostic: {:#?}", diag);

        assert_eq!(
            diag.message, "Test error message",
            "Expected diagnostic message to match input, but got: {:?}",
            diag.message
        );
        assert_eq!(
            diag.range.start, range.start,
            "Expected diagnostic range start to match input, but got: {:?}",
            diag.range.start
        );
        assert_eq!(
            diag.range.end, range.end,
            "Expected diagnostic range end to match input, but got: {:?}",
            diag.range.end
        );

        assert_eq!(
            diag.severity,
            Some(DiagnosticSeverity::ERROR),
            "Expected diagnostic severity to be ERROR, but got: {:?}",
            diag.severity
        );
    }

    #[apply(async_test)]
    async fn test_create_semancic_diags_function() {
        let range = Range::new(Position::new(1, 1), Position::new(1, 5));
        let diag = Analysis::create_diag("Test error semantic message", range.clone());

        println!("Diagnostic: {:#?}", diag);

        assert_eq!(
            diag.message, "Test error semantic message",
            "Expected diagnostic message to match input, but got: {:?}",
            diag.message
        );
        assert_eq!(
            diag.range.start, range.start,
            "Expected diagnostic range start to match input, but got: {:?}",
            diag.range.start
        );
        assert_eq!(
            diag.range.end, range.end,
            "Expected diagnostic range end to match input, but got: {:?}",
            diag.range.end
        );

        assert_eq!(
            diag.severity,
            Some(DiagnosticSeverity::ERROR),
            "Expected diagnostic severity to be ERROR, but got: {:?}",
            diag.severity
        );
    }
}
