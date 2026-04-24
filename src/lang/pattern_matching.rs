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

use crate::lang::analyzer::Analysis;
use trustworthiness_checker::{
    SExpr,
    lang::dsrv::{
        ast::{STopDecl, SpannedExpr},
        span::Span,
    },
};

// Recursively extract all nodes from an expression tree and store them in a flat vector.
pub fn extract_nodes(spanned: &SpannedExpr, results: &mut Vec<SpannedExpr>) {
    // Visit and push the current node onto the results vector.
    results.push(spanned.clone());

    // Recursively extract nodes from the expression tree.
    match &spanned.node {
        // Nodes with similar structure can be grouped together for cleaner code.
        #[rustfmt::skip] // Disable rustfmt for this match arm to maintain the grouping and readability.
        SExpr::LIndex(e1, e2) | SExpr::LAppend(e1, e2) | SExpr::LConcat(e1, e2) | SExpr::Latch(e1, e2) | 
        SExpr::Update(e1, e2) | SExpr::Default(e1, e2) | SExpr::Init(e1, e2) => {
            extract_nodes(e1, results);
            extract_nodes(e2, results);
        }

        #[rustfmt::skip] // Disable rustfmt for this match arm to maintain the grouping and readability.
        SExpr::LTail(e) | SExpr::LLen(e) | SExpr::Abs(e) | SExpr::Cos(e) | SExpr::IsDefined(e) | 
        SExpr::LHead(e) | SExpr::When(e) | SExpr::Not(e) | SExpr::Sin(e) | SExpr::Tan(e)  => {
            extract_nodes(e, results);
        }

        SExpr::List(elements) => {
            for el in elements {
                extract_nodes(el, results);
            }
        }

        SExpr::Map(kv_pair) => {
            for (_, v) in kv_pair {
                extract_nodes(v, results);
            }
        }

        SExpr::MGet(e, _) | SExpr::MRemove(e, _) | SExpr::MHasKey(e, _) | SExpr::SIndex(e, _) => {
            extract_nodes(e, results);
        }

        SExpr::MInsert(m, _, v) => {
            extract_nodes(m, results);
            extract_nodes(v, results);
        }

        SExpr::BinOp(lhs, rhs, _op) => {
            extract_nodes(lhs, results);
            extract_nodes(rhs, results);
        }

        SExpr::Dynamic(e, _) | SExpr::RestrictedDynamic(e, _, _) | SExpr::Defer(e, _, _) => {
            extract_nodes(e, results);
        }

        SExpr::If(e1, e2, e3) => {
            extract_nodes(e1, results);
            extract_nodes(e2, results);
            extract_nodes(e3, results);
        }

        //Base cases: no further nodes to extract.
        SExpr::Val(_) | SExpr::Var(_) => {}

        _ => {}
    }
}

// Extract variable names from top-level declarations and their assigned expressions.
pub fn extract_from_stmts(stmts: &[STopDecl], results: &mut Vec<SpannedExpr>) {
    for stmt in stmts {
        match stmt {
            STopDecl::Input(var_name, _type, span)
            | STopDecl::Output(var_name, _type, span)
            | STopDecl::Aux(var_name, _type, span) => {
                results.push(SpannedExpr {
                    node: SExpr::Var(var_name.clone()),
                    span: span.clone(),
                });
            }
            STopDecl::Assignment(var_name, expr, span) => {
                let var_len = var_name.to_string().len() as u32;
                results.push(SpannedExpr {
                    node: SExpr::Var(var_name.clone()),
                    span: Span {
                        start: span.start,
                        end: span.start + var_len,
                    },
                });
                extract_nodes(expr, results);
            }
        }
    }
}

// Helper function to find the smallest node at a given offset in the analysis.
impl Analysis {
    pub fn node_at_offset(&self, offset: u32) -> Option<&SpannedExpr> {
        self.spanned_nodes
            .iter()
            .filter(|spanned| offset >= spanned.span.start && offset <= spanned.span.end)
            .min_by_key(|spanned| spanned.span.end - spanned.span.start) // Find the smallest node that contains the offset by finding the one with the smallest span
    }

