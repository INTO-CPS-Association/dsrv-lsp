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

// use crate::utils::byte_to_pos;
use logos::Logos;
// use ropey::Rope;
// use tower_lsp::lsp_types::{Diagnostic, Range};
// use tower_lsp_server::ls_types::{Diagnostic, Range};


#[derive(Logos, Debug, Clone, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum Token {
    #[regex(r"//[^\n\r]*", allow_greedy = true)]
    LineComment,
    // #[token("(*", lex_block_comment)]
    // BlockComment,

    // Keywords
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,

    #[token("in")]
    In,
    #[token("out")]
    Out,
    #[token("aux")]
    Aux,
    #[token("var")]
    Var,

    // Primitives
    #[token("dynamic")]
    Dynamic,
    #[token("defer")]
    Defer,

    // Built in function keyword
    #[token("eval")]
    Eval,
    #[token("update")]
    Update,
    #[token("default")]
    Default,
    #[token("is_defined")]
    IsDefined,
    #[token("when")]
    When,
    #[token("latch")]
    Latch,
    #[token("init")]
    Init,
    #[token("monitored_at")]
    MonitoredAt,
    #[token("dist")]
    Dist,

    // Types
    #[token("Int")]
    Int,
    #[token("Float")]
    Float,
    #[token("Bool")]
    Bool,
    #[token("Str")]
    Str,
    #[token("Unit")]
    Unit,
    #[token("List")]
    List,
    #[token("Map")]
    Map,

    // Operators
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
    #[token("=>")]
    Impl,
    #[token("==")]
    EqEq,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("++")]
    Concat,
    #[token("=")]
    Eq,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    // Punctuation
    Bang,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    //Math Functions
    #[token("sin")]
    Sin,
    #[token("cos")]
    Cos,
    #[token("tan")]
    Tan,
    #[token("abs")]
    Abs,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r"\d+\.\d+")]
    FloatLiteral,
    #[regex(r"\d+")]
    IntLiteral,
    #[regex(r"\s+")]
    Whitespace,
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenData {
    pub token: Token,
    pub content: String,
    pub span: std::ops::Range<usize>,
}

/// Main function to tokenize source code and collect diagnostics if there are lexer errors. Returns a vector of tokens and their corresponding spans.
pub fn tokenize(text: &str) -> Vec<TokenData> {
    // create the lexer and iterate through the tokens.
    let mut lexer = Token::lexer(text);
    let mut tokens = Vec::new();

    while let Some(token_result) = lexer.next() {
        // Using the built in span method of the lexer to get the byte range of the token in the source code.
        let span = lexer.span();
        // Save the actual content (text) of the token by slicing the original source code using the span of the token.
        let content = lexer.slice().to_string();

        match token_result {
            Ok(t) => {
                // Ignore whitespace and comments
                if t != Token::Whitespace && t != Token::LineComment
                /* && t != Token::BlockComment  */
                {
                    tokens.push(TokenData {
                        token: t,
                        content,
                        span,
                    });
                }
            }
            Err(_) => {
                // If the token is an error, we push an error token with the content and span of the invalid token.
                tokens.push(TokenData {
                    token: Token::Error,
                    content,
                    span,
                });
            }
        }
    }
    tokens
}


// Helper function to get a slice of tokens around the cursor position for context-aware suggestions
pub fn get_context_slice(tokens: &[TokenData], cursor_offset: usize, n: usize) -> Vec<TokenData> {
    // We look for tokens that end before or exactly at the cursor, take the last n tokens, and reverse them to maintain the original order

    let pos = tokens
        .iter()
        .position(|t| t.span.start >= cursor_offset)
        .unwrap_or(tokens.len());

    // Look to see if the last token is an identifier, or literals and if true ignore it and take the previous token instead as the context for suggestions, this allows us to provide suggestions even when the user is in the middle of typing an identifier or a literal
    let mut end_idx = pos;
    if pos > 0 {
        let last_token = &tokens[pos - 1];
        match last_token.token {
            Token::Identifier | Token::IntLiteral | Token::StringLiteral | Token::FloatLiteral => {
                end_idx = pos - 1
            }
            _ => {}
        }
    }
    // Take the last n tokens before the cursor (ignoring the current token if it's an identifier or literal) and return them in the original order
    let start_idx = end_idx.saturating_sub(n);
    tokens[start_idx..end_idx].to_vec()
}

