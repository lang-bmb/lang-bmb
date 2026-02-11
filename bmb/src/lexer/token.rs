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

    // v0.75: Async/await keywords
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("Future")]
    FutureType,

    // v0.82: Select macro for multi-channel operations
    #[token("select")]
    Select,

    // v0.83: Async I/O types
    #[token("AsyncFile")]
    AsyncFileType,
    #[token("AsyncSocket")]
    AsyncSocketType,

    // v0.84: Thread pool for parallel task execution
    #[token("ThreadPool")]
    ThreadPoolType,

    // v0.85: Scoped threads for structured concurrency
    #[token("Scope")]
    ScopeType,

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

    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..].replace('_', ""), 16).ok()
    }, priority = 3)]
    #[regex(r"0[oO][0-7][0-7_]*", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..].replace('_', ""), 8).ok()
    }, priority = 3)]
    #[regex(r"0[bB][01][01_]*", |lex| {
        let s = lex.slice();
        i64::from_str_radix(&s[2..].replace('_', ""), 2).ok()
    }, priority = 3)]
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
            // v0.75: Async/await
            Token::Async => write!(f, "async"),
            Token::Await => write!(f, "await"),
            Token::FutureType => write!(f, "Future"),
            // v0.82: Select macro
            Token::Select => write!(f, "select"),
            // v0.83: Async I/O types
            Token::AsyncFileType => write!(f, "AsyncFile"),
            Token::AsyncSocketType => write!(f, "AsyncSocket"),
            // v0.84: ThreadPool type
            Token::ThreadPoolType => write!(f, "ThreadPool"),
            // v0.85: Scope type
            Token::ScopeType => write!(f, "Scope"),
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

#[cfg(test)]
mod tests {
    use super::*;

    // === Display tests for all token categories ===

    #[test]
    fn test_display_keyword_control_flow() {
        assert_eq!(format!("{}", Token::Fn), "fn");
        assert_eq!(format!("{}", Token::Let), "let");
        assert_eq!(format!("{}", Token::Var), "var");
        assert_eq!(format!("{}", Token::If), "if");
        assert_eq!(format!("{}", Token::Then), "then");
        assert_eq!(format!("{}", Token::Else), "else");
        assert_eq!(format!("{}", Token::Ret), "ret");
        assert_eq!(format!("{}", Token::Return), "return");
        assert_eq!(format!("{}", Token::While), "while");
        assert_eq!(format!("{}", Token::For), "for");
        assert_eq!(format!("{}", Token::In), "in");
        assert_eq!(format!("{}", Token::Loop), "loop");
        assert_eq!(format!("{}", Token::Break), "break");
        assert_eq!(format!("{}", Token::Continue), "continue");
        assert_eq!(format!("{}", Token::Match), "match");
    }

    #[test]
    fn test_display_keyword_logic() {
        assert_eq!(format!("{}", Token::And), "and");
        assert_eq!(format!("{}", Token::Or), "or");
        assert_eq!(format!("{}", Token::Not), "not");
        assert_eq!(format!("{}", Token::True), "true");
        assert_eq!(format!("{}", Token::False), "false");
    }

    #[test]
    fn test_display_keyword_types_and_data() {
        assert_eq!(format!("{}", Token::Struct), "struct");
        assert_eq!(format!("{}", Token::Enum), "enum");
        assert_eq!(format!("{}", Token::New), "new");
        assert_eq!(format!("{}", Token::Mut), "mut");
        assert_eq!(format!("{}", Token::Set), "set");
        assert_eq!(format!("{}", Token::Type), "type");
        assert_eq!(format!("{}", Token::Trait), "trait");
        assert_eq!(format!("{}", Token::Impl), "impl");
        assert_eq!(format!("{}", Token::As), "as");
        assert_eq!(format!("{}", Token::Null), "null");
        assert_eq!(format!("{}", Token::Sizeof), "sizeof");
        assert_eq!(format!("{}", Token::Todo), "todo");
    }

    #[test]
    fn test_display_keyword_module_system() {
        assert_eq!(format!("{}", Token::Pub), "pub");
        assert_eq!(format!("{}", Token::Use), "use");
        assert_eq!(format!("{}", Token::Mod), "mod");
        assert_eq!(format!("{}", Token::Extern), "extern");
        assert_eq!(format!("{}", Token::Module), "module");
        assert_eq!(format!("{}", Token::Version), "version");
        assert_eq!(format!("{}", Token::Summary), "summary");
        assert_eq!(format!("{}", Token::Exports), "exports");
        assert_eq!(format!("{}", Token::Depends), "depends");
        assert_eq!(format!("{}", Token::HeaderSep), "===");
    }

