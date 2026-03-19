#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
// --- Trivia (Essential for LSP) ---
    Whitespace = 0,
    Comment,
    Error, // Represents malformed text chunks

    // --- Values and Identifiers ---
    Ident,      // e.g., "my_var", "Node1"
    Number,     // e.g., "42", "3.14"
    StringLit,  // e.g., "hello"
    BoolLit,    // "true", "false"

    // --- Types ---
    IntType, FloatType, StrType, BoolType, 
    
    // --- Keywords ---
    InKw, OutKw, AuxKw,
    IfKw, ThenKw, ElseKw,
    DynamicKw, DeferKw, UpdateKw, DefaultKw,
    IsDefinedKw, WhenKw, EvalKw,
    LatchKw, InitKw,
    MonitoredAtKw, DistKw,
    
    // --- Built-in Module Keywords (for List.get, Map.insert, etc.) ---
    ListKw, MapKw,
    SinKw, CosKw, TanKw, AbsKw,

    // --- Operators (SBinOp mapping) ---
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    DoublePipe, // ||
    DoubleAmp,  // &&
    DoublePlus, // ++ (Concat)
    EqEq,       // ==
    LAngleEq,   // <=
    RAngleEq,   // >=
    LAngle,     // <
    RAngle,     // >
    Bang,       // !
    Arrow,      // => (Implication)

    // --- Punctuation ---
    Dot,        // . (for Map.get)
    Comma,      // ,
    Colon,      // :
    Eq,         // = (Assignment)
    LParen, RParen,     // ( )
    LBracket, RBracket, // [ ]
    LBrace, RBrace,     // { }

    // --- Composite Nodes ---
    Root,               // The whole DsrvSpecification
    StmtAssignment,     // STopDecl::Assignment
    StmtInput,          // STopDecl::Input
    ExprBinOp,          // SExpr::BinOp
    ExprIf,             // SExpr::If
    ExprList,           // SExpr::List
    ExprMap,            // SExpr::Map
    ParamList,          // For function arguments
}

// use crate::lang::syntax::cst::SyntaxKind::*;
use crate::lang::syntax::lexer::Token;
/// Maps a Token to its corresponding SyntaxKind for CST construction. This is essential for building the CST nodes correctly based on the tokens.
pub fn token_to_syntaxkind(token: &Token) -> SyntaxKind {
  match token {
    // --- Trivia (Essential for LSP) ---
    Token::Whitespace => SyntaxKind::Whitespace,
    Token::LineComment | Token::BlockComment => SyntaxKind::Comment,
    Token::Error => SyntaxKind::Error,
    
    // --- Values and Identifiers ---
    Token::Identifier => SyntaxKind::Ident,
    Token::IntLiteral | Token::FloatLiteral => SyntaxKind::Number,
    Token::StringLiteral => SyntaxKind::StringLit,
    Token::True | Token::False => SyntaxKind::BoolLit,

    // --- Keywords ---
    Token::In => SyntaxKind::InKw,
    Token::Out => SyntaxKind::OutKw,
    Token::Aux | Token::Var  => SyntaxKind::AuxKw,
    Token::If => SyntaxKind::IfKw,
    Token::Then => SyntaxKind::ThenKw,
    Token::Else => SyntaxKind::ElseKw,
    Token::Dynamic => SyntaxKind::DynamicKw,
    Token::Defer => SyntaxKind::DeferKw,
    Token::Update => SyntaxKind::UpdateKw,
    Token::Default => SyntaxKind::DefaultKw,
    Token::Eval => SyntaxKind::EvalKw,
    Token::IsDefined => SyntaxKind::IsDefinedKw,
    Token::When => SyntaxKind::WhenKw,
    Token::Latch => SyntaxKind::LatchKw,
    Token::Init => SyntaxKind::InitKw,
    Token::MonitoredAt => SyntaxKind::MonitoredAtKw,
    Token::Dist => SyntaxKind::DistKw,

    // --- Built-in Module Keywords (for List.get, Map.insert, etc.) ---
    Token::List => SyntaxKind::ListKw,
    Token::Map => SyntaxKind::MapKw,
    
    // --- Types
    Token::Int => SyntaxKind::IntType,
    Token::Float => SyntaxKind::FloatType,
    Token::Str => SyntaxKind::StrType,
    Token::Bool => SyntaxKind::BoolType,
    
    // SinKw, CosKw, TanKw, AbsKw,
    Token::Sin => SyntaxKind::SinKw,
    Token::Cos => SyntaxKind::CosKw,
    Token::Tan => SyntaxKind::TanKw,
    Token::Abs => SyntaxKind::AbsKw,

    // --- Operators (SBinOp mapping) ---
    Token::Plus => SyntaxKind::Plus,
    Token::Minus => SyntaxKind::Minus,
    Token::Star => SyntaxKind::Star,
    Token::Slash => SyntaxKind::Slash,
    Token::Percent => SyntaxKind::Percent,
    Token::AndAnd => SyntaxKind::DoubleAmp,
    Token::OrOr => SyntaxKind::DoublePipe,
    Token::Concat => SyntaxKind::DoublePlus,
    Token::EqEq => SyntaxKind::EqEq,
    Token::Le => SyntaxKind::LAngleEq,
    Token::Ge => SyntaxKind::RAngleEq,
    Token::Lt => SyntaxKind::LAngle,
    Token::Gt => SyntaxKind::RAngle,
    Token::Bang => SyntaxKind::Bang,
    Token::Impl => SyntaxKind::Arrow,
    
    // --- Punctuation ---
    Token::Dot => SyntaxKind::Dot,
    Token::Comma => SyntaxKind::Comma,
    Token::Colon => SyntaxKind::Colon,
    Token::Eq => SyntaxKind::Eq,
    Token::LParen => SyntaxKind::LParen,
    Token::RParen => SyntaxKind::RParen,
    Token::LBracket => SyntaxKind::LBracket,
    Token::RBracket => SyntaxKind::RBracket,
    Token::LBrace => SyntaxKind::LBrace,
    Token::RBrace => SyntaxKind::RBrace,

  }
}