pub fn filter_suggestions(cursor_offset: usize, tokens: &[TokenData]) -> Vec<&'static str> {
    let context_tokens = get_context_slice(&tokens, cursor_offset, 3);
    log::info!("Tokens at cursor: {:?}", context_tokens);

    let last_token = match context_tokens.last() {
        Some(t) => t,
        None => {
            return vec!["toplevel"];
        }
    };
    match last_token.token {
        Token::In | Token::Out | Token::Aux | Token::Var => {
            vec!["_"]
        }

        Token::Colon | Token::Comma => {
            vec!["type"]
        }

        Token::Dot => {
            if context_tokens.len() >= 2 {
                match context_tokens[context_tokens.len() - 2].token {
                    Token::List => vec!["list method"],
                    Token::Map => vec!["map method"],
                    _ => vec![],
                }
            } else {
                vec![]
            }
        }

        #[rustfmt::skip]
        // Expression Context
        Token::Eq | Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent | Token::LParen | Token::LBracket | Token::AndAnd | Token::OrOr | Token::Impl | Token::EqEq | Token::Le | Token::Ge | Token::Lt | Token::Gt | Token::Bang | Token::Concat | Token::If | Token::Then | Token::Else => vec!["expr"],

        _ => vec!["toplevel", "expr"],
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fixtures;

    #[test]
    fn test_tokenize_simple_input() {
        let input = fixtures::input_untyped_valid_simple();
        let tokens = fixtures::tokenize_input(input);

        println!("Tokens: {:#?}", tokens);

        assert!(
            tokens.len() == 11,
            "Expected 11 tokens, got {}",
            tokens.len()
        );

        assert!(
            matches!(tokens[3].token, Token::Identifier),
            "Expected Identifier token, got {:?}",
            tokens[3].token
        );

        assert!(
            tokens[7].content == "=",
            "Expected '=' token, got {:?}",
            tokens[7].content
        )
    }

    #[test]
    fn test_tokenize_typed_input() {
        let input = fixtures::input_typed_valid_simple();
        let tokens = fixtures::tokenize_input(input);

        println!("Tokens: {:#?}", tokens);

        assert!(
            matches!(tokens[2].token, Token::Colon),
            "Expected Colon token, got {:?}",
            tokens[2].token
        );
        assert!(
            matches!(tokens[7].token, Token::Int),
            "Expected Int token, got {:?}",
            tokens[7].token
        );
    }

    #[test]
    fn test_tokenize_invalid_input() {
        let input = fixtures::input_untyped_invalid_simple();
        let tokens = fixtures::tokenize_input(input);

        println!("Tokens: {:#?}", tokens);

        assert!(!tokens.is_empty(), "Expected tokens, got none");

        assert!(
            matches!(tokens.last().unwrap().token, Token::Eq),
            "Expected Eq token, got {:?}",
            tokens.last().unwrap().token
        );
    }

    #[test]
    fn test_tokenize_empty_input() {
        let input = fixtures::input_empty();
        let tokens = fixtures::tokenize_input(input);

        println!("Tokens: {:#?}", tokens);

        assert!(tokens.is_empty(), "Expected no tokens, got {:?}", tokens);
    }

    #[test]
    fn test_get_context_slide() {
        use crate::lang::syntax::lexer::Token::*;

        let input = fixtures::input_untyped_invalid_simple();
        let tokens = fixtures::tokenize_input(input);

        let context_slide = get_context_slice(&tokens, 15, 2);

        let result = vec![
            TokenData {
                token: Identifier,
                content: "z".to_string(),
                span: 11..12,
            },
            TokenData {
                token: Eq,
                content: "=".to_string(),
                span: 13..14,
            },
        ];

        println!("Context slide: {:#?}", context_slide);

        assert!(
            context_slide.len() == 2,
            "Expected 2 tokens in context slide, got {}",
            context_slide.len()
        );
        assert_eq!(
            context_slide, result,
            "Expected context slide to be {:?}, got {:?}",
            result, context_slide
        );
    }

    #[test]
    fn test_get_context_slide_complex() {
        use crate::lang::syntax::lexer::Token::*;
        let input = fixtures::input_untyped_complex_invalid();
        let tokens = fixtures::tokenize_input(input);

        let context_slide = get_context_slice(&tokens, 250, 3);

        println!("Context slide: {:#?}", context_slide);

        let result = vec![
            TokenData {
                token: Eq,
                content: "=".to_string(),
                span: 244..245,
            },
            TokenData {
                token: Map,
                content: "Map".to_string(),
                span: 246..249,
            },
            TokenData {
                token: Dot,
                content: ".".to_string(),
                span: 249..250,
            },
        ];

        assert!(
            context_slide.len() == 3,
            "Expected 3 tokens in context slide, got {}",
            context_slide.len()
        );
        assert_eq!(
            context_slide, result,
            "Expected context slide to be {:?}, got {:?}",
            result, context_slide
        );
    }

    #[test]
    fn test_filter_suggestions_simple() {
        let input = fixtures::input_untyped_invalid_simple();
        let tokens = fixtures::tokenize_input(input);

        let filter = filter_suggestions(15, &tokens);

        println!("Filter: {:#?}", filter);

        assert!(!filter.is_empty(), "Expected suggestions, got none");

        assert_eq!(
            filter,
            vec!["expr"],
            "Expected [\"expr\"], but got {:?}",
            filter
        );
    }

    #[test]
    fn test_filter_suggestions_complex() {
        let input = fixtures::input_untyped_complex_invalid();
        let tokens = fixtures::tokenize_input(input);

        let filter = filter_suggestions(250, &tokens);

        println!("Filter: {:#?}", filter);

        assert!(!filter.is_empty(), "Expected suggestions, got none");

        assert_eq!(
            filter,
            vec!["map method"],
            "Expected [\"map method\"], but got {:?}",
            filter
        );
    }
}
