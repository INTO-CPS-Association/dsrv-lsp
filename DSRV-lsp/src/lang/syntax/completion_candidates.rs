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
        insert_text: "in $1",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "in <label> [: <Type>]",
        documentation: "Declares an input stream that provides a sequence of event values to the monitor. The label acts as a variable name in the input namespace in(ϕ)",
    },
    DsrvBuiltIn {
        label: "out",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["toplevel"],
        insert_text: "out ${1}",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "out <label> [: <Type>]",
        documentation: "Declares an output stream, also known as a verdict stream. It transforms input data into results based on defined stream equations.",
    },
    DsrvBuiltIn {
        label: "aux",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["toplevel"],
        insert_text: "aux $1  ",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "aux <label> [: <Type>]",
        documentation: "Declares an auxiliary internal stream variable used to simplify complex equations and is not exposed as a final output",
    },
    DsrvBuiltIn {
        label: "dynamic",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "dynamic($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "dynamic(ψ)",
        documentation: "Dynamic property which behaves like the most recent value of ψ or is `deferred (⊥)` if none has been sent",
    },
    DsrvBuiltIn {
        label: "If then else",
        kind: CompletionItemKind::SNIPPET,
        trigger_context: &["expr"],
        insert_text: "if ${1:condition} then ${2:expr} else ${3:expr}",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "if σ then ψ1 else ψ2",
        documentation: "Conditional expression that evaluates σ and returns ψ1 if σ is `true`, otherwise returns ψ2",
    },
    DsrvBuiltIn {
        label: "defer",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "defer($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "defer(ψ)",
        documentation: "DUP primitives that allows exactly one dynamic update. It remains `deferred (⊥)` until the first point at which ψ becomes available, after whichs it behaves as ψ.",
    },
    DsrvBuiltIn {
        label: "update",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "update($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "update(ψ1, ψ2)",
        documentation: "DUP helper that returns the value of ψ1 until the first point that ψ2 is `not deferred (⊥) `after which it returns ψ2.",
    },
    DsrvBuiltIn {
        label: "default",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "default($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "default(ψ, c)",
        documentation: "DUP helper that returns defaults to the value, c, if ψ is `deferred (⊥)` otherwise returns the value of ψ.",
    },
    DsrvBuiltIn {
        label: "is_defined",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "is_defined($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "is_defined(ψ)",
        documentation: "Returns `False` if ψ is `deferred (⊥)` otherwise returns `True` if ψ is `not deferred`",
    },
    DsrvBuiltIn {
        label: "when",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "when($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "when(ψ)",
        documentation: "Returns `False` until the first time ψ is `not deferred (⊥)` after which it returns `True`",
    },
    DsrvBuiltIn {
        label: "latch",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "latch($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "latch(v, t)",
        documentation: "", //TODO: Add docstring for latch after speaking with T
    },
    DsrvBuiltIn {
        label: "init",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "init($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "init()",  //TODO: Add proper detail for init after speaking with T
        documentation: "", //TODO: Add docstring for init after speaking with T
    },
    DsrvBuiltIn {
        label: "List",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "List($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List(e1, e2, ...)",
        documentation: "Constructs a new List container from a comma-separated sequence of stream expressions.",
    },
    DsrvBuiltIn {
        label: "List.get",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.get($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.get(L, i)",
        documentation: "Returns the element at the specified index",
    },
    DsrvBuiltIn {
        label: "List.append",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.append($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.append(L, e)",
        documentation: "Ruturns a new list with the element added to the end",
    },
    DsrvBuiltIn {
        label: "List.concat",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.concat($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.concat(L1, L2)",
        documentation: "Concatenates the two list into one list",
    },
    DsrvBuiltIn {
        label: "List.head",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.head($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.head(L)",
        documentation: "Returns the first element of the list",
    },
    DsrvBuiltIn {
        label: "List.tail",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.tail($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.tail(L)",
        documentation: "Returns the last element of the list",
    },
    DsrvBuiltIn {
        label: "List.len",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "List.len($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.len(L)",
        documentation: "Returns the length of the list",
    },
    DsrvBuiltIn {
        label: "Map",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "Map($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map(\"key\": val, ...)",
        documentation: "Constructs a new Map container from key-value pairs where keys are strings.",
    },
    DsrvBuiltIn {
        label: "Map.get",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "Map.get($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.get(M, k)",
        documentation: "Returns the value associated with the specified key",
    },
    DsrvBuiltIn {
        label: "Map.insert",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "Map.insert($1, $2, $3)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.insert(M, k, v)",
        documentation: "Inserts the value into the map with the specified key",
    },
    DsrvBuiltIn {
        label: "Map.remove",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "Map.remove($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.remove(M, k)",
        documentation: "Removes the value from the map at the specified key",
    },
    DsrvBuiltIn {
        label: "Map.has_key",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["expr"],
        insert_text: "Map.has_key($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.had_key(M, k)",
        documentation: "Checks if the map has a value associated with the specified key",
    },
    DsrvBuiltIn {
        label: "sin",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "sin($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "sin(e)",
        documentation: "Gives the sinus of the given value",
    },
    DsrvBuiltIn {
        label: "cos",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "cos($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "cos(e)",
        documentation: "Gives the cosinus of the given value",
    },
    DsrvBuiltIn {
        label: "tan",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "tan($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "tan(e)",
        documentation: "Gives the tangens of the given value",
    },
    DsrvBuiltIn {
        label: "abs",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "abs($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "abs(e)",
        documentation: "Returns the absolute value of the expression",
    },
    //TODO: Add monitored at after speaking with T about the semantics and signature of monitored at
    DsrvBuiltIn {
        label: "Monitore_at",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "",
        documentation: "",
    },
    // TODO: Add dist after speaking with T about the semantics and signature of dist
    DsrvBuiltIn {
        label: "dist",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "",
        documentation: "",
    },
    DsrvBuiltIn {
        label: "SIndex",
        kind: CompletionItemKind::OPERATOR,
        trigger_context: &["expr"],
        insert_text: "[$1]",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "expr[i]",
        documentation: "Temporal lookback operator. Accesses the value of the stream at 'i' time units in the past.",
    },
    DsrvBuiltIn {
        label: "Not",
        kind: CompletionItemKind::OPERATOR,
        trigger_context: &["expr"],
        insert_text: "!$1",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "!ψ",
        documentation: "Logical negation operator, inverts a boolean stream value",
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
