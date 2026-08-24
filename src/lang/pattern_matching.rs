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

use std::cmp::Reverse;

use contiguous_tree::TreeCursorExt;
use trustworthiness_checker::lang::dsrv::{
    ast::{DsrvSpecification, Expr, ExprRef, ExprView},
    span::Span,
};

/// The small amount of AST information needed by the language server.
///
/// The checker used to expose a recursive `SExpr` tree. The current checker
/// exposes arena-backed `ExprRef` cursors instead, so the LSP keeps a compact
/// snapshot for offset lookup and hover without retaining references into the
/// specification's arena.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SExpr {
    If,
    SIndex,
    Val(Literal),
    BinOp,
    Var(trustworthiness_checker::VarName),
    Dynamic,
    Defer,
    Update,
    Default,
    IsDefined,
    When,
    Latch,
    Init,
    Not,
    Neg,
    Lambda,
    Apply,
    Fix,
    Partial,
    List,
    Tuple,
    LIndex,
    LAppend,
    LConcat,
    LHead,
    LTail,
    LLen,
    LMap,
    LFilter,
    LFold,
    Map,
    Struct,
    ObjectLiteral,
    MGet,
    SGet,
    MInsert,
    MRemove,
    MHasKey,
    Sin,
    Cos,
    Tan,
    Abs,
    MonitoredAt,
    Dist,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Bool(bool),
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpannedExpr {
    pub node: SExpr,
    pub span: Span,
}

fn snapshot(expr: ExprRef<'_>) -> SpannedExpr {
    SpannedExpr {
        node: snapshot_kind(expr),
        span: expr.span(),
    }
}

fn snapshot_kind(expr: ExprRef<'_>) -> SExpr {
    match expr.view() {
        ExprView::If(..) => SExpr::If,
        ExprView::SIndex(..) => SExpr::SIndex,
        ExprView::Val(value) => SExpr::Val(snapshot_literal(value)),
        ExprView::BinOp(..) => SExpr::BinOp,
        ExprView::Var(variable) => SExpr::Var(variable.clone()),
        ExprView::Dynamic(..) => SExpr::Dynamic,
        ExprView::Defer(..) => SExpr::Defer,
        ExprView::Update(..) => SExpr::Update,
        ExprView::Default(..) => SExpr::Default,
        ExprView::IsDefined(..) => SExpr::IsDefined,
        ExprView::When(..) => SExpr::When,
        ExprView::Latch(..) => SExpr::Latch,
        ExprView::Init(..) => SExpr::Init,
        ExprView::Not(..) => SExpr::Not,
        ExprView::Neg(..) => SExpr::Neg,
        ExprView::Lambda(..) => SExpr::Lambda,
        ExprView::Apply(..) => SExpr::Apply,
        ExprView::Fix(..) => SExpr::Fix,
        ExprView::Partial(..) => SExpr::Partial,
        ExprView::List(..) => SExpr::List,
        ExprView::Tuple(..) => SExpr::Tuple,
        ExprView::LIndex(..) => SExpr::LIndex,
        ExprView::LAppend(..) => SExpr::LAppend,
        ExprView::LConcat(..) => SExpr::LConcat,
        ExprView::LHead(..) => SExpr::LHead,
        ExprView::LTail(..) => SExpr::LTail,
        ExprView::LLen(..) => SExpr::LLen,
        ExprView::LMap(..) => SExpr::LMap,
        ExprView::LFilter(..) => SExpr::LFilter,
        ExprView::LFold(..) => SExpr::LFold,
        ExprView::Map(..) => SExpr::Map,
        ExprView::Struct(..) => SExpr::Struct,
        ExprView::ObjectLiteral(..) => SExpr::ObjectLiteral,
        ExprView::MGet(..) => SExpr::MGet,
        ExprView::SGet(..) => SExpr::SGet,
        ExprView::MInsert(..) => SExpr::MInsert,
        ExprView::MRemove(..) => SExpr::MRemove,
        ExprView::MHasKey(..) => SExpr::MHasKey,
        ExprView::Sin(..) => SExpr::Sin,
        ExprView::Cos(..) => SExpr::Cos,
        ExprView::Tan(..) => SExpr::Tan,
        ExprView::Abs(..) => SExpr::Abs,
        ExprView::MonitoredAt(..) => SExpr::MonitoredAt,
        ExprView::Dist(..) => SExpr::Dist,
    }
}

fn snapshot_literal(value: &trustworthiness_checker::Value) -> Literal {
    match value {
        trustworthiness_checker::Value::Bool(value) => Literal::Bool(*value),
        _ => Literal::Other,
    }
}

/// Extract a source-ordered, flat snapshot of one expression tree.
pub fn extract_nodes(spanned: &Expr, results: &mut Vec<SpannedExpr>) {
    let mut pending = vec![spanned.as_ref()];
    while let Some(expr) = pending.pop() {
        results.push(snapshot(expr));
        pending.extend(expr.children().rev());
    }
}

