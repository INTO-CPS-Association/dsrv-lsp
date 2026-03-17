use trustworthiness_checker::{
    SExpr,
    lang::dynamic_lola::{ast::SpannedExpr},
};

use crate::lang::analyzer::Analysis;

pub fn extract_nodes(spanned: &SpannedExpr, results: &mut Vec<SpannedExpr>) {
    results.push(spanned.clone());

    match &spanned.node {
        SExpr::LIndex(e1, e2) | SExpr::LAppend(e1, e2) | SExpr::LConcat(e1, e2) => {
            extract_nodes(e1, results);
            extract_nodes(e2, results);
        }

        SExpr::List(elements) => {
            for el in elements {
                extract_nodes(el, results);
            }
        }

        SExpr::BinOp(lhs, rhs, _op) => {
            extract_nodes(lhs, results);
            extract_nodes(rhs, results);
        }

        SExpr::Val(_) | SExpr::Var(_) => {}

        _ => {}
    }
}

impl Analysis {
    pub fn node_at_offset(&self, offset: u32) -> Option<&SpannedExpr> {
        self.spanned_nodes
            .iter()
            .filter(|spanned| offset >= spanned.span.start && offset < spanned.span.end)
            .min_by_key(|spanned| spanned.span.end - spanned.span.start)
    }
}
