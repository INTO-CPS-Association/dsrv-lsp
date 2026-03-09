use std::collections::BTreeMap;

use ecow::EcoString;
use ecow::EcoVec;
use trustworthiness_checker::Value;
use trustworthiness_checker::VarName;
use trustworthiness_checker::lang::dynamic_lola::ast::*;
use trustworthiness_checker::lang::dynamic_lola::span::*;

pub trait Visitor {
   // no children, just a leaf node
    fn visit_leaf(&mut self, expr: &SpannedExpr, span: &Span);

    // One child, covers most of the math stuff (Not, Sin, Cos, Tan, Abs) and LHead, LTail, LLen, IsDifined, When, Var, Val.
    fn visit_unary(&mut self, child: &SpannedExpr, op_type: &str, span: &Span);
    // Two children, covers BinOP, Update, Default, Latch, Init, LIndex, LAppend, LConcat, 
    fn visit_binary(&mut self, left: &SpannedExpr, right: &SpannedExpr, op_type: &str, span: &Span);

    // Special
    fn visit_if(&mut self, cond: &SpannedExpr, then_br: &SpannedExpr, else_br: &SpannedExpr, span: &Span);
    fn visit_list(&mut self, items: &EcoVec<SpannedExpr>, span: &Span);
    fn visit_map(&mut self, entries: BTreeMap<EcoString, SpannedExpr>, span: &Span);
    fn visit_sindex(&mut self, expr: &SpannedExpr, index: u64, span: &Span);

  }