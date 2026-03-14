use tower_lsp::lsp_types::*;

pub struct DsrvBuiltIn {
    pub label: &'static str,
    pub kind: CompletionItemKind,
    pub trigger_context: &'static [&'static str],
    pub insert_text: &'static str,
    pub insert_text_format: InsertTextFormat,
    pub detail: &'static str,
    pub documentation: &'static str,
}

pub static BUILTIN_REGISTRY: &[DsrvBuiltIn] = &[
    DsrvBuiltIn {
        label: "in",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["toplevel"],
        insert_text: "in ${1:label}",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "in <label> [: <Type>]",
        documentation: "Declares an input stream that provides a sequence of event values to the monitor. The label acts as a variable name in the input namespace in(ϕ)",
    },
    DsrvBuiltIn {
        label: "out",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["toplevel"],
        insert_text: "out ${1:label}",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "out <label> [: <Type>]",
        documentation: "Declares an output stream, also known as a verdict stream. It transforms input data into results based on defined stream equations.",
    },
    DsrvBuiltIn {
    label: "aux",
    kind: CompletionItemKind::KEYWORD,
    trigger_context: &["toplevel"],
    insert_text: "aux ${1:label}",
    insert_text_format: InsertTextFormat::SNIPPET,
    detail: "aux <label> [: <Type>]",
    documentation: "Declares an auxiliary internal stream variable used to simplify complex equations and is not exposed as a final output",
    },
    DsrvBuiltIn {
    label: "dynamic",
    kind: CompletionItemKind::FUNCTION,
    trigger_context: &["expr"],
    insert_text: "dynamic(${1:p})",
    insert_text_format: InsertTextFormat::SNIPPET,
    detail: "dynamic(ψ)",
    documentation: "Dynamic property which behaves like the most recent value of ψ or is ⊥ if none has been sent",
    },
];

// const DATA: &str = include_str!("languageBuiltin.json");

// pub fn load_json() -> Vec<LanguageBuiltin> {
//   serde_json::from_str(DATA).expect("Failed to parse JSON file")
// }

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]

// #[derive(Debug, Clone, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct BuiltinEntry {
//     pub name: String,
//     pub kind: String,
//     pub context: Vec<String>,
//     pub insert: Option<String>,
//     pub insert_format: Option<String>,
//     pub signature: Option<String>,
//     pub docs: Option<String>,
// }

// pub fn load_built_ins() -> Vec<BuiltinEntry> {
//     let json = include_str!("./builtin_catalog.json");
//     serde_json::from_str(json).expect("Failed to parse built-in catalog")
// }

// // Converts string representation of completion item kind to LSP's CompletionItemKind
// fn completion_kind_from_str(kind: &str) -> Option<CompletionItemKind> {
//     match kind {
//         "function" => Some(CompletionItemKind::FUNCTION),
//         "keyword" => Some(CompletionItemKind::KEYWORD),
//         "constant" => Some(CompletionItemKind::CONSTANT),
//         "operator" => Some(CompletionItemKind::OPERATOR),
//         "type" => Some(CompletionItemKind::UNIT),
//         _ => None,
//     }
// }

// fn insert_format_from_str(fmt: &str) -> Option<InsertTextFormat> {
//     match fmt {
//         "snippet" => Some(InsertTextFormat::SNIPPET),
//         "plain" => Some(InsertTextFormat::PLAIN_TEXT),
//         _ => None,
//     }
// }

// pub fn json_to_completion_item(builtins: &[BuiltinEntry]) -> Vec<CompletionItem> {
//     builtins
//         .iter()
//         .map(|b| CompletionItem {
//             label: b.name.clone(),
//             kind: completion_kind_from_str(&b.kind),
//             detail: b.signature.clone(),
//             documentation: b.docs.as_ref().map(|d| {
//                 Documentation::MarkupContent(MarkupContent {
//                     kind: MarkupKind::Markdown,
//                     value: d.clone(),
//                 })
//             }),
//             insert_text: b.insert.clone().or_else(|| Some(b.name.clone())),
//             insert_text_format: b.insert_format.as_deref().and_then(insert_format_from_str),
//             ..Default::default()
//         })
//         .collect()
// }
