use super::*;

pub static MATH: &[DsrvBuiltIn] = &[
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