    #[test]
    fn test_display_keyword_contracts() {
        assert_eq!(format!("{}", Token::Pre), "pre");
        assert_eq!(format!("{}", Token::Post), "post");
        assert_eq!(format!("{}", Token::Where), "where");
        assert_eq!(format!("{}", Token::It), "it");
        assert_eq!(format!("{}", Token::Invariant), "invariant");
        assert_eq!(format!("{}", Token::Implies), "implies");
        assert_eq!(format!("{}", Token::Forall), "forall");
        assert_eq!(format!("{}", Token::Exists), "exists");
        assert_eq!(format!("{}", Token::Try), "try");
    }

    #[test]
    fn test_display_keyword_concurrency() {
        assert_eq!(format!("{}", Token::Spawn), "spawn");
        assert_eq!(format!("{}", Token::MutexType), "Mutex");
        assert_eq!(format!("{}", Token::ArcType), "Arc");
        assert_eq!(format!("{}", Token::AtomicType), "Atomic");
        assert_eq!(format!("{}", Token::ChannelKw), "channel");
        assert_eq!(format!("{}", Token::SenderType), "Sender");
        assert_eq!(format!("{}", Token::ReceiverType), "Receiver");
        assert_eq!(format!("{}", Token::RwLockType), "RwLock");
        assert_eq!(format!("{}", Token::BarrierType), "Barrier");
        assert_eq!(format!("{}", Token::CondvarType), "Condvar");
    }

    #[test]
    fn test_display_keyword_async() {
        assert_eq!(format!("{}", Token::Async), "async");
        assert_eq!(format!("{}", Token::Await), "await");
        assert_eq!(format!("{}", Token::FutureType), "Future");
        assert_eq!(format!("{}", Token::Select), "select");
        assert_eq!(format!("{}", Token::AsyncFileType), "AsyncFile");
        assert_eq!(format!("{}", Token::AsyncSocketType), "AsyncSocket");
        assert_eq!(format!("{}", Token::ThreadPoolType), "ThreadPool");
        assert_eq!(format!("{}", Token::ScopeType), "Scope");
    }

    #[test]
    fn test_display_type_keywords() {
        assert_eq!(format!("{}", Token::TyI32), "i32");
        assert_eq!(format!("{}", Token::TyI64), "i64");
        assert_eq!(format!("{}", Token::TyU32), "u32");
        assert_eq!(format!("{}", Token::TyU64), "u64");
        assert_eq!(format!("{}", Token::TyF64), "f64");
        assert_eq!(format!("{}", Token::TyBool), "bool");
        assert_eq!(format!("{}", Token::TyString), "String");
        assert_eq!(format!("{}", Token::TyChar), "char");
    }

