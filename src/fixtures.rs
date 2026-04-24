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
use crate::lang::{
    analyzer::Analysis,
    syntax::lexer::{self, TokenData},
};

#[allow(dead_code)]
pub async fn analyze_spec(input: &str) -> Analysis {
    Analysis::analyze_specification(input).await
}

#[allow(dead_code)]
pub fn tokenize_input(input: &str) -> Vec<TokenData> {
    lexer::tokenize(input)
}

#[allow(dead_code)]
pub fn input_untyped_valid_simple() -> &'static str {
    "in x\nin y\nout z\nz = x + y"
}

#[allow(dead_code)]
pub fn input_typed_valid_simple() -> &'static str {
    "in x: Int\nin y: Int\nout z: Int\n\nz = x + y"
}

#[allow(dead_code)]
pub fn input_untyped_invalid_simple() -> &'static str {
    "in x\nout z\nz = "
}

#[allow(dead_code)]
pub fn input_typed_invalid_simple() -> &'static str {
    "in x: Int\n in y: Str\nout z: Int\nz = x + y"
}

#[allow(dead_code)]
pub fn input_semantic_undeclared_var() -> &'static str {
    "in x: Int\nout z: Int\nz = x + y"
}

pub fn input_empty() -> &'static str {
    ""
}

#[allow(dead_code)]
pub fn input_untyped_simple_with_comments() -> &'static str {
    "in x\nin y // test - comments\nout z\nz = x + y //test comment"
}

// Code snippet from the robosapiens-trustworthiness-checker by the Into-CPS organization under the GPL licence
#[allow(dead_code)]
pub fn input_untyped_complex_with_comments() -> &'static str {
    "in rawHuman\nin useNucOne\nin reqUseNucOne\nout hasError\nout hadError\nout cameraSwap\nout swapRequest\nout swapRequestSticky\nout safeSwap\naux swapRequestHelper\naux numberOfHumans\n\nnumberOfHumans = Map.get(rawHuman, \"number_of_humans\")\nswapRequest = !(reqUseNucOne == default(reqUseNucOne[1], true))\nswapRequestHelper = if cameraSwap then swapRequest else default(swapRequestHelper[1], swapRequest)\nswapRequestSticky = default(swapRequestHelper[1], false) || swapRequest\n// Denotes if we swapped cameras.\n// Default expr encapsulates that we expect the system to use left camera when booted\n// init is to make sure this var is never NoVal. If it is NoVal it means we haven't swapped.\ncameraSwap = init(!(default(useNucOne[1], true) == useNucOne), false)\nsafeSwap = cameraSwap => numberOfHumans == 0 // cameraSwap implies no humans - either we are not swapping or we are swapping when no people are present\n// Implication: swapRequestSticky implies swapping safely and then negated\n// Negated: Because the logic is inverted\nhasError = !(swapRequestSticky => safeSwap)\nhadError = hasError || default(hadError[1], false) // Globally"
}

// Code snippet from the robosapiens-trustworthiness-checker by the Into-CPS organization under the GPL licence
#[allow(dead_code)]
pub fn input_untyped_complex_invalid() -> &'static str {
    "in iMap\nout oMap\nout nestedMap\nout mapGet\nout mapRemove\nout mapInsert\nout mapHasKey\noMap = iMap\nnestedMap = Map(\"a\": iMap, \"b\": iMap)\nmapGet = Map.get(iMap, \"x\")\nmapRemove = Map.remove(iMap, \"x\")\nmapInsert = Map.insert(iMap, \"z\", 42)\nmapHasKey = Map."
}

use trustworthiness_checker::lang::dsrv::{
    ast::{SExpr, STopDecl, SpannedExpr},
    span::Span,
};

#[allow(dead_code)]
pub fn input_ast_simple() -> SpannedExpr {
    SpannedExpr {
        node: SExpr::BinOp(
            Box::new(SpannedExpr {
                node: SExpr::Val(1.into()),
                span: Span { start: 0, end: 1 },
            }),
            Box::new(SpannedExpr {
                node: SExpr::Val(2.into()),
                span: Span { start: 4, end: 5 },
            }),
            "+".into(),
        ),
        span: Span { start: 0, end: 5 },
    }
}

#[allow(dead_code)]
pub fn input_ast_long() -> SpannedExpr {
    SpannedExpr {
        node: SExpr::If(
            Box::new(SpannedExpr {
                node: SExpr::Var("x".into()),
                span: Span { start: 0, end: 1 },
            }),
            Box::new(SpannedExpr {
                node: SExpr::Default(
                    Box::new(SpannedExpr {
                        node: SExpr::Val(1.into()),
                        span: Span { start: 4, end: 5 },
                    }),
                    Box::new(SpannedExpr {
                        node: SExpr::Val(2.into()),
                        span: Span { start: 8, end: 9 },
                    }),
                ),
                span: Span { start: 4, end: 9 },
            }),
            Box::new(SpannedExpr {
                node: SExpr::Val(3.into()),
                span: Span { start: 12, end: 13 },
            }),
        ),
        span: Span { start: 0, end: 13 },
    }
}

#[allow(dead_code)]
pub fn input_stmts_simple() -> Vec<STopDecl> {
    vec![
        STopDecl::Input("x".into(), None, Span { start: 0, end: 4 }),
        STopDecl::Input("y".into(), None, Span { start: 5, end: 9 }),
        STopDecl::Output("z".into(), None, Span { start: 10, end: 15 }),
        STopDecl::Assignment(
            "z".into(),
            SpannedExpr {
                node: SExpr::BinOp(
                    Box::new(SpannedExpr {
                        node: SExpr::Var("x".into()),
                        span: Span { start: 21, end: 22 },
                    }),
                    Box::new(SpannedExpr {
                        node: SExpr::Var("y".into()),
                        span: Span { start: 25, end: 26 },
                    }),
                    "+".into(),
                ),
                span: Span { start: 21, end: 26 },
            },
            Span { start: 17, end: 26 },
        ),
    ]
}

#[allow(dead_code)]
pub fn input_spanned_nodes_simple() -> Vec<SpannedExpr> {
    vec![
        SpannedExpr {
            node: SExpr::Var("x".into()),
            span: Span { start: 0, end: 4 },
        },
        SpannedExpr {
            node: SExpr::Var("y".into()),
            span: Span { start: 5, end: 9 },
        },
        SpannedExpr {
            node: SExpr::Var("z".into()),
            span: Span { start: 10, end: 15 },
        },
    ]
}
