pub mod functions;
pub mod keywords_and_types;
pub mod list_and_map;
pub mod math;

use once_cell::sync::Lazy;

// use tower_lsp::lsp_types::*;
use tower_lsp_server::ls_types::{CompletionItemKind, InsertTextFormat};


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
