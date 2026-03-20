use crate::utils::byte_to_pos;
use logos::{Lexer, Logos};
use ropey::Rope;
use tower_lsp::lsp_types::{Diagnostic, Range};

// Custom function to handle block comments, allowing for nested block comments
fn lex_block_comment(lex: &mut Lexer<Token>) -> Result<(), LexerError> {
    let remainder = lex.remainder();
    let mut depth = 1;
    let mut chars = remainder.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '*' => {
                // Check for closing of block comment
                if chars.peek().map(|(_, c)| *c) == Some(')') {
                    chars.next(); // Consume the ')'
                    depth -= 1; // Decrease depth for block comment nesting
                    if depth == 0 {
                        // If depth is zero, we have closed all nested block comments
                        lex.bump(i + "*)".len()); // Advance the lexer past the closing '*)'
                        return Ok(());
                    }
                }
            }
            '(' => {
                // Check for nesting of block comment
                if chars.peek().map(|(_, c)| *c) == Some('*') {
                    chars.next(); // Consume the '*'
                    depth += 1; // Increase depth for block comment nesting
                }
            }
            _ => {} // Ignore other characters
        }
    }
    lex.bump(remainder.len());
    Err(LexerError::UnclosedComment)
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LexerError {
    #[default]
    Invalid,
    UnclosedComment,
}

// Implement a method to convert LexerError into a vector of Diagnostics with a given Span
impl LexerError {
    pub fn into_diags(self, span: std::ops::Range<usize>, source: &str) -> Diagnostic {
        let rope = Rope::from_str(source);
        let msg = match self {
            LexerError::Invalid => "Invalid token".to_string(),
            LexerError::UnclosedComment => "Unclosed block comment".to_string(),
        };
        Diagnostic::new_simple(
            Range::new(
                byte_to_pos(&rope, span.start).unwrap(),
                byte_to_pos(&rope, span.end).unwrap(),
            ),
            msg,
        )
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Copy, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
#[logos(error=LexerError)]
pub enum Token {
    #[regex(r"//[^\n\r]*", allow_greedy = true)]
    LineComment,
    #[token("(*", lex_block_comment)]
    BlockComment,

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

#[derive(Debug, Clone)]
pub struct TokenData {
    pub token: Token,
    pub content: String,
    pub span: std::ops::Range<usize>,
}

/// Main function to tokenize source code and collect diagnostics if there are lexer errors. Returns a vector of tokens and their corresponding spans.
pub fn tokenize(text: &str) -> Vec<TokenData> {
    let mut lexer = Token::lexer(text);
    let mut tokens = Vec::new();

    while let Some(token_result) = lexer.next() {
        let span = lexer.span();
        let content = lexer.slice().to_string();

        match token_result {
            Ok(t) => {
                if t != Token::Whitespace && t != Token::LineComment && t != Token::BlockComment {
                    tokens.push(TokenData {
                        token: t,
                        content,
                        span,
                    });
                }
            }
            Err(_) => {
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

pub fn find_token_at_cursor(tokens: &[TokenData], cursor_offset: usize) -> Option<&TokenData> {
    tokens
        .iter()
        .filter(|t| t.span.start <= cursor_offset)
        .last()
}

pub fn find_token_pair_at_cursor(tokens: &[TokenData], cursor_offset: usize) -> Vec<&TokenData> {
    tokens
        .iter()
        // We look for tokens that end before or exactly at the cursor
        .filter(|t| t.span.end <= cursor_offset)
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

pub fn filter_suggestions(cursor_offset: usize, tokens: &[TokenData]) -> Vec<&str> {
    let mut context = Vec::new();

    let last_pair = find_token_pair_at_cursor(&tokens, cursor_offset);
    log::info!("Tokens at cursor: {:?}", last_pair);
    let first_token = last_pair.first().unwrap();

    match first_token.token {
      Token::List => {
        if last_pair.last().unwrap().token == Token::Dot {
            context.push("list method")
        }
      },
      Token::Map => {
        if last_pair.last().unwrap().token == Token::Dot {
            context.push("map method")
        }
      },
    
      Token::Eq | Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent | Token::LParen | Token::Comma | Token::LBracket | Token::AndAnd | Token::OrOr | Token::Impl | Token::EqEq | Token::Le | Token::Ge | Token::Lt | Token::Gt | Token::Bang => context.push("expr"),
      
      
      Token::In | Token::Aux | Token::Out | Token::Var => context.push("_"),
      
        
        _ => context.extend(["toplevel", "variable"]),
    } 
    context
}
