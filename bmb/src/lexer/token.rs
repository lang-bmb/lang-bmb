//! Token definitions

use logos::Logos;

/// BMB Token
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*")]
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
    // v0.5 Phase 2: Mutability and loops
    #[token("mut")]
    Mut,
    // v0.51.23: Store assignment keyword
    #[token("set")]
    Set,
    #[token("while")]
    While,
    // v0.5 Phase 3: For loop
    #[token("for")]
    For,
    #[token("in")]
    In,
    // v0.36: Additional control flow
    #[token("loop")]
    Loop,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("return")]
    Return,
    // v0.36: Bitwise operators
    #[token("band")]
    Band,
    #[token("bor")]
    Bor,
    #[token("bxor")]
    Bxor,
    #[token("bnot")]
    Bnot,
    // v0.5 Phase 4: Module system
    #[token("pub")]
    Pub,
    #[token("use")]
    Use,
    #[token("mod")]
    Mod,
    // v0.2: Contract system
    #[token("where")]
    Where,
    // v0.2: Refinement type self-reference
    #[token("it")]
    It,
    // v0.13.0: External function declaration
    #[token("extern")]
    Extern,
    // v0.13.2: Error propagation
    #[token("try")]
    Try,
    // v0.39: Type casting
    #[token("as")]
    As,
    // v0.51.40: Null pointer literal
    #[token("null")]
    Null,
    // v0.51.41: Sizeof operator
    #[token("sizeof")]
    Sizeof,
    // v0.20.1: Trait system
    #[token("trait")]
    Trait,
    #[token("impl")]
    Impl,
    // v0.31: Incremental development
    #[token("todo")]
    Todo,

    // v0.70: Concurrency primitives
    #[token("spawn")]
    Spawn,

    // v0.71: Mutex type for thread-safe synchronization
    #[token("Mutex")]
    MutexType,

    // v0.72: Arc and Atomic types for shared memory concurrency
    #[token("Arc")]
    ArcType,
    #[token("Atomic")]
    AtomicType,

    // v0.73: Channel types for message-passing concurrency
    #[token("channel")]
    ChannelKw,
    #[token("Sender")]
    SenderType,
    #[token("Receiver")]
    ReceiverType,

    // v0.74: Advanced synchronization primitives
    #[token("RwLock")]
    RwLockType,
    #[token("Barrier")]
    BarrierType,
    #[token("Condvar")]
    CondvarType,

    // v0.50.6: Type aliases and refinement types
    #[token("type")]
    Type,

    // v0.36: Contract keywords
    #[token("invariant")]
    Invariant,
    #[token("implies")]
    Implies,

    // v0.37: Quantifiers for verification
    #[token("forall")]
    Forall,
    #[token("exists")]
    Exists,

    // v0.31: Module header system (RFC-0002)
    #[token("module")]
    Module,
    #[token("version")]
    Version,
    #[token("summary")]
    Summary,
    #[token("exports")]
    Exports,
    #[token("depends")]
    Depends,
    #[token("===")]
    HeaderSep,

    // Type keywords
    #[token("i32")]
    TyI32,
    #[token("i64")]
    TyI64,
    // v0.38: Unsigned integer types
    #[token("u32")]
    TyU32,
    #[token("u64")]
    TyU64,
    #[token("f64")]
    TyF64,
    #[token("bool")]
    TyBool,
    #[token("String")]
    TyString,
    // v0.64: Character type
    #[token("char")]
    TyChar,

    // Literals
    // v0.34: Extended to support scientific notation (e.g., 3.14e10, 1e-5, 6.022E23)
    #[regex(r"[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?|[0-9]+[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok(), priority = 3)]
    FloatLit(f64),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok(), priority = 2)]
    IntLit(i64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        // Remove surrounding quotes and process escape sequences
        let inner = &s[1..s.len()-1];
        let mut result = String::new();
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('0') => result.push('\0'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        result
    })]
    StringLit(String),

    // v0.64: Character literals with escape sequences
    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        // Remove surrounding quotes: 'x' -> x, '\n' -> \n
        let inner = &s[1..s.len()-1];
        if inner.starts_with('\\') && inner.len() == 2 {
            // Handle escape sequences
            match inner.chars().nth(1) {
                Some('n') => Some('\n'),
                Some('t') => Some('\t'),
                Some('r') => Some('\r'),
                Some('\\') => Some('\\'),
                Some('\'') => Some('\''),
                Some('0') => Some('\0'),
                _ => None, // Invalid escape
            }
        } else if inner.len() == 1 {
            inner.chars().next()
        } else {
            None
        }
    })]
    CharLit(char),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string(), priority = 1)]
    Ident(String),

    // Symbols
    // v0.51.23: Store operator for index/field assignment
    #[token(":=")]
    ColonEq,
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
    // v0.2: Range operators (order matters - longer first)
    #[token("..<")]
    DotDotLt,
    #[token("..=")]
    DotDotEq,
    #[token("..")]
    DotDot,
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
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    // v0.5 Phase 5: References
    #[token("&")]
    Ampersand,
    // v0.2: Attributes
    #[token("@")]
    At,
    // v0.13.2: Error propagation operator
    #[token("?")]
    Question,
    // v0.20.0: Closure syntax
    #[token("|")]
    Pipe,

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

    // v0.32: Shift operators
    #[token("<<")]
    LtLt,
    #[token(">>")]
    GtGt,

    // v0.37: Wrapping arithmetic operators
    #[token("+%")]
    PlusPercent,
    #[token("-%")]
    MinusPercent,
    #[token("*%")]
    StarPercent,

    // v0.38: Checked arithmetic operators (return Option<T>)
    #[token("+?")]
    PlusQuestion,
    #[token("-?")]
    MinusQuestion,
    #[token("*?")]
    StarQuestion,

    // v0.38: Saturating arithmetic operators (clamp to min/max)
    #[token("+|")]
    PlusPipe,
    #[token("-|")]
    MinusPipe,
    #[token("*|")]
    StarPipe,

    // v0.32: Symbolic logical operators
    #[token("&&")]
    AmpAmp,
    #[token("||")]
    PipePipe,
    #[token("!")]
    Bang,
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
            Token::Mut => write!(f, "mut"),
            // v0.51.23: Store assignment keyword
            Token::Set => write!(f, "set"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::In => write!(f, "in"),
            Token::Pub => write!(f, "pub"),
            Token::Use => write!(f, "use"),
            Token::Mod => write!(f, "mod"),
            Token::Where => write!(f, "where"),
            Token::It => write!(f, "it"),
            Token::Extern => write!(f, "extern"),
            Token::Try => write!(f, "try"),
            Token::As => write!(f, "as"),
            Token::Null => write!(f, "null"),
            Token::Sizeof => write!(f, "sizeof"),
            Token::Trait => write!(f, "trait"),
            Token::Impl => write!(f, "impl"),
            Token::TyI32 => write!(f, "i32"),
            Token::TyI64 => write!(f, "i64"),
            // v0.38: Unsigned types
            Token::TyU32 => write!(f, "u32"),
            Token::TyU64 => write!(f, "u64"),
            Token::TyF64 => write!(f, "f64"),
            Token::TyBool => write!(f, "bool"),
            Token::TyString => write!(f, "String"),
            // v0.64: Char type
            Token::TyChar => write!(f, "char"),
            Token::IntLit(n) => write!(f, "{n}"),
            Token::FloatLit(n) => write!(f, "{n}"),
            Token::StringLit(s) => write!(f, "\"{s}\""),
            // v0.64: Character literal display
            Token::CharLit(c) => write!(f, "'{c}'"),
            Token::Ident(s) => write!(f, "{s}"),
            Token::Colon => write!(f, ":"),
            // v0.51.23: Store operator
            Token::ColonEq => write!(f, ":="),
            Token::ColonColon => write!(f, "::"),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::Underscore => write!(f, "_"),
            Token::DotDotLt => write!(f, "..<"),
            Token::DotDotEq => write!(f, "..="),
            Token::DotDot => write!(f, ".."),
            Token::Dot => write!(f, "."),
            Token::Eq => write!(f, "="),
            Token::Semi => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Ampersand => write!(f, "&"),
            Token::At => write!(f, "@"),
            Token::Question => write!(f, "?"),
            Token::Pipe => write!(f, "|"),
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
            // v0.32: Shift operators
            Token::LtLt => write!(f, "<<"),
            Token::GtGt => write!(f, ">>"),
            // v0.37: Wrapping arithmetic operators
            Token::PlusPercent => write!(f, "+%"),
            Token::MinusPercent => write!(f, "-%"),
            Token::StarPercent => write!(f, "*%"),
            // v0.38: Checked arithmetic operators
            Token::PlusQuestion => write!(f, "+?"),
            Token::MinusQuestion => write!(f, "-?"),
            Token::StarQuestion => write!(f, "*?"),
            // v0.38: Saturating arithmetic operators
            Token::PlusPipe => write!(f, "+|"),
            Token::MinusPipe => write!(f, "-|"),
            Token::StarPipe => write!(f, "*|"),
            // v0.32: Symbolic logical operators
            Token::AmpAmp => write!(f, "&&"),
            Token::PipePipe => write!(f, "||"),
            Token::Bang => write!(f, "!"),
            Token::Todo => write!(f, "todo"),
            // v0.70: Concurrency primitives
            Token::Spawn => write!(f, "spawn"),
            // v0.71: Mutex type
            Token::MutexType => write!(f, "Mutex"),
            // v0.72: Arc and Atomic types
            Token::ArcType => write!(f, "Arc"),
            Token::AtomicType => write!(f, "Atomic"),
            // v0.73: Channel types
            Token::ChannelKw => write!(f, "channel"),
            Token::SenderType => write!(f, "Sender"),
            Token::ReceiverType => write!(f, "Receiver"),
            // v0.74: Advanced synchronization primitives
            Token::RwLockType => write!(f, "RwLock"),
            Token::BarrierType => write!(f, "Barrier"),
            Token::CondvarType => write!(f, "Condvar"),
            // v0.50.6: Type aliases
            Token::Type => write!(f, "type"),
            // v0.31: Module header tokens
            Token::Module => write!(f, "module"),
            Token::Version => write!(f, "version"),
            Token::Summary => write!(f, "summary"),
            Token::Exports => write!(f, "exports"),
            Token::Depends => write!(f, "depends"),
            Token::HeaderSep => write!(f, "==="),
            // v0.36: Additional control flow
            Token::Loop => write!(f, "loop"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Return => write!(f, "return"),
            // v0.36: Bitwise operators
            Token::Band => write!(f, "band"),
            Token::Bor => write!(f, "bor"),
            Token::Bxor => write!(f, "bxor"),
            Token::Bnot => write!(f, "bnot"),
            // v0.36: Contract keywords
            Token::Invariant => write!(f, "invariant"),
            Token::Implies => write!(f, "implies"),
            // v0.37: Quantifiers
            Token::Forall => write!(f, "forall"),
            Token::Exists => write!(f, "exists"),
        }
    }
}
