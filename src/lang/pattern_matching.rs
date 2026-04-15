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

        SExpr::MGet(e, _) | SExpr::MRemove(e, _) | SExpr::MHasKey(e, _) | SExpr::SIndex(e, _)=> {
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

        SExpr::Dynamic(e, _) | SExpr::RestrictedDynamic(e, _, _) | 
        SExpr::Defer(e, _, _) => {
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
            .filter(|spanned| offset >= spanned.span.start && offset < spanned.span.end)
            .min_by_key(|spanned| spanned.span.end - spanned.span.start)
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
