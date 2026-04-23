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
use crate::lang::{analyzer::Analysis, syntax::lexer::{self, TokenData}};

#[allow(dead_code)]
pub async fn analyze_spec(input: &str) -> Analysis {
  Analysis::analyze_specification(input).await
}

#[allow(dead_code)]
pub fn tokenize_input(input: &str) -> Vec<TokenData> {
  lexer::tokenize(input)
}



#[allow(dead_code)]
pub fn input_valid_simple() -> &'static str {
  "in x\nin y\nout z\nz = x + y"
}

#[allow(dead_code)]
pub fn input_valid_typed() -> &'static str {
    "in x: Int\nin y: Int\nout z: Int\n\nz = x + y"
}

#[allow(dead_code)]
pub fn input_invalid_simple() -> &'static str {
  "in x\nout z\nz = "
}

#[allow(dead_code)]
pub fn input_invalid_typed() -> &'static str {
    "in x: Int\n in y: Str\nout z: Int\nz = x + y"
}

#[allow(dead_code)]
pub fn input_semantic_undeclared_var() -> &'static str {
  "in x: Int\nout z: Int\nz = x + y"
}

#[allow(dead_code)]
pub fn input_empty() -> &'static str {
  ""
}
