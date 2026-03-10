use std::collections::BTreeMap;

use ecow::EcoString;
use ecow::EcoVec;
use serde_json::value::Index;
use trustworthiness_checker::Value;
use trustworthiness_checker::VarName;
use trustworthiness_checker::core::StreamTypeAscription;
use trustworthiness_checker::distributed::distribution_graphs::NodeName;
use trustworthiness_checker::lang::dynamic_lola::ast::*;
use trustworthiness_checker::lang::dynamic_lola::span::*;

pub enum RelationType {
    MonitoredAt,
    Dist,
}

pub trait Visitor {
    // no children, just a leaf node
    fn visit_leaf(&mut self, expr: &SExpr, span: &Span);

    // One child, covers most of the math stuff (Not, Sin, Cos, Tan, Abs) and LHead, LTail, LLen, IsDifined, When.
    fn visit_unary(&mut self, child: &SpannedExpr, op_type: &str, span: &Span);

    // Two children, covers BinOP, Update, Default, Latch, Init, LIndex, LAppend, LConcat,
    fn visit_binary(&mut self, left: &SpannedExpr, right: &SpannedExpr, op_type: &str, span: &Span);

    // Special
    fn visit_if(
        &mut self,
        cond: &SpannedExpr,
        then_br: &SpannedExpr,
        else_br: &SpannedExpr,
        span: &Span,
    );
    fn visit_list(&mut self, items: &EcoVec<SpannedExpr>, span: &Span);
    fn visit_sindex(&mut self, expr: &SpannedExpr, index: &u64, span: &Span);

    fn visit_map(&mut self, entries: &BTreeMap<EcoString, SpannedExpr>, span: &Span);
    fn visit_mapfn(&mut self, expr: &SpannedExpr, key: &EcoString, span: &Span);
    fn visit_map_insert(
        &mut self,
        expr: &SpannedExpr,
        key: &EcoString,
        val: &SpannedExpr,
        span: &Span,
    );

    fn visit_dynamic(&mut self, expr: &SpannedExpr, t: &StreamTypeAscription, span: &Span);

    fn visit_defer(
        &mut self,
        expr: &SpannedExpr,
        t: &StreamTypeAscription,
        restric: &EcoVec<VarName>,
        span: &Span,
    );

    fn visit_dist(&mut self, l: &VarOrNodeName, r: &VarOrNodeName, span: &Span);
    fn visit_monit(&mut self, l: &VarName, r: &NodeName, span: &Span);
}

// Walker that walks the entire AST, calling the appropriate visit method for each node. This is a depth first traversal.
pub fn walk_expr<V: Visitor>(v: &mut V, expr: &SpannedExpr) {
    match &expr.node {
        SExpr::BinOp(l, r, op) => {
            v.visit_binary(l, r, &format!("{:?}", op), &expr.span);
            walk_expr(v, l);
            walk_expr(v, r);
        }
        SExpr::Default(l, r)
        | SExpr::Latch(l, r)
        | SExpr::LIndex(l, r)
        | SExpr::Init(l, r)
        | SExpr::LAppend(l, r)
        | SExpr::LConcat(l, r)
        | SExpr::Update(l, r) => {
            v.visit_binary(l, r, "Structural", &expr.span);
            walk_expr(v, l);
            walk_expr(v, r);
        }
        SExpr::Abs(c)
        | SExpr::Cos(c)
        | SExpr::Sin(c)
        | SExpr::Tan(c)
        | SExpr::LHead(c)
        | SExpr::LTail(c)
        | SExpr::LLen(c)
        | SExpr::IsDefined(c)
        | SExpr::When(c)
        | SExpr::Not(c) => {
            v.visit_unary(c, "UnaryOp", &expr.span);
            walk_expr(v, c);
        }

        SExpr::Dynamic(c, t) => {
            v.visit_dynamic(c, t, &expr.span);
            walk_expr(v, c);
        }

        SExpr::Defer(c, t, vec) | SExpr::RestrictedDynamic(c, t, vec) => {
            v.visit_defer(c, t, vec, &expr.span);
            walk_expr(v, c);
        }

        SExpr::If(i, t, e) => {
            v.visit_if(i, t, e, &expr.span);
            walk_expr(v, i);
            walk_expr(v, t);
            walk_expr(v, e);
        }

        SExpr::Val(_) | SExpr::Var(_) => {
            v.visit_leaf(&expr.node, &expr.span);
        }

        SExpr::List(i) => {
            v.visit_list(i, &expr.span);
            for item in i {
                walk_expr(v, item);
            }
        }

        SExpr::Map(b) => {
            v.visit_map(b, &expr.span);
            for item in b.values() {
                walk_expr(v, item);
            }
        }

        SExpr::SIndex(c, i) => {
            v.visit_sindex(c, i, &expr.span);
            walk_expr(v, c);
        }

        SExpr::MGet(c, k) | SExpr::MRemove(c, k) | SExpr::MHasKey(c, k) => {
            v.visit_mapfn(c, k, &expr.span);
            walk_expr(v, c);
        }

        SExpr::MInsert(c, k, val) => {
            v.visit_map_insert(c, k, val, &expr.span);
            walk_expr(v, c);
            walk_expr(v, val);
        }

        SExpr::Dist(l, r) => {
            v.visit_dist(l, r, &expr.span);
        }

        SExpr::MonitoredAt(l, r) => {
            v.visit_monit(l, r, &expr.span);
        }
    }
}

pub struct NodeInfo {
    pub span: Span,
    pub kind: String,
    pub name: Option<String>,
}

pub struct NodeCollector {
    pub nodes: Vec<NodeInfo>,
}

impl NodeCollector {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl Visitor for NodeCollector {
    fn visit_leaf(&mut self, expr: &SExpr, span: &Span) {
let (kind, name) = match expr {
            SExpr::Val(v) => ("Value".to_string(), Some(format!("{:?}", v))),
            SExpr::Var(v) => ("Variable".to_string(), Some(v.to_string())),
            _ => ("Leaf".to_string(), None),
        };

        self.nodes.push(NodeInfo {
            span: span.clone(),
            kind,
            name,
        });
    }

    fn visit_binary(
        &mut self,
        _left: &SpannedExpr,
        _right: &SpannedExpr,
        op_type: &str,
        span: &Span,
    ) {
        self.nodes.push(NodeInfo {
            span: span.clone(),
            kind: format!("Binary({})", op_type),
            name: None,
        });
    }

    fn visit_unary(&mut self, child: &SpannedExpr, op_type: &str, span: &Span) {
        self.nodes.push(NodeInfo {
            span: span.clone(),
            kind: format!("Unary({})", op_type),
            name: None,
        })
    }

    fn visit_if(
            &mut self,
            cond: &SpannedExpr,
            then_br: &SpannedExpr,
            else_br: &SpannedExpr,
            span: &Span,
        ) {
        self.nodes.push(NodeInfo {
            span: span.clone(),
            kind: "If".to_string(),
            name: None
        });
    }

    fn visit_list(&mut self, items: &EcoVec<SpannedExpr>, span: &Span) {
        self.nodes.push(NodeInfo { span: span.clone(), kind: "Index".to_string(), name: Some() });
    }


}