    #[test]
    fn test_display_literals() {
        assert_eq!(format!("{}", Token::IntLit(42)), "42");
        assert_eq!(format!("{}", Token::IntLit(0)), "0");
        assert_eq!(format!("{}", Token::IntLit(-1)), "-1");
        assert_eq!(format!("{}", Token::FloatLit(1.5)), "1.5");
        assert_eq!(format!("{}", Token::StringLit("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", Token::CharLit('a')), "'a'");
        assert_eq!(format!("{}", Token::Ident("foo".to_string())), "foo");
    }

    #[test]
    fn test_display_symbols() {
        assert_eq!(format!("{}", Token::Colon), ":");
        assert_eq!(format!("{}", Token::ColonEq), ":=");
        assert_eq!(format!("{}", Token::ColonColon), "::");
        assert_eq!(format!("{}", Token::Arrow), "->");
        assert_eq!(format!("{}", Token::FatArrow), "=>");
        assert_eq!(format!("{}", Token::Underscore), "_");
        assert_eq!(format!("{}", Token::Dot), ".");
        assert_eq!(format!("{}", Token::DotDot), "..");
        assert_eq!(format!("{}", Token::DotDotLt), "..<");
        assert_eq!(format!("{}", Token::DotDotEq), "..=");
        assert_eq!(format!("{}", Token::Eq), "=");
        assert_eq!(format!("{}", Token::Semi), ";");
        assert_eq!(format!("{}", Token::Comma), ",");
    }

    #[test]
    fn test_display_delimiters() {
        assert_eq!(format!("{}", Token::LParen), "(");
        assert_eq!(format!("{}", Token::RParen), ")");
        assert_eq!(format!("{}", Token::LBrace), "{");
        assert_eq!(format!("{}", Token::RBrace), "}");
        assert_eq!(format!("{}", Token::LBracket), "[");
        assert_eq!(format!("{}", Token::RBracket), "]");
    }

    #[test]
    fn test_display_misc_symbols() {
        assert_eq!(format!("{}", Token::Ampersand), "&");
        assert_eq!(format!("{}", Token::At), "@");
        assert_eq!(format!("{}", Token::Question), "?");
        assert_eq!(format!("{}", Token::Pipe), "|");
    }

    #[test]
    fn test_display_arithmetic_operators() {
        assert_eq!(format!("{}", Token::Plus), "+");
        assert_eq!(format!("{}", Token::Minus), "-");
        assert_eq!(format!("{}", Token::Star), "*");
        assert_eq!(format!("{}", Token::Slash), "/");
        assert_eq!(format!("{}", Token::Percent), "%");
    }

    #[test]
    fn test_display_comparison_operators() {
        assert_eq!(format!("{}", Token::EqEq), "==");
        assert_eq!(format!("{}", Token::NotEq), "!=");
        assert_eq!(format!("{}", Token::Lt), "<");
        assert_eq!(format!("{}", Token::Gt), ">");
        assert_eq!(format!("{}", Token::LtEq), "<=");
        assert_eq!(format!("{}", Token::GtEq), ">=");
    }

    #[test]
    fn test_display_shift_operators() {
        assert_eq!(format!("{}", Token::LtLt), "<<");
        assert_eq!(format!("{}", Token::GtGt), ">>");
    }

    #[test]
    fn test_display_bitwise_operators() {
        assert_eq!(format!("{}", Token::Band), "band");
        assert_eq!(format!("{}", Token::Bor), "bor");
        assert_eq!(format!("{}", Token::Bxor), "bxor");
        assert_eq!(format!("{}", Token::Bnot), "bnot");
    }

    #[test]
    fn test_display_wrapping_operators() {
        assert_eq!(format!("{}", Token::PlusPercent), "+%");
        assert_eq!(format!("{}", Token::MinusPercent), "-%");
        assert_eq!(format!("{}", Token::StarPercent), "*%");
    }

    #[test]
    fn test_display_checked_operators() {
        assert_eq!(format!("{}", Token::PlusQuestion), "+?");
        assert_eq!(format!("{}", Token::MinusQuestion), "-?");
        assert_eq!(format!("{}", Token::StarQuestion), "*?");
    }

    #[test]
    fn test_display_saturating_operators() {
        assert_eq!(format!("{}", Token::PlusPipe), "+|");
        assert_eq!(format!("{}", Token::MinusPipe), "-|");
        assert_eq!(format!("{}", Token::StarPipe), "*|");
    }

    #[test]
    fn test_display_symbolic_logical_operators() {
        assert_eq!(format!("{}", Token::AmpAmp), "&&");
        assert_eq!(format!("{}", Token::PipePipe), "||");
        assert_eq!(format!("{}", Token::Bang), "!");
    }

    // === PartialEq tests ===

    #[test]
    fn test_token_equality_keywords() {
        assert_eq!(Token::Fn, Token::Fn);
        assert_ne!(Token::Fn, Token::Let);
        assert_ne!(Token::If, Token::Else);
    }

    #[test]
    fn test_token_equality_literals() {
        assert_eq!(Token::IntLit(42), Token::IntLit(42));
        assert_ne!(Token::IntLit(42), Token::IntLit(43));
        assert_eq!(Token::StringLit("a".to_string()), Token::StringLit("a".to_string()));
        assert_ne!(Token::StringLit("a".to_string()), Token::StringLit("b".to_string()));
        assert_eq!(Token::CharLit('x'), Token::CharLit('x'));
        assert_ne!(Token::CharLit('x'), Token::CharLit('y'));
    }

    #[test]
    fn test_token_equality_ident() {
        assert_eq!(Token::Ident("foo".to_string()), Token::Ident("foo".to_string()));
        assert_ne!(Token::Ident("foo".to_string()), Token::Ident("bar".to_string()));
    }

    // === Clone and Debug tests ===

    #[test]
    fn test_token_clone() {
        let t = Token::IntLit(99);
        let t2 = t.clone();
        assert_eq!(t, t2);

        let t3 = Token::StringLit("test".to_string());
        let t4 = t3.clone();
        assert_eq!(t3, t4);
    }

    #[test]
    fn test_token_debug() {
        let t = Token::Fn;
        let dbg = format!("{:?}", t);
        assert_eq!(dbg, "Fn");

        let t2 = Token::IntLit(5);
        let dbg2 = format!("{:?}", t2);
        assert!(dbg2.contains("IntLit"));
        assert!(dbg2.contains("5"));
    }

    // === Lexer tokenization integration tests for token.rs-specific features ===

    #[test]
    fn test_lex_scientific_notation() {
        let mut lexer = Token::lexer("3.14e10");
        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, Token::FloatLit(n) if (n - 3.14e10).abs() < 1e5));

