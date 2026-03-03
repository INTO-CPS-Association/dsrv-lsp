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
                        lex.bump(i + 2); // Advance the lexer past the closing '*)'
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

#[derive(Logos, Debug, Clone, PartialEq)]
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

    // Operators and punctuation
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
    Bang,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLiteral,
    #[regex(r"-?\d+", |lex| lex.slice().parse::<i64>().ok())]
    IntLiteral(i64),
    #[regex(r"-?\d+\.\d+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLiteral(f64),
    #[regex(r"\s+")]
    Whitespace,
    Error,
}


/// Main function to tokenize source code and collect diagnostics if there are lexer errors. Returns a vector of tokens and their corresponding spans.
pub fn tokenize(source: &str, diags: &mut Vec<Diagnostic>) -> (Vec<Token>, Vec<std::ops::Range<usize>>) {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();
    let mut spans = Vec::new();

    while let Some(result) = lexer.next() {
        let span = lexer.span(); // Get the byte range of the current token
        match result {
            Ok(token) => {
                tokens.push(token); // Push the successfully parsed token to the tokens vector
            }
            Err(err) => {
                diags.push(err.into_diags(span.clone(), source));
                tokens.push(Token::Error); // Push an error token to the tokens vector to maintain alignment with spans, even if the token couldn't be parsed successfully
            }
        }
        spans.push(span); // Push the span of the current token to the spans vector, regardless of whether it was successfully parsed or not
    }

    (tokens, spans)
}