/// Extract declaration and expression nodes from a parsed specification.
///
/// Declaration spans are recovered from the source tokens because the current
/// public `DsrvSpecification` API intentionally exposes expression roots but not
/// parser-local declaration records. Expression nodes come directly from the
/// checker-owned forest and retain the parser's exact spans.
pub fn extract_from_stmts(spec: &DsrvSpecification, source: &str, results: &mut Vec<SpannedExpr>) {
    let tokens = crate::lang::syntax::lexer::tokenize(source);
    for (index, token) in tokens.iter().enumerate() {
        let is_declaration = matches!(
            token.token,
            crate::lang::syntax::lexer::Token::In
                | crate::lang::syntax::lexer::Token::Out
                | crate::lang::syntax::lexer::Token::Aux
                | crate::lang::syntax::lexer::Token::Var
        );
        if is_declaration {
            if let Some(name) = tokens
                .get(index + 1)
                .filter(|next| next.token == crate::lang::syntax::lexer::Token::Identifier)
            {
                results.push(SpannedExpr {
                    node: SExpr::Var(name.content.clone().into()),
                    span: Span {
                        start: token.span.start as u32,
                        end: name.span.end as u32,
                    },
                });
            }
        } else if token.token == crate::lang::syntax::lexer::Token::Identifier
            && tokens
                .get(index + 1)
                .is_some_and(|next| next.token == crate::lang::syntax::lexer::Token::Eq)
        {
            results.push(SpannedExpr {
                node: SExpr::Var(token.content.clone().into()),
                span: Span {
                    start: token.span.start as u32,
                    end: token.span.end as u32,
                },
            });
        }
    }

    results.extend(spec.nodes().map(snapshot));
    results.sort_by_key(|node| (node.span.start, Reverse(node.span.end)));
}

// Helper function to find the smallest node at a given offset in the analysis.
impl Analysis {
    pub fn node_at_offset(&self, offset: u32) -> Option<&SpannedExpr> {
        self.spanned_nodes
            .iter()
            .filter(|spanned| offset >= spanned.span.start && offset <= spanned.span.end)
            .min_by_key(|spanned| spanned.span.end - spanned.span.start)
    }
}

use crate::lang::analyser::Analysis;

#[cfg(test)]
mod test {
    use macro_rules_attribute::apply;
    use trustworthiness_checker::async_test;

    use crate::fixtures;

    use super::*;

    #[test]
    fn test_extract_nodes_simple() {
        let expression = fixtures::input_ast_simple();
        let mut nodes = Vec::new();
        extract_nodes(&expression, &mut nodes);

        assert_eq!(nodes.len(), 3);
        assert!(matches!(nodes[0].node, SExpr::BinOp));
        assert!(matches!(nodes[2].node, SExpr::Val(_)));
    }

    #[test]
    fn test_extract_nodes_complex() {
        let expression = fixtures::input_ast_long();
        let mut nodes = Vec::new();
        extract_nodes(&expression, &mut nodes);

        assert_eq!(nodes.len(), 6);
        assert!(matches!(nodes[0].node, SExpr::If));
        assert!(matches!(nodes[2].node, SExpr::Default));
        assert!(matches!(nodes[5].node, SExpr::Val(_)));
    }

    #[test]
    fn test_extract_from_stmts() {
        let source = fixtures::input_untyped_valid_simple();
        let spec: DsrvSpecification = source.parse().unwrap();
        let mut results = Vec::new();
        extract_from_stmts(&spec, source, &mut results);

        assert_eq!(results.len(), 7);
        assert!(matches!(
            results[0].node,
            SExpr::Var(ref name) if name.to_string() == "x"
        ));
        assert!(matches!(
            results[1].node,
            SExpr::Var(ref name) if name.to_string() == "y"
        ));
        assert!(matches!(
            results[2].node,
            SExpr::Var(ref name) if name.to_string() == "z"
        ));
        assert!(matches!(results[3].node, SExpr::Var(ref name) if name == &"z".into()));
    }

    #[apply(async_test)]
    async fn test_node_at_offset() {
        let spanned_nodes = fixtures::input_spanned_nodes_simple();

        let analysis = Analysis {
            spec: None,
            typed: None,
            diags: vec![],
            spanned_nodes,
        };

        let node = analysis.node_at_offset(2).unwrap();
        assert!(matches!(
            node.node,
            SExpr::Var(ref name) if name.to_string() == "x"
        ));

        let node = analysis.node_at_offset(7).unwrap();
        assert!(matches!(
            node.node,
            SExpr::Var(ref name) if name.to_string() == "y"
        ));

        assert!(analysis.node_at_offset(18).is_none());

        let node = analysis.node_at_offset(4).unwrap();
        assert!(matches!(
            node.node,
            SExpr::Var(ref name) if name.to_string() == "x"
        ));
    }
}
