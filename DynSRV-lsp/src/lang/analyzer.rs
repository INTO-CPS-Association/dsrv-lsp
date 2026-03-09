use crate::utils::*;
use lalrpop_util::ParseError;
use ropey::Rope;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::{
    core::parser::ExprParser,
    dynamic_lola::{
        ast::{LOLASpecification, SpannedExpr},
        lalr::TopDeclsParser,
        lalr_parser::{create_lola_spec, parse_sexpr, parse_str},
        parser::{CombExprParser, lola_specification},
        span::*,
        type_checker::TypedLOLASpecification,
    },
};

#[derive(Clone, Debug)]
pub struct Analysis {
    pub spec: Option<LOLASpecification>, // The parsed specification, if parsing was successful
    pub typed: Option<TypedLOLASpecification>, //For future use, when type checker is implemented
    pub diags: Vec<Diagnostic>,          // Diagnostics from both syntax and semantic analysis
    pub spanned_nodes: Vec<SpannedExpr>, // A vector of all expressions in the spec annotated with their spans
}

impl Analysis {
    // Create Clone function for Analysis struct
    pub async fn analyze_2_point_0(text: &str) -> Analysis {
        let mut s: &str = text;
        // Turn s into &mut &str
        let s = &mut s;

        match TopDeclsParser::new().parse(text) {
            Ok(stmts) => {
                let spec = create_lola_spec(&stmts);

                Analysis {
                    spec: Some(spec.clone()),
                    typed: None,
                    diags: vec![],
                    spanned_nodes: vec![],
                }
            }

            Err(error) => {
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

        // match CombExprParser::raw_parse_error(s) {
        //     // match parse_str(s) {
        //     Ok(spec) => {
        //         log::info!("Parsed spec: {:?}", spec);

        //         let exprs = spec.exprs.clone();
        //         //get spans for each expression in the spec
        //         let span_nodes = exprs
        //             .values()
        //             .map(|e| SpannedExpr {
        //                 span: e.span.clone(),
        //                 node: e.clone().node,
        //             })
        //             .collect();

        //         log::info!("Spanned nodes: {:?}", span_nodes);

        //         Analysis {
        //             spec: Some(spec.clone()),
        //             typed: None,
        //             diags: vec![],
        //             spanned_nodes: span_nodes,
        //         }
        //     }

        //     Err(error) => {
        //         log::error!("Parse error: {:?}", error);
        //         Analysis {
        //             spec: None,
        //             typed: None,
        //             diags: vec![],
        //             spanned_nodes: vec![],
        //         }
        //     }
        // }
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
    // match TopDeclsParser::new().parse(text) {
    //     Ok(stmts) => {
    //         let spec = create_lola_spec(&stmts);
    //         let mut diags = vec![];
    //         let (tokens, token_spans) = tokenize(text, &mut diags);

    //         Analysis {
    //             spec: Some(spec.clone()),
    //             typed: None,
    //             diags: vec![],
    //         }
    //     }

    //     Err(error) => {
    //         // Map the error's byte positions to line and column positions in the text_document immediately.
    //         let error = error.map_location(|byte| byte_to_pos(&Rope::from_str(text), byte));

    //         // Convert the parse error into a diagnostic message with a range indicating where the error occurred in the source code
    //         let diags = match error {
    //             ParseError::InvalidToken { location } => {
    //                 let range =
    //                     Range::new(location.unwrap_or_default(), location.unwrap_or_default());
    //                 Self::create_diag("Invalid Token", range)
    //             }
    //             ParseError::UnrecognizedEof {
    //                 location,
    //                 expected: _,
    //             } => {
    //                 let range = Range {
    //                     start: location.unwrap_or_default(),
    //                     end: location.unwrap_or_default(),
    //                 };

    //                 Self::create_diag("Syntax error: Unexpected EOF", range)
    //             }

    //             ParseError::UnrecognizedToken { token, expected: _ } => {
    //                 let (start, _tok, end) = token;
    //                 Self::create_diag(
    //                     "Syntax error: Unrecognized token",
    //                     Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
    //                 )
    //             }
    //             ParseError::ExtraToken { token } => {
    //                 let (start, _tok, end) = token;

    //                 Self::create_diag(
    //                     "Syntax error: Extra token:",
    //                     Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
    //                 )
    //             }

    //             ParseError::User { error } => {
    //                 let p = Position::new(1, 1);
    //                 Self::create_diag(&format!("User error: {:?}", error), Range::new(p, p))
    //             }
    //         };

    //         // Return the analysis result with the diagnostic message
    //         Analysis {
    //             spec: None,
    //             typed: None,
    //             diags: vec![diags],
    //         }
    //     }
    // }
}

//Helper function to create a diagnostic with a given message and range

//Not needed anymore after the ast files was updated to include spans
// fn wrap_with_spans<'a>(
//     spec: &'a LOLASpecification,
//     token_spans: &[(Token, std::ops::Range<usize>)],
// ) -> Vec<(std::ops::Range<usize>, SExpr)> {
//     let mut cursor = 0;
//     fn traverse<'a>(
//         expr: &'a SExpr,
//         token_spans: &[(Token, std::ops::Range<usize>)],
//         cursor: &mut usize,
//     ) -> (std::ops::Range<usize>, SExpr) {
//         // Heuristic
//         let start_pos = token_spans.get(*cursor).map(|(_, r)| r.start).unwrap_or(0);

//         // For each expression type, we determine how many tokens it consumes and traverse its children accordingly, updating the cursor to reflect the current position in the token stream. We then create a Spanned node for the expression using the start position of the first token it consumes and the end position of the last token it consumes.
//         match expr {
//             SExpr::Val(_) | SExpr::Var(_) | SExpr::MonitoredAt(_, _) | SExpr::Dist(_, _) => {
//                 let end_pos = token_spans
//                     .get(*cursor)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 *cursor += 1;
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::Not(e)
//             | SExpr::Sin(e)
//             | SExpr::Cos(e)
//             | SExpr::Tan(e)
//             | SExpr::Abs(e)
//             | SExpr::IsDefined(e)
//             | SExpr::When(e)
//             | SExpr::LHead(e)
//             | SExpr::LTail(e)
//             | SExpr::LLen(e) => {
//                 *cursor += 1;
//                 traverse(e, token_spans, cursor); // Traverse the child expression
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::BinOp(e1, e2, _)
//             | SExpr::LConcat(e1, e2)
//             | SExpr::Update(e1, e2)
//             | SExpr::Default(e1, e2)
//             | SExpr::Init(e1, e2)
//             | SExpr::Latch(e1, e2)
//             | SExpr::LAppend(e1, e2)
//             | SExpr::LIndex(e1, e2) => {
//                 traverse(e1, token_spans, cursor);
//                 *cursor += 1;
//                 traverse(e2, token_spans, cursor);
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::If(e1, e2, e3) => {
//                 *cursor += 1;
//                 traverse(e1, token_spans, cursor);
//                 *cursor += 1;
//                 traverse(e2, token_spans, cursor);
//                 *cursor += 1;
//                 traverse(e3, token_spans, cursor);
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::SIndex(e, _) => {
//                 traverse(e, token_spans, cursor);
//                 *cursor += 1; // "["
//                 *cursor += 1; // index
//                 *cursor += 1; // "]"
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::Dynamic(e, _) | SExpr::RestrictedDynamic(e, _, _) | SExpr::Defer(e, _) => {
//                 *cursor += 1; // "dynamic" or "defer"
//                 traverse(e, token_spans, cursor);
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::List(es) => {
//                 *cursor += 1; // "("
//                 for (i, e) in es.iter().enumerate() {
//                     if i > 0 {
//                         *cursor += 1;
//                     } // ","
//                     traverse(e, token_spans, cursor);
//                 }
//                 *cursor += 1; // ")"
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             SExpr::Map(m) => {
//                 *cursor += 1; // "{"
//                 for (i, (_, v)) in m.iter().enumerate() {
//                     if i > 0 {
//                         *cursor += 1;
//                     } // ","
//                     traverse(v, token_spans, cursor);
//                 }
//                 *cursor += 1; // "}"
//                 let end_pos = token_spans
//                     .get(*cursor - 1)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 (start_pos..end_pos, expr.clone())
//             }
//             _ => {
//                 let end_pos = token_spans
//                     .get(*cursor)
//                     .map(|(_, r)| r.end)
//                     .unwrap_or(start_pos);
//                 *cursor += 1;
//                 (start_pos..end_pos, expr.clone())
//             }
//         }
//     }
//     spec.exprs
//         .values()
//         .map(|e| traverse(e, token_spans, &mut cursor))
//         .collect()

// }

// fn _node_at_offset<'a>(nodes: &'a [Spanned<'a, SExpr>], offset: usize) -> Option<&'a SExpr> {
//     nodes
//         .iter()
//         .filter(|n| n.start <= offset && offset <= n.end)
//         .min_by_key(|n| n.end - n.start)
//         .map(|n| n.node)
// }

// -----    Lalr parser version ----
//     match lalr_parser::parse_str(text) {
//         Ok(spec) => {
//             log::info!("Parsed spec: {:?}", spec);

//             Analysis {
//                 spec: Some(spec.clone()),
//                 typed: None,
//                 diags: vec![],
//                 spanned_nodes: vec![]
//             }
//         }
//         Err(error) => {
//             log::error!("Parse error: {:?}", error);
//             Analysis {
//                 spec: None,
//                 typed: None,
//                 diags: vec![],
//                 spanned_nodes: vec![],
//             }
//     }

// }
