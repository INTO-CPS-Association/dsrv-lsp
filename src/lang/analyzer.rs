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
use regex::Regex;
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
                // log::info!("stmts: {:#?}", stmts);
                // log::info!("stmts: {:?}", stmts[0]);
                // log::info!("lenth: {:?}", stmts.len());
                // log::info!("Parsed specification: {:#?}", spec);

                // Use the pattern matching function to extract all spanned nodes into a flat vector.
                let mut nodes = Vec::new();
                extract_from_stmts(&stmts, &mut nodes);
                log::info!("Extracted spanned nodes: {:#?}", nodes);

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

        let msg_formatted = regex_format(&msg);

        Self::create_diag(&msg_formatted, range)
    }
}

fn regex_format(msg: &str) -> String {
    let re = Regex::new(r#"(\w+)\(Var\(VarName::new\("(\w+)"\)\)\)"#).unwrap();

    let result = re
        .replace_all(&msg, |caps: &regex::Captures| {
            let var_type = match &caps[1] {
                "Int" => "integer",
                "Float" => "float",
                "Str" => "string",
                "Bool" => "boolean",
                "Unit" => "Unit",
                _ => "unknown",
            };
            format!("{} `{}`", var_type, &caps[2])
        })
        .to_string();

    let re2 = Regex::new(r#"(\w+)\(Val\(Known\("*?(\w+)"*?\)\)\)"#).unwrap();
    let result = re2.replace_all(&result, |caps: &regex::Captures| {
        let var_type = match &caps[1] {
            "Int" => "integer",
            "Float" => "float",
            "Str" => "string",
            "Bool" => "boolean",
            "Unit" => "Unit",
            _ => "unknown",
        };
        format!("{} `{}`", var_type, &caps[2])
    });

    let re3 = Regex::new(r#"binary function (?P<kind>[SNB])Op\((\w+)\)"#).unwrap();

    let result = re3.replace_all(&result, |caps: &regex::Captures| {
        let op_type = match &caps["kind"] {
            "S" => "String",
            "N" => "Numerical",
            "B" => "Boolean",
            _ => "unknown",
        };
        format!("`{} {}`", op_type, &caps[2])
    });

    result.into_owned().trim().to_string()
}

// "Numerical operation not valid on integers".into(),
// "Numerical operation not valid on floats".into(),
// "Cannot apply binary function {:?} to expressions of type {:?} and {:?}",
// "Cannot create default-expression with two different types: {:?} and {:?}",
// "Cannot create if-expression with two different types: {:?} and {:?}",
// "If expression condition must be a boolean".into(),
// "Mismatched type in Stream Index expression, expression and default does not match: {:?}",
// "Type mismatch: expected {:?}, got {:?}",
// "Type ascription required for dynamic"
// "Expected Dynamic to be applied to a Str, got {:?}",
// "Expected RestrictedDynamic to be applied to a Str, got {:?}",
// "Type ascription required for restricted dynamic"
// "Type ascription required for defer"
// "Expected Defer to be applied to a Str, got {:?}",
// "Not can only be applied to boolean expressions".into(),
// "Init requires both arguments to have the same type, got {:?} and {:?}",
// "Sin can only be applied to float expressions, got {:?}",
// "Cos can only be applied to float expressions, got {:?}",
// "Tan can only be applied to float expressions, got {:?}",
// "Abs can only be applied to numeric expressions, got {:?}",
// "Usage of undeclared variable: {:?}",
//"Stream expression {:?} not assigned a type before semantic analysis",

#[cfg(test)]
mod test {
    use trustworthiness_checker::async_test;
    use macro_rules_attribute::apply;

    use super::*;

    #[apply(async_test)]
    async fn test_analyze_syntax_valid_input_not_typed() {
        let input = "in x\nin y\nout z\n\nz = x + y";
        let analysis = Analysis::analyze_specification(input).await;

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
        let input2 = "in x: Int\nin y: Int\nout z: Int\n\nz = x + y";
        let analysis = Analysis::analyze_specification(input2).await;

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
}
