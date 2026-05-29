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

use super::*;

pub static CORE: &[DsrvBuiltIn] = &[
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
];

pub static TYPES: &[DsrvBuiltIn] = &[
    DsrvBuiltIn {
        label: "Int",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["type"],
        insert_text: "Int",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "Int",
        documentation: "Integer type, representing whole numbers without a fractional component.",
    },
    DsrvBuiltIn {
        label: "Float",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["type"],
        insert_text: "Float",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "Float",
        documentation: "Float type, representing numbers that can have a fractional component.",
    },
    DsrvBuiltIn {
        label: "Bool",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["type"],
        insert_text: "Bool",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "Bool",
        documentation: "Boolean type representing truth values, which can be either `true` or `false`.",
    },
    DsrvBuiltIn {
        label: "Str",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["type"],
        insert_text: "Str",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "String",
        documentation: "String type representing sequences of characters, used for textual data.",
    },
    DsrvBuiltIn {
        label: "Unit",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["type"],
        insert_text: "Unit",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "Unit",
        documentation: "Unit type representing a pure event or trigger stream. It carries no data payload and is used purely to signal that an event occurred at a specific timestep.",
    },
    
    DsrvBuiltIn {
    label: "false",
    kind: CompletionItemKind::CONSTANT,
    trigger_context: &["expr"],
    insert_text: "false",
    insert_text_format: InsertTextFormat::PLAIN_TEXT,
    detail: "False",
    documentation: "The boolean constant `false`, representing the logical value of falsehood.",
    },
    
    DsrvBuiltIn {
        label: "true",
        kind: CompletionItemKind::CONSTANT,
        trigger_context: &["expr"],
        insert_text: "true",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "True",
        documentation: "The boolean constant `true`, representing the logical value of truth.",
    },
];
