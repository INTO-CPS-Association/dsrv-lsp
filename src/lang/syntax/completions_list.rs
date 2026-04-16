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

pub mod functions;
pub mod keywords_and_types;
pub mod list_and_map;
pub mod math;

use once_cell::sync::Lazy;

// use tower_lsp::lsp_types::*;
use tower_lsp_server::ls_types::{CompletionItemKind, InsertTextFormat};


// Struct to hold the information about a built-in function or keyword for autocompletion
#[derive(Debug, Clone)]
pub struct DsrvBuiltIn {
    pub label: &'static str,
    pub kind: CompletionItemKind,
    pub trigger_context: &'static [&'static str],
    pub insert_text: &'static str,
    pub insert_text_format: InsertTextFormat,
    pub detail: &'static str,
    pub documentation: &'static str,
}

// Global registry of built-in functions and keywords for autocompletion
pub static BUILTIN_REGISTRY: Lazy<Vec<DsrvBuiltIn>> = Lazy::new(|| {
    let mut v = Vec::new();
    v.extend_from_slice(keywords_and_types::CORE);
    v.extend_from_slice(keywords_and_types::TYPES);
    v.extend_from_slice(list_and_map::LIST);
    v.extend_from_slice(list_and_map::MAP);
    v.extend_from_slice(math::MATH);
    v.extend_from_slice(functions::FUNCTIONS);
    v.extend_from_slice(functions::DIST_FUNCTIONS);

    v
});

pub fn get_builtin_by_label(label_name: &str) -> Option<&DsrvBuiltIn> {
    BUILTIN_REGISTRY
        .iter()
        .find(|item| item.label == label_name)
}