    // Not used at this time but might later on
    // pub fn parent_of_node(&self, child_span: Span) -> Option<&SpannedExpr> {
    //     self.spanned_nodes
    //         .iter()
    //         .filter(|p| {
    //             p.span.start <= child_span.start
    //                 && p.span.end >= child_span.end
    //                 && p.span != child_span
    //         })
    //         .min_by_key(|p| p.span.end - p.span.start)
    // }
}

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    use crate::fixtures;

    use super::*;

    #[test]
    fn test_extract_nodes_simple() {
        let spanned = fixtures::input_ast_simple();
        let mut nodes = Vec::new();
        extract_nodes(&spanned, &mut nodes);

        // println!("Spanned: {:#?}", spanned);
        println!("Extracted Nodes: {:#?}", nodes);

        assert!(nodes.len() == 3);
        assert!(
            matches!(nodes[0].node, SExpr::BinOp(_, _, _)),
            "First node should be the BinOp"
        );
        assert!(
            matches!(nodes[2].node, SExpr::Val(_)),
            "Third node should be a Val"
        );
    }

    #[test]
    fn test_extract_nodes_complex() {
        let spanned = fixtures::input_ast_long();
        let mut nodes = Vec::new();
        extract_nodes(&spanned, &mut nodes);

        println!("Extracted Nodes: {:#?}", nodes);
        assert!(nodes.len() == 6, "Expected 6 node, got {}", nodes.len());
        assert!(
            matches!(nodes[0].node, SExpr::If(_, _, _)),
            "First node should be the If"
        );
        assert!(
            matches!(nodes[2].node, SExpr::Default(_, _)),
            "Third node should be the Default"
        );
        assert!(
            matches!(nodes[5].node, SExpr::Val(_)),
            "Sixth node should be a Val"
        );
    }

    #[test]
    fn test_extract_from_stmts() {
        let stmts = fixtures::input_stmts_simple();
        
        let mut results = Vec::new();
        extract_from_stmts(&stmts, &mut results);

        println!("Extracted from statements: {:#?}", results);

        assert!(
            results.len() == 7,
            "Expected 6 nodes, got {}",
            results.len()
        );
        assert!(
            matches!(results[0].node, SExpr::Var(ref name) if name.to_string() == "x"),
            "First node should be variable 'x'"
        );
        assert!(
            matches!(results[1].node, SExpr::Var(ref name) if name.to_string() == "y"),
            "Second node should be variable 'y'"
        );
        assert!(
            matches!(results[2].node, SExpr::Var(ref name) if name.to_string() == "z"),
            "Third node should be variable 'z'"
        );

        // Test if the span of the variable 'z' in the assignment is cut of correctly to only include the variable name and not the whole expression
        assert_eq!(
            results[3].span.end, 18,
            "Expected span end to be 18 for variable 'z' in assignment"
        );
    }

    #[apply(async_test)]
    async fn test_node_at_offset() {
        let spanned_nodes = fixtures::input_spanned_nodes_simple();
        
        // Make an analysis with the spanned nodes
        let analysis = Analysis {
            spec: None,
            typed: None,
            diags: vec![],
            spanned_nodes,
        };

        // Test offsets that should return a node
        let node = analysis.node_at_offset(2).unwrap();
        assert!(
            matches!(node.node, SExpr::Var(ref name) if name.to_string() == "x"),
            "Offset 2 should return variable 'x'"
        );

        let node = analysis.node_at_offset(7).unwrap();
        assert!(
            matches!(node.node, SExpr::Var(ref name) if name.to_string() == "y"),
            "Offset 7 should return variable 'y'"
        );

        let node = analysis.node_at_offset(18);
        assert!(
            node.is_none(),
            "Offset 18 should return None as it is outside all spans"
        );

        // Test offsets that are on the boundary of spans
        let node = analysis.node_at_offset(4).unwrap();
        assert!(
            matches!(node.node, SExpr::Var(ref name) if name.to_string() == "x"),
            "Offset 4 should return variable 'x'"
        );
    }
}