        let mut lexer = Token::lexer("1e5");
        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, Token::FloatLit(n) if (n - 1e5).abs() < 1.0));

        let mut lexer = Token::lexer("6.022E23");
        let tok = lexer.next().unwrap().unwrap();
        assert!(matches!(tok, Token::FloatLit(n) if n > 6e23));
    }

    #[test]
    fn test_lex_string_escape_sequences() {
        let mut lexer = Token::lexer(r#""\n\t\r\\\"" "#);
        let tok = lexer.next().unwrap().unwrap();
        match tok {
            Token::StringLit(s) => {
                assert_eq!(s, "\n\t\r\\\"");
            }
            _ => panic!("expected StringLit"),
        }
    }

    #[test]
    fn test_lex_string_null_escape() {
        let mut lexer = Token::lexer(r#""\0""#);
        let tok = lexer.next().unwrap().unwrap();
        match tok {
            Token::StringLit(s) => assert_eq!(s, "\0"),
            _ => panic!("expected StringLit"),
        }
    }

    #[test]
    fn test_lex_char_literal() {
        let mut lexer = Token::lexer("'a'");
        let tok = lexer.next().unwrap().unwrap();
        assert_eq!(tok, Token::CharLit('a'));
    }

    #[test]
    fn test_lex_char_escape_sequences() {
        let cases = [
            (r"'\n'", '\n'),
            (r"'\t'", '\t'),
            (r"'\r'", '\r'),
            (r"'\\'", '\\'),
            (r"'\0'", '\0'),
        ];
        for (input, expected) in cases {
            let mut lexer = Token::lexer(input);
            let tok = lexer.next().unwrap().unwrap();
            assert_eq!(tok, Token::CharLit(expected), "failed for input: {input}");
        }
    }

    #[test]
    fn test_lex_range_operators() {
        let mut lexer = Token::lexer(".. ..< ..=");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::DotDot);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::DotDotLt);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::DotDotEq);
    }

    #[test]
    fn test_lex_wrapping_operators() {
        let mut lexer = Token::lexer("+% -% *%");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::PlusPercent);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::MinusPercent);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::StarPercent);
    }

    #[test]
    fn test_lex_checked_operators() {
        let mut lexer = Token::lexer("+? -? *?");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::PlusQuestion);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::MinusQuestion);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::StarQuestion);
    }

    #[test]
    fn test_lex_saturating_operators() {
        let mut lexer = Token::lexer("+| -| *|");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::PlusPipe);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::MinusPipe);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::StarPipe);
    }

    #[test]
    fn test_lex_shift_operators() {
        let mut lexer = Token::lexer("<< >>");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::LtLt);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::GtGt);
    }

    #[test]
    fn test_lex_bitwise_keywords() {
        let mut lexer = Token::lexer("band bor bxor bnot");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Band);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Bor);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Bxor);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Bnot);
    }

    #[test]
    fn test_lex_concurrency_keywords() {
        let mut lexer = Token::lexer("spawn Mutex Arc Atomic");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Spawn);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::MutexType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::ArcType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::AtomicType);
    }

    #[test]
    fn test_lex_channel_types() {
        let mut lexer = Token::lexer("channel Sender Receiver");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::ChannelKw);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::SenderType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::ReceiverType);
    }

    #[test]
    fn test_lex_sync_primitives() {
        let mut lexer = Token::lexer("RwLock Barrier Condvar");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::RwLockType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::BarrierType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::CondvarType);
    }

    #[test]
    fn test_lex_async_keywords() {
        let mut lexer = Token::lexer("async await Future select");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Async);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Await);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::FutureType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Select);
    }

    #[test]
    fn test_lex_async_io_types() {
        let mut lexer = Token::lexer("AsyncFile AsyncSocket ThreadPool Scope");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::AsyncFileType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::AsyncSocketType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::ThreadPoolType);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::ScopeType);
    }

    #[test]
    fn test_lex_contract_keywords() {
        let mut lexer = Token::lexer("pre post where it invariant implies forall exists");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Pre);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Post);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Where);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::It);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Invariant);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Implies);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Forall);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Exists);
    }

    #[test]
    fn test_lex_module_header() {
        let mut lexer = Token::lexer("module version summary exports depends ===");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Module);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Version);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Summary);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Exports);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Depends);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::HeaderSep);
    }

    #[test]
    fn test_lex_symbolic_logical() {
        let mut lexer = Token::lexer("&& || !");
        assert_eq!(lexer.next().unwrap().unwrap(), Token::AmpAmp);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::PipePipe);
        assert_eq!(lexer.next().unwrap().unwrap(), Token::Bang);
    }
}
