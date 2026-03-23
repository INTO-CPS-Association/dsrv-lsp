use crate::{lang::pattern_matching::extract_nodes, utils::*};
use lalrpop_util::ParseError;
use regex::Regex;
use ropey::Rope;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dsrv::{
    ast::{DsrvSpecification, SpannedExpr},
    lalr::TopDeclsParser,
    lalr_parser::create_dsrv_spec,
    span::Span,
    type_checker::{SemanticError, TypedDsrvSpecification, type_check},
};

// dynamic_lola::{
//     ast::{, SpannedExpr}, lalr::TopDeclsParser, lalr_parser::create_lola_spec, type_checker::TypedLOLASpecification
// };

#[derive(Clone, Debug)]
pub struct Analysis {
    pub spec: Option<DsrvSpecification>, // The parsed specification, if parsing was successful
    pub typed: Option<TypedDsrvSpecification>, //For future use, when type checker is implemented
    pub diags: Vec<Diagnostic>,          // Diagnostics from both syntax and semantic analysis
    pub spanned_nodes: Vec<SpannedExpr>, // A vector of all expressions in the spec annotated with their spans
}

impl Analysis {
    // Create Clone function for Analysis struct
    pub async fn analyze_2_point_0(text: &str) -> Analysis {
        match TopDeclsParser::new().parse(text) {
            Ok(stmts) => {
                let spec = create_dsrv_spec(&stmts);
                // log::info!("Parsed specification: {:#?}", spec);
                let mut nodes = Vec::new();

                for (_name, expr) in &spec.exprs {
                    extract_nodes(expr, &mut nodes);
                }
                // log::info!("Extracted spanned nodes: {:#?}", nodes);

                if !(spec.type_annotations.is_empty()) {
                    match type_check(spec.clone()) {
                        Ok(s) => {
                            log::info!("Type checked specification: {:#?}", s);
                            Analysis {
                                spec: Some(spec.clone()),
                                typed: Some(s.clone()),
                                diags: vec![],
                                spanned_nodes: nodes.clone(),
                            };
                        }
                        Err(errs) => {
                            log::error!("Type checking errors: {:#?}", errs);

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

    let result = re.replace_all(&msg, |caps: &regex::Captures| {
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
