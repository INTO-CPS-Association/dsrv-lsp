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

pub static FUNCTIONS: &[DsrvBuiltIn] = &[
    DsrvBuiltIn {
        label: "dynamic",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "dynamic($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "dynamic(ψ [, type])",
        documentation: "Dynamic property which evaluates a stream of strings, ψ or is `deferred (⊥)` if none has been sent. \n\n Optionally takes a type annotation (e.g, `, Int`) to ensure the dynamically generated stream matches the expected type.",
    },
    DsrvBuiltIn {
        label: "eval",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "eval($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "eval(ψ, [type])",
        documentation: "Alias for dynamic. DUP primitive which evaluates a stream of strings, ψ or is `deferred (⊥)` if none has been sent. \n\n Optionally takes a type annotation (e.g, `, Int`) to ensure the dynamically generated stream matches the expected type.",
    },
    DsrvBuiltIn {
        label: "defer",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "defer($1)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "defer(ψ, [type])",
        documentation: "DUP primitive that remains `deferred (⊥)` until the first point at which ψ becomes available, after which it behaves as ψ. Supports optional Type Ascription..",
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
        detail: "latch(x, y)",
        documentation: "A sample-and-hold latch, where x provides the values to be sampled while y controls when they are sampled.\n
        When y provides a non-∄ value the latch samples the current value of x and holds that value until the next time y provides a non-∄ value. Otherwise, it evaluates to ∄, mirroring the semantics of an input stream w who receives the values of x but emits events only when y is non-∄", 
    },
    DsrvBuiltIn {
        label: "init",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "init($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "init(ψ, c)",
        documentation: "Initializes the ψ stream with values from the c stream until ψ provides a value that is not `NoVal`. Then yields from ψ.",
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
        label: "If then else",
        kind: CompletionItemKind::SNIPPET,
        trigger_context: &["expr"],
        insert_text: "if ${1:condition} then ${2:expr} else ${3:expr}",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "if σ then ψ1 else ψ2",
        documentation: "Conditional expression that evaluates σ and returns ψ1 if σ is `true`, otherwise returns ψ2",
    },
    DsrvBuiltIn {
        label: "then",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["condition"],
        insert_text: "then $1",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "then ψ1",
        documentation: "Part of the if-then-else expression. It follows the condition and introduces the expression that is returned when the condition is `true`.",
    },
    DsrvBuiltIn {
        label: "else",
        kind: CompletionItemKind::KEYWORD,
        trigger_context: &["condition"],
        insert_text: "else $1",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "else ψ2",
        documentation: "Part of the if-then-else expression. It follows the then clause and introduces the expression that is returned when the condition is `false`.",
    },
];

pub static DIST_FUNCTIONS: &[DsrvBuiltIn] = &[
    DsrvBuiltIn {
        label: "Monitored_at",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "Monitored_at($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Monitored_at(v, n)",
        documentation: "Returns a boolean stream that continuously evaluates to `true` if the variable `v` is currently being monitored by the computational node `n` in the distributed graph, and `false` otherwise.",
    },
    DsrvBuiltIn {
        label: "dist",
        kind: CompletionItemKind::FUNCTION,
        trigger_context: &["expr"],
        insert_text: "Dist($1, $2)",
        insert_text_format: InsertTextFormat::SNIPPET,
        detail: "Dist(u, v)",
        documentation: "Returns an integer stream representing the shortest topological distance between two entities (`u` and `v`) in the dynamically evolving distributed network graph. Both `u` and `v` can be either a variable name or a node name.",
    },
];
