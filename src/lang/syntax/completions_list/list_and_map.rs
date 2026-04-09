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

pub static LIST: &[DsrvBuiltIn] = &[
    DsrvBuiltIn {
        label: "List()",
        kind: CompletionItemKind::CONSTRUCTOR,
        trigger_context: &["expr"],
        insert_text: "List($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List(e1, e2, ...)",
        documentation: "Constructs a new List container from a comma-separated sequence of stream expressions.",
    },
    DsrvBuiltIn {
        label: "List.",
        kind: CompletionItemKind::CLASS,
        trigger_context: &["expr"],
        insert_text: "List.",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "List.",
        documentation: "The List type, to be called with methods like List.get, List.append, etc.",
    },
    DsrvBuiltIn {
        label: "List.get",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "get($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.get(L, i)",
        documentation: "Returns the element at the specified index",
    },
    DsrvBuiltIn {
        label: "List.append",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "append($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.append(L, e)",
        documentation: "Ruturns a new list with the element added to the end",
    },
    DsrvBuiltIn {
        label: "List.concat",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "concat($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.concat(L1, L2)",
        documentation: "Concatenates the two list into one list",
    },
    DsrvBuiltIn {
        label: "List.head",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "head($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.head(L)",
        documentation: "Returns the first element of the list",
    },
    DsrvBuiltIn {
        label: "List.tail",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "tail($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.tail(L)",
        documentation: "Returns a new list containing all elements except the first",
    },
    DsrvBuiltIn {
        label: "List.len",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["list method"],
        insert_text: "len($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "List.len(L)",
        documentation: "Returns the length of the list",
    },
];

pub static MAP: &[DsrvBuiltIn] = &[
      DsrvBuiltIn {
        label: "Map",
        kind: CompletionItemKind::CONSTRUCTOR,
        trigger_context: &["expr"],
        insert_text: "Map($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map(\"key\": val, ...)",
        documentation: "Constructs a new Map container from key-value pairs where keys are strings.",
    },
    DsrvBuiltIn {
        label: "Map",
        kind: CompletionItemKind::CLASS,
        trigger_context: &["expr"],
        insert_text: "Map.",
        insert_text_format: InsertTextFormat::PLAIN_TEXT,
        detail: "Map",
        documentation: "The Map type, to be called with methods like Map.get, Map.insert, etc.",
    },
    DsrvBuiltIn {
        label: "Map.get",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["map method"],
        insert_text: "get($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.get(M, k)",
        documentation: "Returns the value associated with the specified key",
    },
    DsrvBuiltIn {
        label: "Map.insert",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["map method"],
        insert_text: "insert($1, $2, $3)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.insert(M, k, v)",
        documentation: "Inserts the value into the map with the specified key",
    },
    DsrvBuiltIn {
        label: "Map.remove",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["map method"],
        insert_text: "remove($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.remove(M, k)",
        documentation: "Removes the value from the map at the specified key",
    },
    DsrvBuiltIn {
        label: "Map.has_key",
        kind: CompletionItemKind::METHOD,
        trigger_context: &["map method"],
        insert_text: "has_key($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Map.has_key(M, k)",
        documentation: "Checks if the map has a value associated with the specified key",
    },
];