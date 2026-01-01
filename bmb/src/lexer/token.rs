//! Token definitions

use logos::Logos;

/// BMB Token
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"--[^\n]*")]
pub enum Token {
    // Keywords
    #[token("fn")]
    Fn,
    #[token("let")]
    Let,
    #[token("var")]
    Var,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("pre")]
    Pre,
    #[token("post")]
    Post,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("ret")]
    Ret,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("not")]
    Not,
    // v0.5: Data types
    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,
    #[token("match")]
    Match,
    #[token("new")]
    New,

    // Type keywords
    #[token("i32")]
    TyI32,
    #[token("i64")]
    TyI64,
    #[token("f64")]
    TyF64,
    #[token("bool")]
    TyBool,

    // Literals
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().ok())]
    FloatLit(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok(), priority = 2)]
    IntLit(i64),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),

    // Symbols
    #[token(":")]
    Colon,
    #[token("::")]
    ColonColon,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("_")]
    Underscore,
    #[token(".")]
    Dot,
    #[token("=")]
    Eq,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // Operators
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
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Fn => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::Var => write!(f, "var"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Else => write!(f, "else"),
            Token::Pre => write!(f, "pre"),
            Token::Post => write!(f, "post"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Ret => write!(f, "ret"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Struct => write!(f, "struct"),
            Token::Enum => write!(f, "enum"),
            Token::Match => write!(f, "match"),
            Token::New => write!(f, "new"),
            Token::TyI32 => write!(f, "i32"),
            Token::TyI64 => write!(f, "i64"),
            Token::TyF64 => write!(f, "f64"),
            Token::TyBool => write!(f, "bool"),
            Token::IntLit(n) => write!(f, "{n}"),
            Token::FloatLit(n) => write!(f, "{n}"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Colon => write!(f, ":"),
            Token::ColonColon => write!(f, "::"),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Underscore => write!(f, "_"),
            Token::Dot => write!(f, "."),
            Token::Eq => write!(f, "="),
            Token::Semi => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::EqEq => write!(f, "=="),
            Token::NotEq => write!(f, "!="),
            Token::LtEq => write!(f, "<="),
            Token::GtEq => write!(f, ">="),
            Token::Lt => write!(f, "<"),
            Token::Gt => write!(f, ">"),
        }
    }
}
