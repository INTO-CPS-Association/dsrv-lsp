use serde::Deserialize;
use tower_lsp::lsp_types::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinEntry {
    pub name: String,
    pub kind: String,
    pub context: Vec<String>,
    pub insert: Option<String>,
    pub insert_format: Option<String>,
    pub signature: Option<String>,
    pub docs: Option<String>,
}

pub fn load_built_ins() -> Vec<BuiltinEntry> {
    let json = include_str!("./builtin_catalog.json");
    serde_json::from_str(json).expect("Failed to parse built-in catalog")
}

// Converts string representation of completion item kind to LSP's CompletionItemKind
fn completion_kind_from_str(kind: &str) -> Option<CompletionItemKind> {
    match kind {
        "function" => Some(CompletionItemKind::FUNCTION),
        "keyword" => Some(CompletionItemKind::KEYWORD),
        "constant" => Some(CompletionItemKind::CONSTANT),
        "operator" => Some(CompletionItemKind::OPERATOR),
        "type" => Some(CompletionItemKind::UNIT),
        _ => None,
    }
}

fn insert_format_from_str(fmt: &str) -> Option<InsertTextFormat> {
    match fmt {
        "snippet" => Some(InsertTextFormat::SNIPPET),
        "plain" => Some(InsertTextFormat::PLAIN_TEXT),
        _ => None,
    }
}

pub fn json_to_completionItem(builtins: &[BuiltinEntry]) -> Vec<CompletionItem> {
    builtins
        .iter()
        .map(|b| CompletionItem {
            label: b.name.clone(),
            kind: completion_kind_from_str(&b.kind),
            detail: b.signature.clone(),
            documentation: b.docs.as_ref().map(|d| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: d.clone(),
                })
            }),
            insert_text: b.insert.clone().or_else(|| Some(b.name.clone())),
            insert_text_format: b.insert_format.as_deref().and_then(insert_format_from_str),
            ..Default::default()
        })
        .collect()
}
