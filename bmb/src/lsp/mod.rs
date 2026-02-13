//! BMB Language Server Protocol implementation
//!
//! Provides IDE features:
//! - Diagnostics (type errors, parse errors)
//! - Hover (type information)
//! - Completion (keywords, built-ins)
//! - Formatting (v0.9.0)
//! - Go to Definition (v0.9.0)
//! - Find References (v0.9.0)

use std::collections::HashMap;
use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::ast::{Expr, Item, Program, Span};
use crate::error::CompileError;
use crate::lexer;
use crate::parser;
use crate::types::TypeChecker;

/// BMB Language keywords for completion
const BMB_KEYWORDS: &[&str] = &[
    "fn", "let", "mut", "if", "then", "else", "match", "for", "in", "while",
    "struct", "enum", "type", "pub", "use", "pre", "post", "where",
    "true", "false", "rec", "own", "ref", "move", "copy", "drop", "linear",
    "forall", "exists", "old", "ret", "low", "satisfies", "modifies",
    "invariant", "decreases",
];

/// BMB built-in functions for completion
const BMB_BUILTINS: &[(&str, &str)] = &[
    ("print", "print(x: i64) -> Unit"),
    ("println", "println(x: i64) -> Unit"),
    ("assert", "assert(cond: bool) -> Unit"),
    ("read_int", "read_int() -> i64"),
    ("abs", "abs(n: i64) -> i64"),
    ("min", "min(a: i64, b: i64) -> i64"),
    ("max", "max(a: i64, b: i64) -> i64"),
];

/// Symbol definition with location
#[derive(Debug, Clone)]
struct SymbolDef {
    name: String,
    kind: SymbolKind,
    span: Span,
    /// Type string for hover display (v0.50.25)
    type_str: Option<String>,
}

/// Local variable in a specific scope (v0.50.25)
#[derive(Debug, Clone)]
struct LocalVar {
    name: String,
    type_str: String,
    /// Span where the variable is defined
    def_span: Span,
    /// Span of the scope where this variable is visible
    scope_span: Span,
}

/// Symbol reference (usage)
#[derive(Debug, Clone)]
struct SymbolRef {
    name: String,
    span: Span,
}

/// Symbol kind for definition
#[derive(Debug, Clone, Copy, PartialEq)]
enum SymbolKind {
    Function,
    Struct,
    Enum,
    #[allow(dead_code)]
    Variable,
    Parameter,
    Trait,   // v0.20.1
    Method,  // v0.20.1
}

/// Document state
struct DocumentState {
    content: String,
    ast: Option<Program>,
    /// Symbol definitions in this document
    definitions: Vec<SymbolDef>,
    /// Symbol references in this document
    references: Vec<SymbolRef>,
    /// Local variables with their scopes (v0.50.25)
    locals: Vec<LocalVar>,
    #[allow(dead_code)]
    version: i32,
}

/// BMB Language Server Backend
pub struct Backend {
    client: Client,
    documents: RwLock<HashMap<Url, DocumentState>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Analyze document and publish diagnostics
    async fn analyze_document(&self, uri: &Url, content: &str, version: i32) {
        let diagnostics = self.get_diagnostics(uri, content);

        // Parse AST if successful for hover/completion
        let ast = self.try_parse(content);

        // Collect symbols from AST
        let (definitions, references, locals) = if let Some(ref ast) = ast {
            self.collect_symbols(ast)
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        // Store document state
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(uri.clone(), DocumentState {
                content: content.to_string(),
                ast,
                definitions,
                references,
                locals,
                version,
            });
        }

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;
    }

    /// Collect symbol definitions, references, and local variables from AST
    fn collect_symbols(&self, ast: &Program) -> (Vec<SymbolDef>, Vec<SymbolRef>, Vec<LocalVar>) {
        let mut definitions = Vec::new();
        let mut references = Vec::new();
        let mut locals = Vec::new();

        for item in &ast.items {
            match item {
                Item::FnDef(f) => {
                    // Function definition with signature
                    let params_str: Vec<String> = f.params.iter()
                        .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                        .collect();
                    let sig = format!("fn({}) -> {}", params_str.join(", "), format_type(&f.ret_ty.node));

                    definitions.push(SymbolDef {
                        name: f.name.node.clone(),
                        kind: SymbolKind::Function,
                        span: f.name.span,
                        type_str: Some(sig),
                    });

                    // Parameters as definitions with types (v0.50.25)
                    // Body span approximation: from first param to end of body
                    let fn_scope_span = f.body.span;

                    for param in &f.params {
                        definitions.push(SymbolDef {
                            name: param.name.node.clone(),
                            kind: SymbolKind::Parameter,
                            span: param.name.span,
                            type_str: Some(format_type(&param.ty.node)),
                        });

                        // Also add as local var for scope-based completion
                        locals.push(LocalVar {
                            name: param.name.node.clone(),
                            type_str: format_type(&param.ty.node),
                            def_span: param.name.span,
                            scope_span: fn_scope_span,
                        });
                    }

                    // Collect local variables from body (v0.50.25)
                    self.collect_locals(&f.body.node, fn_scope_span, &mut locals);

                    // Collect references from body
                    self.collect_expr_refs(&f.body.node, &mut references);

                    // Pre/post conditions
                    if let Some(pre) = &f.pre {
                        self.collect_expr_refs(&pre.node, &mut references);
                    }
                    if let Some(post) = &f.post {
                        self.collect_expr_refs(&post.node, &mut references);
                    }
                }
                Item::StructDef(s) => {
                    let fields_info = s.fields.iter()
                        .map(|f| format!("{}: {}", f.name.node, format_type(&f.ty.node)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    definitions.push(SymbolDef {
                        name: s.name.node.clone(),
                        kind: SymbolKind::Struct,
                        span: s.name.span,
                        type_str: Some(format!("struct {{ {} }}", fields_info)),
                    });
                }
                Item::EnumDef(e) => {
                    let variants = e.variants.iter()
                        .map(|v| v.name.node.as_str())
                        .collect::<Vec<_>>()
                        .join(" | ");
                    definitions.push(SymbolDef {
                        name: e.name.node.clone(),
                        kind: SymbolKind::Enum,
                        span: e.name.span,
                        type_str: Some(format!("enum {{ {} }}", variants)),
                    });
                }
                Item::Use(_) => {}
                // v0.13.0: Extern functions as function definitions
                Item::ExternFn(e) => {
                    let params_str: Vec<String> = e.params.iter()
                        .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                        .collect();
                    let sig = format!("extern fn({}) -> {}", params_str.join(", "), format_type(&e.ret_ty.node));
                    definitions.push(SymbolDef {
                        name: e.name.node.clone(),
                        kind: SymbolKind::Function,
                        span: e.name.span,
                        type_str: Some(sig),
                    });
                }
                // v0.20.1: Trait definitions
                Item::TraitDef(t) => {
                    definitions.push(SymbolDef {
                        name: t.name.node.clone(),
                        kind: SymbolKind::Trait,
                        span: t.name.span,
                        type_str: Some("trait".to_string()),
                    });
                }
                // v0.20.1: Impl blocks - register methods
                Item::ImplBlock(i) => {
                    for method in &i.methods {
                        let params_str: Vec<String> = method.params.iter()
                            .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                            .collect();
                        let sig = format!("fn({}) -> {}", params_str.join(", "), format_type(&method.ret_ty.node));
                        definitions.push(SymbolDef {
                            name: method.name.node.clone(),
                            kind: SymbolKind::Method,
                            span: method.name.span,
                            type_str: Some(sig),
                        });
                        self.collect_expr_refs(&method.body.node, &mut references);
                    }
                }
                // v0.50.6: Type aliases - register as type definitions
                Item::TypeAlias(_) => {}
            }
        }

        (definitions, references, locals)
    }

    /// Collect symbol references from expression
    fn collect_expr_refs(&self, expr: &Expr, refs: &mut Vec<SymbolRef>) {
        match expr {
            Expr::Var(_name) => {
                // This is a reference to a variable/function
                // Note: We can't easily get the span here from Expr::Var
                // For a more complete implementation, Expr::Var would need to be Spanned
            }
            Expr::Call { func: _, args, .. } => {
                // Function call is a reference (name-only, no span in current AST)
                for arg in args {
                    self.collect_expr_refs(&arg.node, refs);
                }
            }
            Expr::Let { value, body, .. } => {
                self.collect_expr_refs(&value.node, refs);
                self.collect_expr_refs(&body.node, refs);
            }
            // v0.60.21: Uninitialized let binding
            Expr::LetUninit { body, .. } => {
                self.collect_expr_refs(&body.node, refs);
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.collect_expr_refs(&cond.node, refs);
                self.collect_expr_refs(&then_branch.node, refs);
                self.collect_expr_refs(&else_branch.node, refs);
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expr_refs(&left.node, refs);
                self.collect_expr_refs(&right.node, refs);
            }
            Expr::Unary { expr, .. } => {
                self.collect_expr_refs(&expr.node, refs);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_expr_refs(&stmt.node, refs);
                }
            }
            // v0.37: Include invariant in refs collection
            Expr::While { cond, invariant, body } => {
                self.collect_expr_refs(&cond.node, refs);
                if let Some(inv) = invariant {
                    self.collect_expr_refs(&inv.node, refs);
                }
                self.collect_expr_refs(&body.node, refs);
            }
            Expr::For { iter, body, .. } => {
                self.collect_expr_refs(&iter.node, refs);
                self.collect_expr_refs(&body.node, refs);
            }
            Expr::Match { expr, arms } => {
                self.collect_expr_refs(&expr.node, refs);
                for arm in arms {
                    self.collect_expr_refs(&arm.body.node, refs);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_expr_refs(&receiver.node, refs);
                for arg in args {
                    self.collect_expr_refs(&arg.node, refs);
                }
            }
            Expr::FieldAccess { expr, .. } => {
                self.collect_expr_refs(&expr.node, refs);
            }
            // v0.43: Tuple field access
            Expr::TupleField { expr, .. } => {
                self.collect_expr_refs(&expr.node, refs);
            }
            Expr::Index { expr, index } => {
                self.collect_expr_refs(&expr.node, refs);
                self.collect_expr_refs(&index.node, refs);
            }
            // v0.51: Index assignment
            Expr::IndexAssign { array, index, value } => {
                self.collect_expr_refs(&array.node, refs);
                self.collect_expr_refs(&index.node, refs);
                self.collect_expr_refs(&value.node, refs);
            }
            // v0.51.23: Field assignment
            Expr::FieldAssign { object, value, .. } => {
                self.collect_expr_refs(&object.node, refs);
                self.collect_expr_refs(&value.node, refs);
            }
            // v0.60.21: Dereference assignment
            Expr::DerefAssign { ptr, value } => {
                self.collect_expr_refs(&ptr.node, refs);
                self.collect_expr_refs(&value.node, refs);
            }
            Expr::ArrayLit(elems) => {
                for elem in elems {
                    self.collect_expr_refs(&elem.node, refs);
                }
            }
            // v0.60.22: Array repeat
            Expr::ArrayRepeat { value, .. } => {
                self.collect_expr_refs(&value.node, refs);
            }
            // v0.42: Tuple expressions
            Expr::Tuple(elems) => {
                for elem in elems {
                    self.collect_expr_refs(&elem.node, refs);
                }
            }
            Expr::StructInit { fields, .. } => {
                for (_, value) in fields {
                    self.collect_expr_refs(&value.node, refs);
                }
            }
            Expr::Range { start, end, .. } => {
                self.collect_expr_refs(&start.node, refs);
                self.collect_expr_refs(&end.node, refs);
            }
            Expr::Assign { value, .. } => {
                self.collect_expr_refs(&value.node, refs);
            }
            Expr::Ref(inner) | Expr::RefMut(inner) | Expr::Deref(inner) => {
                self.collect_expr_refs(&inner.node, refs);
            }
            Expr::EnumVariant { args, .. } => {
                for arg in args {
                    self.collect_expr_refs(&arg.node, refs);
                }
            }
            Expr::StateRef { expr, .. } => {
                self.collect_expr_refs(&expr.node, refs);
            }
            // Literals and simple expressions
            _ => {}
        }
    }

    /// Collect local variables from expressions (v0.50.25)
    fn collect_locals(&self, expr: &Expr, scope_span: Span, locals: &mut Vec<LocalVar>) {
        match expr {
            Expr::Let { name, ty, value, body, .. } => {
                // Get type string - either explicit or inferred placeholder
                let type_str = ty.as_ref()
                    .map(|t| format_type(&t.node))
                    .unwrap_or_else(|| "inferred".to_string());

                // The let variable is visible from its definition to end of body
                let body_span = body.span;
                locals.push(LocalVar {
                    name: name.clone(),
                    type_str,
                    def_span: value.span, // Use value span as approximate def location
                    scope_span: body_span,
                });

                // Recurse into value and body
                self.collect_locals(&value.node, scope_span, locals);
                self.collect_locals(&body.node, body_span, locals);
            }
            // v0.60.21: Uninitialized let binding
            Expr::LetUninit { name, ty, body, .. } => {
                let type_str = format_type(&ty.node);
                let body_span = body.span;
                locals.push(LocalVar {
                    name: name.clone(),
                    type_str,
                    def_span: ty.span, // Use type annotation span as def location
                    scope_span: body_span,
                });

                // Recurse into body only (no value for uninitialized binding)
                self.collect_locals(&body.node, body_span, locals);
            }
            Expr::Block(stmts) => {
                for stmt in stmts {
                    self.collect_locals(&stmt.node, scope_span, locals);
                }
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.collect_locals(&cond.node, scope_span, locals);
                self.collect_locals(&then_branch.node, then_branch.span, locals);
                self.collect_locals(&else_branch.node, else_branch.span, locals);
            }
            Expr::Match { expr, arms } => {
                self.collect_locals(&expr.node, scope_span, locals);
                for arm in arms {
                    // Pattern bindings could be added here if needed
                    self.collect_locals(&arm.body.node, arm.body.span, locals);
                }
            }
            Expr::For { var, iter, body } => {
                // Loop variable is visible in body
                locals.push(LocalVar {
                    name: var.clone(),
                    type_str: "inferred".to_string(), // For loop var type is inferred
                    def_span: iter.span,
                    scope_span: body.span,
                });
                self.collect_locals(&iter.node, scope_span, locals);
                self.collect_locals(&body.node, body.span, locals);
            }
            Expr::While { cond, body, invariant } => {
                self.collect_locals(&cond.node, scope_span, locals);
                self.collect_locals(&body.node, body.span, locals);
                if let Some(inv) = invariant {
                    self.collect_locals(&inv.node, scope_span, locals);
                }
            }
            Expr::Closure { params, body, .. } => {
                let closure_scope = body.span;
                for param in params {
                    let type_str = param.ty.as_ref()
                        .map(|t| format_type(&t.node))
                        .unwrap_or_else(|| "inferred".to_string());
                    locals.push(LocalVar {
                        name: param.name.node.clone(),
                        type_str,
                        def_span: param.name.span,
                        scope_span: closure_scope,
                    });
                }
                self.collect_locals(&body.node, closure_scope, locals);
            }
            // Recurse into expressions that may contain let bindings
            Expr::Call { args, .. } => {
                for arg in args {
                    self.collect_locals(&arg.node, scope_span, locals);
                }
            }
            Expr::MethodCall { receiver, args, .. } => {
                self.collect_locals(&receiver.node, scope_span, locals);
                for arg in args {
                    self.collect_locals(&arg.node, scope_span, locals);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_locals(&left.node, scope_span, locals);
                self.collect_locals(&right.node, scope_span, locals);
            }
            Expr::Unary { expr, .. } => {
                self.collect_locals(&expr.node, scope_span, locals);
            }
            Expr::Index { expr, index } => {
                self.collect_locals(&expr.node, scope_span, locals);
                self.collect_locals(&index.node, scope_span, locals);
            }
            Expr::IndexAssign { array, index, value } => {
                self.collect_locals(&array.node, scope_span, locals);
                self.collect_locals(&index.node, scope_span, locals);
                self.collect_locals(&value.node, scope_span, locals);
            }
            // v0.51.23: Field assignment
            Expr::FieldAssign { object, value, .. } => {
                self.collect_locals(&object.node, scope_span, locals);
                self.collect_locals(&value.node, scope_span, locals);
            }
            // v0.60.21: Dereference assignment
            Expr::DerefAssign { ptr, value } => {
                self.collect_locals(&ptr.node, scope_span, locals);
                self.collect_locals(&value.node, scope_span, locals);
            }
            Expr::FieldAccess { expr, .. } | Expr::TupleField { expr, .. } => {
                self.collect_locals(&expr.node, scope_span, locals);
            }
            Expr::ArrayLit(elems) | Expr::Tuple(elems) => {
                for elem in elems {
                    self.collect_locals(&elem.node, scope_span, locals);
                }
            }
            // v0.60.22: Array repeat
            Expr::ArrayRepeat { value, .. } => {
                self.collect_locals(&value.node, scope_span, locals);
            }
            Expr::StructInit { fields, .. } => {
                for (_, value) in fields {
                    self.collect_locals(&value.node, scope_span, locals);
                }
            }
            Expr::Range { start, end, .. } => {
                self.collect_locals(&start.node, scope_span, locals);
                self.collect_locals(&end.node, scope_span, locals);
            }
            Expr::EnumVariant { args, .. } => {
                for arg in args {
                    self.collect_locals(&arg.node, scope_span, locals);
                }
            }
            Expr::Ref(inner) | Expr::RefMut(inner) | Expr::Deref(inner) => {
                self.collect_locals(&inner.node, scope_span, locals);
            }
            Expr::Loop { body } => {
                self.collect_locals(&body.node, body.span, locals);
            }
            Expr::Return { value } | Expr::Break { value } => {
                if let Some(v) = value {
                    self.collect_locals(&v.node, scope_span, locals);
                }
            }
            Expr::Assign { value, .. } => {
                self.collect_locals(&value.node, scope_span, locals);
            }
            Expr::Cast { expr, .. } => {
                self.collect_locals(&expr.node, scope_span, locals);
            }
            Expr::Forall { body, .. } | Expr::Exists { body, .. } => {
                self.collect_locals(&body.node, scope_span, locals);
            }
            // Terminals - no recursion needed
            _ => {}
        }
    }

    /// Get local variables visible at a given byte offset (v0.50.25)
    fn get_locals_at_offset<'a>(&self, locals: &'a [LocalVar], offset: usize) -> Vec<&'a LocalVar> {
        locals.iter()
            .filter(|local| {
                // Variable is visible if offset is within its scope and after its definition
                offset >= local.def_span.start && offset <= local.scope_span.end
            })
            .collect()
    }

    /// Get diagnostics from lexer, parser, and type checker
    fn get_diagnostics(&self, uri: &Url, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let filename = uri.path();

        // Try to tokenize
        let tokens = match lexer::tokenize(content) {
            Ok(tokens) => tokens,
            Err(e) => {
                diagnostics.push(self.error_to_diagnostic(&e, content));
                return diagnostics;
            }
        };

        // Try to parse
        let ast = match parser::parse(filename, content, tokens) {
            Ok(ast) => ast,
            Err(e) => {
                diagnostics.push(self.error_to_diagnostic(&e, content));
                return diagnostics;
            }
        };

        // Type check
        let mut checker = TypeChecker::new();
        if let Err(e) = checker.check_program(&ast) {
            diagnostics.push(self.error_to_diagnostic(&e, content));
        }

        diagnostics
    }

    /// Try to parse content, returning AST if successful
    fn try_parse(&self, content: &str) -> Option<Program> {
        let tokens = lexer::tokenize(content).ok()?;
        parser::parse("<lsp>", content, tokens).ok()
    }

    /// Convert CompileError to LSP Diagnostic
    fn error_to_diagnostic(&self, error: &CompileError, content: &str) -> Diagnostic {
        let (range, severity) = if let Some(span) = error.span() {
            (self.span_to_range(span, content), DiagnosticSeverity::ERROR)
        } else {
            (Range::default(), DiagnosticSeverity::ERROR)
        };

        let source = match error {
            CompileError::Lexer { .. } => "bmb-lexer",
            CompileError::Parser { .. } => "bmb-parser",
            CompileError::Type { .. } => "bmb-types",
            _ => "bmb",
        };

        Diagnostic {
            range,
            severity: Some(severity),
            source: Some(source.to_string()),
            message: error.message().to_string(),
            ..Default::default()
        }
    }

    /// Convert Span (byte offset) to LSP Range (line/character)
    fn span_to_range(&self, span: Span, content: &str) -> Range {
        let start = self.offset_to_position(span.start, content);
        let end = self.offset_to_position(span.end, content);
        Range { start, end }
    }

    /// Convert byte offset to LSP Position
    fn offset_to_position(&self, offset: usize, content: &str) -> Position {
        let mut line = 0u32;
        let mut col = 0u32;

        for (i, c) in content.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        Position::new(line, col)
    }

    /// Convert LSP Position to byte offset
    fn position_to_offset(&self, position: Position, content: &str) -> usize {
        let mut current_line = 0u32;
        let mut current_col = 0u32;

        for (i, c) in content.char_indices() {
            if current_line == position.line && current_col == position.character {
                return i;
            }
            if c == '\n' {
                if current_line == position.line {
                    return i;
                }
                current_line += 1;
                current_col = 0;
            } else {
                current_col += 1;
            }
        }

        content.len()
    }

    /// Get word at position for hover
    fn get_word_at_position(&self, content: &str, position: Position) -> Option<String> {
        let offset = self.position_to_offset(position, content);

        // Find word boundaries
        let bytes = content.as_bytes();
        let mut start = offset;
        let mut end = offset;

        // Walk back to find start of word
        while start > 0 && Self::is_ident_char(bytes[start - 1] as char) {
            start -= 1;
        }

        // Walk forward to find end of word
        while end < bytes.len() && Self::is_ident_char(bytes[end] as char) {
            end += 1;
        }

        if start < end {
            Some(content[start..end].to_string())
        } else {
            None
        }
    }

    fn is_ident_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                // v0.9.0: Formatting support
                document_formatting_provider: Some(OneOf::Left(true)),
                // v0.9.0: Go to definition
                definition_provider: Some(OneOf::Left(true)),
                // v0.9.0: Find references
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "bmb-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "BMB Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        self.analyze_document(&uri, &content, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        // Full sync - take the whole content
        if let Some(change) = params.content_changes.into_iter().next() {
            self.analyze_document(&uri, &change.text, version).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().unwrap();
        docs.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().unwrap();
        let doc = match docs.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let word = match self.get_word_at_position(&doc.content, position) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Check if it's a keyword
        if BMB_KEYWORDS.contains(&word.as_str()) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Keyword**: `{}`", word),
                }),
                range: None,
            }));
        }

        // Check if it's a built-in function
        for (name, sig) in BMB_BUILTINS {
            if *name == word {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Built-in**: `{}`", sig),
                    }),
                    range: None,
                }));
            }
        }

        // v0.50.25: Check local variables at cursor position
        let offset = self.position_to_offset(position, &doc.content);
        let visible_locals = self.get_locals_at_offset(&doc.locals, offset);
        for local in visible_locals {
            if local.name == word {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**Local variable**: `{}: {}`", local.name, local.type_str),
                    }),
                    range: None,
                }));
            }
        }

        // Check definitions (functions, structs, enums, etc.) with type info
        for def in &doc.definitions {
            if def.name == word
                && let Some(type_str) = &def.type_str {
                    let kind_str = match def.kind {
                        SymbolKind::Function => "Function",
                        SymbolKind::Struct => "Struct",
                        SymbolKind::Enum => "Enum",
                        SymbolKind::Variable => "Variable",
                        SymbolKind::Parameter => "Parameter",
                        SymbolKind::Trait => "Trait",
                        SymbolKind::Method => "Method",
                    };
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("**{}**: `{}`\n\n```bmb\n{}\n```", kind_str, def.name, type_str),
                        }),
                        range: None,
                    }));
                }
        }

        // Fallback: Check AST for user-defined symbols (legacy, for items without type_str)
        if let Some(ast) = &doc.ast {
            for item in &ast.items {
                match item {
                    crate::ast::Item::FnDef(f) if f.name.node == word => {
                        let params: Vec<String> = f.params.iter()
                            .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                            .collect();
                        let sig = format!("fn {}({}) -> {}",
                            f.name.node,
                            params.join(", "),
                            format_type(&f.ret_ty.node)
                        );
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("```bmb\n{}\n```", sig),
                            }),
                            range: None,
                        }));
                    }
                    crate::ast::Item::StructDef(s) if s.name.node == word => {
                        let fields: Vec<String> = s.fields.iter()
                            .map(|f| format!("  {}: {}", f.name.node, format_type(&f.ty.node)))
                            .collect();
                        let def = format!("struct {} {{\n{}\n}}", s.name.node, fields.join(",\n"));
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("```bmb\n{}\n```", def),
                            }),
                            range: None,
                        }));
                    }
                    crate::ast::Item::EnumDef(e) if e.name.node == word => {
                        let variants: Vec<String> = e.variants.iter()
                            .map(|v| format!("  {}", v.name.node))
                            .collect();
                        let def = format!("enum {} {{\n{}\n}}", e.name.node, variants.join(",\n"));
                        return Ok(Some(Hover {
                            contents: HoverContents::Markup(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("```bmb\n{}\n```", def),
                            }),
                            range: None,
                        }));
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let mut items = Vec::new();

        // Add keywords
        for keyword in BMB_KEYWORDS {
            items.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword".to_string()),
                ..Default::default()
            });
        }

        // Add built-in functions
        for (name, sig) in BMB_BUILTINS {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(sig.to_string()),
                insert_text: Some(format!("{}($0)", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Add user-defined symbols and local variables
        let docs = self.documents.read().unwrap();
        if let Some(doc) = docs.get(uri) {
            // v0.50.25: Add local variables visible at cursor position
            let offset = self.position_to_offset(position, &doc.content);
            let visible_locals = self.get_locals_at_offset(&doc.locals, offset);

            // Track added names to avoid duplicates
            let mut added_names: std::collections::HashSet<String> = std::collections::HashSet::new();

            for local in visible_locals {
                if added_names.insert(local.name.clone()) {
                    items.push(CompletionItem {
                        label: local.name.clone(),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: Some(local.type_str.clone()),
                        // Sort local variables higher (prefix with '!')
                        sort_text: Some(format!("!0{}", local.name)),
                        ..Default::default()
                    });
                }
            }

            // Add AST items
            if let Some(ast) = &doc.ast {
                for item in &ast.items {
                    match item {
                        crate::ast::Item::FnDef(f) => {
                            if added_names.insert(f.name.node.clone()) {
                                let params_snippet: Vec<String> = f.params.iter()
                                    .enumerate()
                                    .map(|(i, p)| format!("${{{}:{}}}", i + 1, p.name.node))
                                    .collect();
                                let params_display: Vec<String> = f.params.iter()
                                    .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                                    .collect();
                                items.push(CompletionItem {
                                    label: f.name.node.clone(),
                                    kind: Some(CompletionItemKind::FUNCTION),
                                    detail: Some(format!("fn({}) -> {}", params_display.join(", "), format_type(&f.ret_ty.node))),
                                    insert_text: Some(format!("{}({})", f.name.node, params_snippet.join(", "))),
                                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                                    sort_text: Some(format!("!1{}", f.name.node)),
                                    ..Default::default()
                                });
                            }
                        }
                        crate::ast::Item::StructDef(s) => {
                            if added_names.insert(s.name.node.clone()) {
                                items.push(CompletionItem {
                                    label: s.name.node.clone(),
                                    kind: Some(CompletionItemKind::STRUCT),
                                    detail: Some("struct".to_string()),
                                    sort_text: Some(format!("!2{}", s.name.node)),
                                    ..Default::default()
                                });
                            }
                        }
                        crate::ast::Item::EnumDef(e) => {
                            if added_names.insert(e.name.node.clone()) {
                                items.push(CompletionItem {
                                    label: e.name.node.clone(),
                                    kind: Some(CompletionItemKind::ENUM),
                                    detail: Some("enum".to_string()),
                                    sort_text: Some(format!("!2{}", e.name.node)),
                                    ..Default::default()
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    /// v0.9.0: Format document
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = &params.text_document.uri;

        let docs = self.documents.read().unwrap();
        let doc = match docs.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Only format if we have a valid AST
        let ast = match &doc.ast {
            Some(ast) => ast,
            None => return Ok(None),
        };

        // Format the AST
        let formatted = format_program(ast);

        // Create a text edit that replaces the entire document
        let lines: Vec<&str> = doc.content.lines().collect();
        let last_line = lines.len().saturating_sub(1) as u32;
        let last_col = lines.last().map(|l| l.len() as u32).unwrap_or(0);

        let edit = TextEdit {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(last_line, last_col),
            },
            new_text: formatted,
        };

        Ok(Some(vec![edit]))
    }

    /// v0.9.0: Go to definition
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.read().unwrap();
        let doc = match docs.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = match self.get_word_at_position(&doc.content, position) {
            Some(w) => w,
            None => return Ok(None),
        };

        // Search for definition
        for def in &doc.definitions {
            if def.name == word {
                let range = self.span_to_range(def.span, &doc.content);
                return Ok(Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range,
                })));
            }
        }

        Ok(None)
    }

    /// v0.9.0: Find all references
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.read().unwrap();
        let doc = match docs.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the word at cursor position
        let word = match self.get_word_at_position(&doc.content, position) {
            Some(w) => w,
            None => return Ok(None),
        };

        let mut locations = Vec::new();

        // Include definition if include_declaration is true
        if params.context.include_declaration {
            for def in &doc.definitions {
                if def.name == word {
                    locations.push(Location {
                        uri: uri.clone(),
                        range: self.span_to_range(def.span, &doc.content),
                    });
                }
            }
        }

        // Add all references
        for reference in &doc.references {
            if reference.name == word {
                locations.push(Location {
                    uri: uri.clone(),
                    range: self.span_to_range(reference.span, &doc.content),
                });
            }
        }

        if locations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(locations))
        }
    }
}

/// Format a BMB program to source code (v0.9.0)
fn format_program(program: &Program) -> String {
    use crate::ast::Visibility;

    let mut output = String::new();

    for (i, item) in program.items.iter().enumerate() {
        if i > 0 {
            output.push_str("\n\n");
        }

        match item {
            Item::FnDef(fn_def) => {
                output.push_str(&format_fn_def(fn_def));
            }
            Item::StructDef(s) => {
                if s.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("struct {} {{\n", s.name.node));
                for field in &s.fields {
                    output.push_str(&format!("    {}: {},\n", field.name.node, format_type(&field.ty.node)));
                }
                output.push('}');
            }
            Item::EnumDef(e) => {
                if e.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("enum {} {{\n", e.name.node));
                for variant in &e.variants {
                    output.push_str(&format!("    {},\n", variant.name.node));
                }
                output.push('}');
            }
            Item::Use(u) => {
                let path_str: Vec<_> = u.path.iter().map(|s| s.node.as_str()).collect();
                output.push_str(&format!("use {};", path_str.join("::")));
            }
            // v0.13.0: Format extern function declarations
            Item::ExternFn(e) => {
                if e.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("extern fn {}(", e.name.node));
                let params: Vec<_> = e.params.iter()
                    .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                    .collect();
                output.push_str(&params.join(", "));
                output.push_str(&format!(") -> {};", format_type(&e.ret_ty.node)));
            }
            // v0.20.1: Format trait definitions
            Item::TraitDef(t) => {
                if t.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("trait {} {{\n", t.name.node));
                for method in &t.methods {
                    let params: Vec<_> = method.params.iter()
                        .map(|p| format!("{}: {}", p.name.node, format_type(&p.ty.node)))
                        .collect();
                    output.push_str(&format!("    fn {}({}) -> {};\n",
                        method.name.node, params.join(", "), format_type(&method.ret_ty.node)));
                }
                output.push('}');
            }
            // v0.20.1: Format impl blocks
            Item::ImplBlock(i) => {
                output.push_str(&format!("impl {} for {} {{\n", i.trait_name.node, format_type(&i.target_type.node)));
                for method in &i.methods {
                    output.push_str(&format!("    {}\n", format_fn_def(method).trim()));
                }
                output.push('}');
            }
            // v0.50.6: Format type aliases
            Item::TypeAlias(t) => {
                if t.visibility == Visibility::Public {
                    output.push_str("pub ");
                }
                output.push_str(&format!("type {} = {};", t.name.node, format_type(&t.target.node)));
            }
        }
    }

    output.push('\n');
    output
}

fn format_fn_def(fn_def: &crate::ast::FnDef) -> String {
    use crate::ast::Visibility;

    let mut s = String::new();

    if fn_def.visibility == Visibility::Public {
        s.push_str("pub ");
    }

    // Function signature
    s.push_str(&format!("fn {}(", fn_def.name.node));

    for (i, param) in fn_def.params.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(&format!("{}: {}", param.name.node, format_type(&param.ty.node)));
    }

    s.push_str(&format!(") -> {}", format_type(&fn_def.ret_ty.node)));

    // Contracts
    if let Some(pre) = &fn_def.pre {
        s.push_str(&format!("\n  pre {}", format_expr(&pre.node)));
    }

    if let Some(post) = &fn_def.post {
        s.push_str(&format!("\n  post {}", format_expr(&post.node)));
    }

    // Body
    s.push_str(&format!("\n= {};", format_expr(&fn_def.body.node)));

    s
}

fn format_type(ty: &crate::ast::Type) -> String {
    use crate::ast::Type;

    match ty {
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        // v0.38: Unsigned types
        Type::U32 => "u32".to_string(),
        Type::U64 => "u64".to_string(),
        Type::F64 => "f64".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "String".to_string(),
        // v0.64: Char type
        Type::Char => "char".to_string(),
        Type::Unit => "()".to_string(),
        Type::Range(elem) => format!("Range<{}>", format_type(elem)),
        Type::Named(name) => name.clone(),
        // v0.13.1: Type variable
        Type::TypeVar(name) => name.clone(),
        // v0.13.1: Generic type
        Type::Generic { name, type_args } => {
            let args_str = type_args.iter()
                .map(|t| format_type(t))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", name, args_str)
        }
        Type::Struct { name, .. } => name.clone(),
        Type::Enum { name, .. } => name.clone(),
        Type::Array(elem, size) => format!("[{}; {}]", format_type(elem), size),
        Type::Ref(inner) => format!("&{}", format_type(inner)),
        Type::RefMut(inner) => format!("&mut {}", format_type(inner)),
        Type::Refined { base, constraints } => {
            let constraint_str = constraints.iter()
                .map(|c| format_expr(&c.node))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}{{{}}}", format_type(base), constraint_str)
        }
        // v0.20.0: Fn type
        Type::Fn { params, ret } => {
            let params_str = params.iter()
                .map(|p| format_type(p))
                .collect::<Vec<_>>()
                .join(", ");
            format!("fn({}) -> {}", params_str, format_type(ret))
        }
        // v0.31: Never type
        Type::Never => "!".to_string(),
        // v0.37: Nullable type
        Type::Nullable(inner) => format!("{}?", format_type(inner)),
        // v0.42: Tuple type
        Type::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|t| format_type(t)).collect();
            format!("({})", elems_str.join(", "))
        }
        // v0.51.37: Pointer type
        Type::Ptr(inner) => format!("*{}", format_type(inner)),
        // v0.70: Thread type
        Type::Thread(inner) => format!("Thread<{}>", format_type(inner)),
        // v0.71: Mutex type
        Type::Mutex(inner) => format!("Mutex<{}>", format_type(inner)),
        // v0.72: Arc and Atomic types
        Type::Arc(inner) => format!("Arc<{}>", format_type(inner)),
        Type::Atomic(inner) => format!("Atomic<{}>", format_type(inner)),
        // v0.73: Sender and Receiver types
        Type::Sender(inner) => format!("Sender<{}>", format_type(inner)),
        Type::Receiver(inner) => format!("Receiver<{}>", format_type(inner)),
        // v0.74: RwLock, Barrier, Condvar
        Type::RwLock(inner) => format!("RwLock<{}>", format_type(inner)),
        Type::Barrier => "Barrier".to_string(),
        Type::Condvar => "Condvar".to_string(),
        // v0.75: Future type
        Type::Future(inner) => format!("Future<{}>", format_type(inner)),
        // v0.83: AsyncFile type
        Type::AsyncFile => "AsyncFile".to_string(),
        // v0.83.1: AsyncSocket type
        Type::AsyncSocket => "AsyncSocket".to_string(),
        // v0.84: ThreadPool type
        Type::ThreadPool => "ThreadPool".to_string(),
        // v0.85: Scope type
        Type::Scope => "Scope".to_string(),
    }
}

fn format_expr(expr: &Expr) -> String {
    use crate::ast::{BinOp, UnOp};

    match expr {
        Expr::IntLit(n) => n.to_string(),
        Expr::FloatLit(f) => f.to_string(),
        Expr::BoolLit(b) => b.to_string(),
        Expr::StringLit(s) => format!("\"{}\"", s),
        // v0.64: Character literal
        Expr::CharLit(c) => format!("'{}'", c.escape_default()),
        Expr::Unit => "()".to_string(),
        // v0.51.40: Null pointer literal
        Expr::Null => "null".to_string(),
        // v0.51.41: Sizeof expression
        Expr::Sizeof { ty } => format!("sizeof<{}>()", format_type(&ty.node)),
        // v0.70: Spawn expression
        Expr::Spawn { body } => format!("spawn {{ {} }}", format_expr(&body.node)),
        // v0.72: Atomic creation expression
        Expr::AtomicNew { value } => format!("Atomic::new({})", format_expr(&value.node)),
        // v0.71: Mutex creation expression
        Expr::MutexNew { value } => format!("Mutex::new({})", format_expr(&value.node)),
        // v0.73: Channel creation expression
        Expr::ChannelNew { elem_ty, capacity } => format!(
            "channel<{}>({})",
            format_type(&elem_ty.node),
            format_expr(&capacity.node)
        ),
        Expr::Var(name) => name.clone(),
        Expr::Ret => "ret".to_string(),
        Expr::It => "it".to_string(),

        Expr::Binary { op, left, right } => {
            let op_str = match op {
                BinOp::Add => "+",
                BinOp::Sub => "-",
                BinOp::Mul => "*",
                BinOp::Div => "/",
                BinOp::Mod => "%",
                // v0.37: Wrapping arithmetic
                BinOp::AddWrap => "+%",
                BinOp::SubWrap => "-%",
                BinOp::MulWrap => "*%",
                // v0.38: Checked arithmetic
                BinOp::AddChecked => "+?",
                BinOp::SubChecked => "-?",
                BinOp::MulChecked => "*?",
                // v0.38: Saturating arithmetic
                BinOp::AddSat => "+|",
                BinOp::SubSat => "-|",
                BinOp::MulSat => "*|",
                BinOp::Eq => "==",
                BinOp::Ne => "!=",
                BinOp::Lt => "<",
                BinOp::Le => "<=",
                BinOp::Gt => ">",
                BinOp::Ge => ">=",
                BinOp::And => "and",
                BinOp::Or => "or",
                // v0.32: Shift operators
                BinOp::Shl => "<<",
                BinOp::Shr => ">>",
                // v0.36: Bitwise operators
                BinOp::Band => "band",
                BinOp::Bor => "bor",
                BinOp::Bxor => "bxor",
                // v0.36: Logical implication
                BinOp::Implies => "implies",
            };
            format!("{} {} {}", format_expr(&left.node), op_str, format_expr(&right.node))
        }

        Expr::Unary { op, expr } => {
            let op_str = match op {
                UnOp::Neg => "-",
                UnOp::Not => "not ",
                // v0.36: Bitwise not
                UnOp::Bnot => "bnot ",
            };
            format!("{}{}", op_str, format_expr(&expr.node))
        }

        Expr::If { cond, then_branch, else_branch } => {
            format!(
                "if {} then {} else {}",
                format_expr(&cond.node),
                format_expr(&then_branch.node),
                format_expr(&else_branch.node)
            )
        }

        Expr::Let { name, mutable, ty, value, body } => {
            let mut_str = if *mutable { "mut " } else { "" };
            let ty_str = ty.as_ref().map(|t| format!(": {}", format_type(&t.node))).unwrap_or_default();
            format!(
                "let {}{}{} = {};\n    {}",
                mut_str,
                name,
                ty_str,
                format_expr(&value.node),
                format_expr(&body.node)
            )
        }

        // v0.60.21: Uninitialized let binding
        Expr::LetUninit { name, mutable, ty, body } => {
            let mut_str = if *mutable { "mut " } else { "" };
            format!(
                "let {}{}: {};\n    {}",
                mut_str,
                name,
                format_type(&ty.node),
                format_expr(&body.node)
            )
        }

        Expr::Call { func, args } => {
            let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
            format!("{}({})", func, args_str.join(", "))
        }

        Expr::MethodCall { receiver, method, args } => {
            let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
            format!("{}.{}({})", format_expr(&receiver.node), method, args_str.join(", "))
        }

        Expr::Index { expr: arr, index } => {
            format!("{}[{}]", format_expr(&arr.node), format_expr(&index.node))
        }

        Expr::ArrayLit(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|e| format_expr(&e.node)).collect();
            format!("[{}]", elems_str.join(", "))
        }

        // v0.60.22: Array repeat
        Expr::ArrayRepeat { value, count } => {
            format!("[{}; {}]", format_expr(&value.node), count)
        }

        // v0.42: Tuple expressions
        Expr::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|e| format_expr(&e.node)).collect();
            if elems.len() == 1 {
                format!("({},)", elems_str.join(", "))
            } else {
                format!("({})", elems_str.join(", "))
            }
        }

        Expr::StructInit { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, v)| format!("{}: {}", n.node, format_expr(&v.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
        }

        Expr::FieldAccess { expr, field } => {
            format!("{}.{}", format_expr(&expr.node), field.node)
        }

        // v0.51.23: Field assignment
        Expr::FieldAssign { object, field, value } => {
            format!("{}.{} = {}", format_expr(&object.node), field.node, format_expr(&value.node))
        }

        // v0.60.21: Dereference assignment
        Expr::DerefAssign { ptr, value } => {
            format!("*{} = {}", format_expr(&ptr.node), format_expr(&value.node))
        }

        // v0.43: Tuple field access
        Expr::TupleField { expr, index } => {
            format!("{}.{}", format_expr(&expr.node), index)
        }

        Expr::Match { expr, arms } => {
            let arms_str: Vec<_> = arms.iter()
                .map(|arm| format!("{} => {}", format_pattern(&arm.pattern.node), format_expr(&arm.body.node)))
                .collect();
            format!("match {} {{ {} }}", format_expr(&expr.node), arms_str.join(", "))
        }

        Expr::Block(stmts) => {
            if stmts.is_empty() {
                "{}".to_string()
            } else {
                let stmts_str: Vec<_> = stmts.iter().map(|s| format_expr(&s.node)).collect();
                format!("{{ {} }}", stmts_str.join("; "))
            }
        }

        Expr::Assign { name, value } => {
            format!("{} = {}", name, format_expr(&value.node))
        }

        // v0.37: Include invariant in format if present
        Expr::While { cond, invariant, body } => {
            match invariant {
                Some(inv) => format!(
                    "while {} invariant {} {{ {} }}",
                    format_expr(&cond.node),
                    format_expr(&inv.node),
                    format_expr(&body.node)
                ),
                None => format!(
                    "while {} {{ {} }}",
                    format_expr(&cond.node),
                    format_expr(&body.node)
                ),
            }
        }

        Expr::For { var, iter, body } => {
            format!(
                "for {} in {} {{ {} }}",
                var,
                format_expr(&iter.node),
                format_expr(&body.node)
            )
        }

        Expr::Range { start, end, kind } => {
            let op = match kind {
                crate::ast::RangeKind::Exclusive => "..<",
                crate::ast::RangeKind::Inclusive => "..=",
            };
            format!("{}{}{}", format_expr(&start.node), op, format_expr(&end.node))
        }

        Expr::EnumVariant { enum_name, variant, args } => {
            if args.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let args_str: Vec<_> = args.iter().map(|a| format_expr(&a.node)).collect();
                format!("{}::{}({})", enum_name, variant, args_str.join(", "))
            }
        }

        Expr::Ref(inner) => {
            format!("&{}", format_expr(&inner.node))
        }

        Expr::RefMut(inner) => {
            format!("&mut {}", format_expr(&inner.node))
        }

        Expr::Deref(inner) => {
            format!("*{}", format_expr(&inner.node))
        }

        Expr::StateRef { expr, state } => {
            format!("{}{}", format_expr(&expr.node), state)
        }

        // v0.20.0: Closure expressions
        Expr::Closure { params, ret_ty, body } => {
            let params_str = params
                .iter()
                .map(|p| {
                    if let Some(ty) = &p.ty {
                        format!("{}: {}", p.name.node, format_type(&ty.node))
                    } else {
                        p.name.node.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            let ret_str = ret_ty
                .as_ref()
                .map(|t| format!(" -> {}", format_type(&t.node)))
                .unwrap_or_default();
            format!("fn |{}|{} {{ {} }}", params_str, ret_str, format_expr(&body.node))
        }

        // v0.31: Todo expression
        Expr::Todo { message } => {
            match message {
                Some(msg) => format!("todo \"{}\"", msg),
                None => "todo".to_string(),
            }
        }

        // v0.36: Additional control flow
        Expr::Loop { body } => format!("loop {{ {} }}", format_expr(&body.node)),
        Expr::Break { value } => match value {
            Some(v) => format!("break {}", format_expr(&v.node)),
            None => "break".to_string(),
        },
        Expr::Continue => "continue".to_string(),
        Expr::Return { value } => match value {
            Some(v) => format!("return {}", format_expr(&v.node)),
            None => "return".to_string(),
        },

        // v0.37: Quantifiers
        Expr::Forall { var, ty, body } => {
            format!("forall {}: {}, {}", var.node, format_type(&ty.node), format_expr(&body.node))
        }
        Expr::Exists { var, ty, body } => {
            format!("exists {}: {}, {}", var.node, format_type(&ty.node), format_expr(&body.node))
        }
        // v0.39: Type cast
        Expr::Cast { expr, ty } => {
            format!("{} as {}", format_expr(&expr.node), format_type(&ty.node))
        }
        // v0.51: Index assignment
        Expr::IndexAssign { array, index, value } => {
            format!("{}[{}] = {}", format_expr(&array.node), format_expr(&index.node), format_expr(&value.node))
        }
        // v0.74: RwLock, Barrier, Condvar creation expressions
        Expr::RwLockNew { value } => format!("RwLock::new({})", format_expr(&value.node)),
        Expr::BarrierNew { count } => format!("Barrier::new({})", format_expr(&count.node)),
        Expr::CondvarNew => "Condvar::new()".to_string(),
        // v0.75: Await expression
        Expr::Await { future } => format!("{}.await", format_expr(&future.node)),
        // v0.82: Select expression
        Expr::Select { arms } => {
            let arm_strs: Vec<String> = arms
                .iter()
                .map(|arm| {
                    let binding = arm.binding.as_deref().unwrap_or("_");
                    format!("{} = {} => {{ ... }}", binding, format_expr(&arm.operation.node))
                })
                .collect();
            format!("select {{ {} }}", arm_strs.join(", "))
        }
    }
}

fn format_literal_pattern(lit: &crate::ast::LiteralPattern) -> String {
    use crate::ast::LiteralPattern;
    match lit {
        LiteralPattern::Int(n) => n.to_string(),
        LiteralPattern::Float(f) => f.to_string(),
        LiteralPattern::Bool(b) => b.to_string(),
        LiteralPattern::String(s) => format!("\"{}\"", s),
    }
}

fn format_pattern(pattern: &crate::ast::Pattern) -> String {
    use crate::ast::Pattern;

    match pattern {
        Pattern::Wildcard => "_".to_string(),
        Pattern::Var(name) => name.clone(),
        Pattern::Literal(lit) => format_literal_pattern(lit),
        // v0.41: Nested patterns in enum bindings
        Pattern::EnumVariant { enum_name, variant, bindings } => {
            if bindings.is_empty() {
                format!("{}::{}", enum_name, variant)
            } else {
                let bindings_str: Vec<_> = bindings.iter()
                    .map(|b| format_pattern(&b.node))
                    .collect();
                format!("{}::{}({})", enum_name, variant, bindings_str.join(", "))
            }
        }
        Pattern::Struct { name, fields } => {
            let fields_str: Vec<_> = fields.iter()
                .map(|(n, p)| format!("{}: {}", n.node, format_pattern(&p.node)))
                .collect();
            format!("{} {{ {} }}", name, fields_str.join(", "))
        }
        // v0.39: Range pattern
        Pattern::Range { start, end, inclusive } => {
            let op = if *inclusive { "..=" } else { ".." };
            format!("{}{}{}", format_literal_pattern(start), op, format_literal_pattern(end))
        }
        // v0.40: Or-pattern
        Pattern::Or(alts) => {
            let alts_str: Vec<_> = alts.iter().map(|p| format_pattern(&p.node)).collect();
            alts_str.join(" | ")
        }
        // v0.41: Binding pattern
        Pattern::Binding { name, pattern } => {
            format!("{} @ {}", name, format_pattern(&pattern.node))
        }
        // v0.42: Tuple pattern
        Pattern::Tuple(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            if elems.len() == 1 {
                format!("({},)", elems_str.join(", "))
            } else {
                format!("({})", elems_str.join(", "))
            }
        }
        // v0.44: Array pattern
        Pattern::Array(elems) => {
            let elems_str: Vec<_> = elems.iter().map(|p| format_pattern(&p.node)).collect();
            format!("[{}]", elems_str.join(", "))
        }
        // v0.45: Array rest pattern
        Pattern::ArrayRest { prefix, suffix } => {
            let prefix_str: Vec<_> = prefix.iter().map(|p| format_pattern(&p.node)).collect();
            let suffix_str: Vec<_> = suffix.iter().map(|p| format_pattern(&p.node)).collect();
            match (prefix.is_empty(), suffix.is_empty()) {
                (true, true) => "[..]".to_string(),
                (false, true) => format!("[{}, ..]", prefix_str.join(", ")),
                (true, false) => format!("[.., {}]", suffix_str.join(", ")),
                (false, false) => format!("[{}, .., {}]", prefix_str.join(", "), suffix_str.join(", ")),
            }
        }
    }
}

/// Start the LSP server
pub async fn run_server() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Spanned, Type, BinOp, UnOp, LiteralPattern, Pattern, RangeKind};

    fn sp<T>(node: T) -> Spanned<T> {
        Spanned::new(node, Span::new(0, 0))
    }

    // --- is_ident_char ---

    #[test]
    fn test_is_ident_char_alpha() {
        assert!(Backend::is_ident_char('a'));
        assert!(Backend::is_ident_char('Z'));
    }

    #[test]
    fn test_is_ident_char_digit() {
        assert!(Backend::is_ident_char('0'));
        assert!(Backend::is_ident_char('9'));
    }

    #[test]
    fn test_is_ident_char_underscore() {
        assert!(Backend::is_ident_char('_'));
    }

    #[test]
    fn test_is_ident_char_non_ident() {
        assert!(!Backend::is_ident_char(' '));
        assert!(!Backend::is_ident_char('+'));
        assert!(!Backend::is_ident_char('.'));
        assert!(!Backend::is_ident_char('\n'));
    }

    // --- format_type ---

    #[test]
    fn test_format_type_primitives() {
        assert_eq!(format_type(&Type::I32), "i32");
        assert_eq!(format_type(&Type::I64), "i64");
        assert_eq!(format_type(&Type::U32), "u32");
        assert_eq!(format_type(&Type::U64), "u64");
        assert_eq!(format_type(&Type::F64), "f64");
        assert_eq!(format_type(&Type::Bool), "bool");
        assert_eq!(format_type(&Type::String), "String");
        assert_eq!(format_type(&Type::Char), "char");
        assert_eq!(format_type(&Type::Unit), "()");
        assert_eq!(format_type(&Type::Never), "!");
    }

    #[test]
    fn test_format_type_named() {
        assert_eq!(format_type(&Type::Named("Point".to_string())), "Point");
    }

    #[test]
    fn test_format_type_array() {
        assert_eq!(format_type(&Type::Array(Box::new(Type::I64), 10)), "[i64; 10]");
    }

    #[test]
    fn test_format_type_ref() {
        assert_eq!(format_type(&Type::Ref(Box::new(Type::I64))), "&i64");
        assert_eq!(format_type(&Type::RefMut(Box::new(Type::I64))), "&mut i64");
    }

    #[test]
    fn test_format_type_nullable() {
        assert_eq!(format_type(&Type::Nullable(Box::new(Type::I64))), "i64?");
    }

    #[test]
    fn test_format_type_tuple() {
        assert_eq!(
            format_type(&Type::Tuple(vec![Box::new(Type::I64), Box::new(Type::Bool)])),
            "(i64, bool)"
        );
    }

    #[test]
    fn test_format_type_fn_type() {
        assert_eq!(
            format_type(&Type::Fn {
                params: vec![Box::new(Type::I64), Box::new(Type::I64)],
                ret: Box::new(Type::Bool),
            }),
            "fn(i64, i64) -> bool"
        );
    }

    #[test]
    fn test_format_type_generic() {
        assert_eq!(
            format_type(&Type::Generic {
                name: "Vec".to_string(),
                type_args: vec![Box::new(Type::I64)],
            }),
            "Vec<i64>"
        );
    }

    #[test]
    fn test_format_type_ptr() {
        assert_eq!(format_type(&Type::Ptr(Box::new(Type::I64))), "*i64");
    }

    #[test]
    fn test_format_type_concurrency() {
        assert_eq!(format_type(&Type::Thread(Box::new(Type::I64))), "Thread<i64>");
        assert_eq!(format_type(&Type::Mutex(Box::new(Type::I64))), "Mutex<i64>");
        assert_eq!(format_type(&Type::Arc(Box::new(Type::I64))), "Arc<i64>");
        assert_eq!(format_type(&Type::Atomic(Box::new(Type::I64))), "Atomic<i64>");
        assert_eq!(format_type(&Type::Sender(Box::new(Type::I64))), "Sender<i64>");
        assert_eq!(format_type(&Type::Receiver(Box::new(Type::I64))), "Receiver<i64>");
        assert_eq!(format_type(&Type::Barrier), "Barrier");
        assert_eq!(format_type(&Type::Condvar), "Condvar");
    }

    // --- format_expr ---

    #[test]
    fn test_format_expr_literals() {
        assert_eq!(format_expr(&Expr::IntLit(42)), "42");
        assert_eq!(format_expr(&Expr::FloatLit(1.5)), "1.5");
        assert_eq!(format_expr(&Expr::BoolLit(true)), "true");
        assert_eq!(format_expr(&Expr::StringLit("hello".to_string())), "\"hello\"");
        assert_eq!(format_expr(&Expr::Unit), "()");
        assert_eq!(format_expr(&Expr::Null), "null");
    }

    #[test]
    fn test_format_expr_var() {
        assert_eq!(format_expr(&Expr::Var("x".to_string())), "x");
        assert_eq!(format_expr(&Expr::Ret), "ret");
        assert_eq!(format_expr(&Expr::It), "it");
    }

    #[test]
    fn test_format_expr_binary() {
        let expr = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(sp(Expr::IntLit(1))),
            right: Box::new(sp(Expr::IntLit(2))),
        };
        assert_eq!(format_expr(&expr), "1 + 2");
    }

    #[test]
    fn test_format_expr_unary() {
        let expr = Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(sp(Expr::IntLit(5))),
        };
        assert_eq!(format_expr(&expr), "-5");

        let not_expr = Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(sp(Expr::BoolLit(true))),
        };
        assert_eq!(format_expr(&not_expr), "not true");
    }

    #[test]
    fn test_format_expr_if() {
        let expr = Expr::If {
            cond: Box::new(sp(Expr::BoolLit(true))),
            then_branch: Box::new(sp(Expr::IntLit(1))),
            else_branch: Box::new(sp(Expr::IntLit(0))),
        };
        assert_eq!(format_expr(&expr), "if true then 1 else 0");
    }

    #[test]
    fn test_format_expr_call() {
        let expr = Expr::Call {
            func: "add".to_string(),
            args: vec![sp(Expr::IntLit(1)), sp(Expr::IntLit(2))],
        };
        assert_eq!(format_expr(&expr), "add(1, 2)");
    }

    #[test]
    fn test_format_expr_array_lit() {
        let expr = Expr::ArrayLit(vec![sp(Expr::IntLit(1)), sp(Expr::IntLit(2)), sp(Expr::IntLit(3))]);
        assert_eq!(format_expr(&expr), "[1, 2, 3]");
    }

    #[test]
    fn test_format_expr_index() {
        let expr = Expr::Index {
            expr: Box::new(sp(Expr::Var("arr".to_string()))),
            index: Box::new(sp(Expr::IntLit(0))),
        };
        assert_eq!(format_expr(&expr), "arr[0]");
    }

    #[test]
    fn test_format_expr_block_empty() {
        assert_eq!(format_expr(&Expr::Block(vec![])), "{}");
    }

    #[test]
    fn test_format_expr_range() {
        let expr = Expr::Range {
            start: Box::new(sp(Expr::IntLit(0))),
            end: Box::new(sp(Expr::IntLit(10))),
            kind: RangeKind::Exclusive,
        };
        assert_eq!(format_expr(&expr), "0..<10");
    }

    #[test]
    fn test_format_expr_control_flow() {
        assert_eq!(format_expr(&Expr::Continue), "continue");
        assert_eq!(format_expr(&Expr::Break { value: None }), "break");
        assert_eq!(format_expr(&Expr::Return { value: None }), "return");
        assert_eq!(
            format_expr(&Expr::Return { value: Some(Box::new(sp(Expr::IntLit(42)))) }),
            "return 42"
        );
    }

    #[test]
    fn test_format_expr_todo() {
        assert_eq!(format_expr(&Expr::Todo { message: None }), "todo");
        assert_eq!(format_expr(&Expr::Todo { message: Some("fix".to_string()) }), "todo \"fix\"");
    }

    #[test]
    fn test_format_expr_tuple() {
        let single = Expr::Tuple(vec![sp(Expr::IntLit(1))]);
        assert_eq!(format_expr(&single), "(1,)");
        let pair = Expr::Tuple(vec![sp(Expr::IntLit(1)), sp(Expr::IntLit(2))]);
        assert_eq!(format_expr(&pair), "(1, 2)");
    }

    #[test]
    fn test_format_expr_field_access() {
        let expr = Expr::FieldAccess {
            expr: Box::new(sp(Expr::Var("p".to_string()))),
            field: sp("x".to_string()),
        };
        assert_eq!(format_expr(&expr), "p.x");
    }

    #[test]
    fn test_format_expr_enum_variant() {
        let simple = Expr::EnumVariant {
            enum_name: "Option".to_string(),
            variant: "None".to_string(),
            args: vec![],
        };
        assert_eq!(format_expr(&simple), "Option::None");
        let with_args = Expr::EnumVariant {
            enum_name: "Option".to_string(),
            variant: "Some".to_string(),
            args: vec![sp(Expr::IntLit(42))],
        };
        assert_eq!(format_expr(&with_args), "Option::Some(42)");
    }

    // --- format_pattern ---

    #[test]
    fn test_format_pattern_wildcard() {
        assert_eq!(format_pattern(&Pattern::Wildcard), "_");
    }

    #[test]
    fn test_format_pattern_var() {
        assert_eq!(format_pattern(&Pattern::Var("x".to_string())), "x");
    }

    #[test]
    fn test_format_pattern_literal() {
        assert_eq!(
            format_pattern(&Pattern::Literal(LiteralPattern::Int(42))),
            "42"
        );
    }

    #[test]
    fn test_format_pattern_enum_variant() {
        let simple = Pattern::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "Red".to_string(),
            bindings: vec![],
        };
        assert_eq!(format_pattern(&simple), "Color::Red");

        let with_bindings = Pattern::EnumVariant {
            enum_name: "Option".to_string(),
            variant: "Some".to_string(),
            bindings: vec![sp(Pattern::Var("x".to_string()))],
        };
        assert_eq!(format_pattern(&with_bindings), "Option::Some(x)");
    }

    #[test]
    fn test_format_pattern_tuple() {
        let pair = Pattern::Tuple(vec![
            sp(Pattern::Var("a".to_string())),
            sp(Pattern::Var("b".to_string())),
        ]);
        assert_eq!(format_pattern(&pair), "(a, b)");
    }

    #[test]
    fn test_format_pattern_or() {
        let or_pat = Pattern::Or(vec![
            sp(Pattern::Literal(LiteralPattern::Int(1))),
            sp(Pattern::Literal(LiteralPattern::Int(2))),
        ]);
        assert_eq!(format_pattern(&or_pat), "1 | 2");
    }

    #[test]
    fn test_format_pattern_range() {
        let range = Pattern::Range {
            start: LiteralPattern::Int(0),
            end: LiteralPattern::Int(9),
            inclusive: true,
        };
        assert_eq!(format_pattern(&range), "0..=9");
    }

    #[test]
    fn test_format_pattern_array() {
        let arr = Pattern::Array(vec![
            sp(Pattern::Var("a".to_string())),
            sp(Pattern::Wildcard),
        ]);
        assert_eq!(format_pattern(&arr), "[a, _]");
    }

    #[test]
    fn test_format_pattern_array_rest() {
        let rest = Pattern::ArrayRest {
            prefix: vec![sp(Pattern::Var("first".to_string()))],
            suffix: vec![sp(Pattern::Var("last".to_string()))],
        };
        assert_eq!(format_pattern(&rest), "[first, .., last]");
    }

    // --- format_literal_pattern ---

    #[test]
    fn test_format_literal_pattern_all() {
        assert_eq!(format_literal_pattern(&LiteralPattern::Int(42)), "42");
        assert_eq!(format_literal_pattern(&LiteralPattern::Float(1.5)), "1.5");
        assert_eq!(format_literal_pattern(&LiteralPattern::Bool(true)), "true");
        assert_eq!(format_literal_pattern(&LiteralPattern::String("hi".to_string())), "\"hi\"");
    }

    // --- try_parse ---

    #[test]
    fn test_try_parse_valid() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let result = backend.try_parse("fn main() -> i64 = 42;");
        assert!(result.is_some());
    }

    #[test]
    fn test_try_parse_invalid() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let result = backend.try_parse("fn {{{ invalid");
        assert!(result.is_none());
    }

    // --- coordinate conversion ---

    #[test]
    fn test_offset_to_position_start() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let pos = backend.offset_to_position(0, "hello");
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_offset_to_position_same_line() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let pos = backend.offset_to_position(5, "hello world");
        assert_eq!(pos.line, 0);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_offset_to_position_multiline() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let content = "line1\nline2\nline3";
        let pos = backend.offset_to_position(6, content); // start of "line2"
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 0);
    }

    #[test]
    fn test_position_to_offset_start() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let offset = backend.position_to_offset(Position::new(0, 0), "hello");
        assert_eq!(offset, 0);
    }

    #[test]
    fn test_position_to_offset_multiline() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let content = "line1\nline2\nline3";
        let offset = backend.position_to_offset(Position::new(1, 0), content);
        assert_eq!(offset, 6); // "line1\n" = 6 bytes
    }

    #[test]
    fn test_offset_position_roundtrip() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let content = "fn main() -> i64\n= 42;";
        for i in 0..content.len() {
            let pos = backend.offset_to_position(i, content);
            let back = backend.position_to_offset(pos, content);
            assert_eq!(back, i, "roundtrip failed for offset {}", i);
        }
    }

    // --- get_word_at_position ---

    #[test]
    fn test_get_word_at_position_middle() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let word = backend.get_word_at_position("fn main() -> i64", Position::new(0, 4));
        assert_eq!(word, Some("main".to_string()));
    }

    #[test]
    fn test_get_word_at_position_no_word() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let word = backend.get_word_at_position("fn ()", Position::new(0, 3));
        assert_eq!(word, None);
    }

    // --- get_diagnostics ---

    #[test]
    fn test_get_diagnostics_valid_code() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let url = Url::parse("file:///test.bmb").unwrap();
        let diags = backend.get_diagnostics(&url, "fn main() -> i64 = 42;");
        assert!(diags.is_empty());
    }

    #[test]
    fn test_get_diagnostics_syntax_error() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let url = Url::parse("file:///test.bmb").unwrap();
        let diags = backend.get_diagnostics(&url, "fn {{{ invalid");
        assert!(!diags.is_empty());
    }

    // --- collect_symbols ---

    #[test]
    fn test_collect_symbols_empty_program() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = Program { header: None, items: vec![] };
        let (defs, refs, locals) = backend.collect_symbols(&ast);
        assert!(defs.is_empty());
        assert!(refs.is_empty());
        assert!(locals.is_empty());
    }

    #[test]
    fn test_collect_symbols_fn_def() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("fn main() -> i64 = 42;").unwrap();
        let (defs, _refs, _locals) = backend.collect_symbols(&ast);
        assert!(defs.iter().any(|d| d.name == "main" && d.kind == SymbolKind::Function));
    }

    #[test]
    fn test_collect_symbols_fn_params() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("fn add(a: i64, b: i64) -> i64 = a + b;").unwrap();
        let (defs, _refs, locals) = backend.collect_symbols(&ast);
        // Function itself
        assert!(defs.iter().any(|d| d.name == "add" && d.kind == SymbolKind::Function));
        // Parameters as definitions
        assert!(defs.iter().any(|d| d.name == "a" && d.kind == SymbolKind::Parameter));
        assert!(defs.iter().any(|d| d.name == "b" && d.kind == SymbolKind::Parameter));
        // Parameters also as locals for scope-based completion
        assert!(locals.iter().any(|l| l.name == "a" && l.type_str == "i64"));
        assert!(locals.iter().any(|l| l.name == "b" && l.type_str == "i64"));
    }

    #[test]
    fn test_collect_symbols_fn_type_signature() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("fn add(a: i64, b: i64) -> i64 = a + b;").unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let fn_def = defs.iter().find(|d| d.name == "add").unwrap();
        let sig = fn_def.type_str.as_ref().unwrap();
        assert!(sig.contains("fn("));
        assert!(sig.contains("i64"));
    }

    #[test]
    fn test_collect_symbols_struct_def() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("struct Point { x: i64, y: i64 }").unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let struct_def = defs.iter().find(|d| d.name == "Point").unwrap();
        assert_eq!(struct_def.kind, SymbolKind::Struct);
        let type_str = struct_def.type_str.as_ref().unwrap();
        assert!(type_str.contains("x: i64"));
        assert!(type_str.contains("y: i64"));
    }

    #[test]
    fn test_collect_symbols_enum_def() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("enum Color { Red, Green, Blue }").unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let enum_def = defs.iter().find(|d| d.name == "Color").unwrap();
        assert_eq!(enum_def.kind, SymbolKind::Enum);
        let type_str = enum_def.type_str.as_ref().unwrap();
        assert!(type_str.contains("Red"));
        assert!(type_str.contains("Green"));
        assert!(type_str.contains("Blue"));
    }

    #[test]
    fn test_collect_symbols_extern_fn() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("extern fn putchar(c: i64) -> i64;").unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let ext_def = defs.iter().find(|d| d.name == "putchar").unwrap();
        assert_eq!(ext_def.kind, SymbolKind::Function);
        let sig = ext_def.type_str.as_ref().unwrap();
        assert!(sig.starts_with("extern fn("));
    }

    #[test]
    fn test_collect_symbols_trait_def() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("trait Printable { fn display(self: &Self) -> i64; }").unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let trait_def = defs.iter().find(|d| d.name == "Printable").unwrap();
        assert_eq!(trait_def.kind, SymbolKind::Trait);
    }

    #[test]
    fn test_collect_symbols_impl_block() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let source = r#"
            struct Point { x: i64, y: i64 }
            trait HasVal { fn get(self: Self) -> i64; }
            impl HasVal for Point {
                fn get(self: Self) -> i64 = self.x;
            }
        "#;
        let ast = backend.try_parse(source).unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        let method_def = defs.iter().find(|d| d.name == "get" && d.kind == SymbolKind::Method);
        assert!(method_def.is_some());
    }

    #[test]
    fn test_collect_symbols_multiple_items() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let source = r#"
            fn foo() -> i64 = 1;
            fn bar() -> i64 = 2;
            struct S { x: i64 }
        "#;
        let ast = backend.try_parse(source).unwrap();
        let (defs, _, _) = backend.collect_symbols(&ast);
        assert!(defs.iter().any(|d| d.name == "foo" && d.kind == SymbolKind::Function));
        assert!(defs.iter().any(|d| d.name == "bar" && d.kind == SymbolKind::Function));
        assert!(defs.iter().any(|d| d.name == "S" && d.kind == SymbolKind::Struct));
    }

    // --- collect_locals ---

    #[test]
    fn test_collect_locals_let_binding() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let ast = backend.try_parse("fn main() -> i64 = let x: i64 = 10; x;").unwrap();
        let (_, _, locals) = backend.collect_symbols(&ast);
        // Should have 'x' as a local variable
        assert!(locals.iter().any(|l| l.name == "x" && l.type_str == "i64"));
    }

    #[test]
    fn test_collect_locals_for_loop_var() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let source = "fn main() -> i64 = { let mut sum: i64 = 0; for i in 0..5 { sum = sum + i; 0 }; sum };";
        let ast = backend.try_parse(source).unwrap();
        let (_, _, locals) = backend.collect_symbols(&ast);
        assert!(locals.iter().any(|l| l.name == "i" && l.type_str == "inferred"));
        assert!(locals.iter().any(|l| l.name == "sum" && l.type_str == "i64"));
    }

    #[test]
    fn test_collect_locals_nested_let() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let source = "fn main() -> i64 = let a: i64 = 1; let b: i64 = 2; a + b;";
        let ast = backend.try_parse(source).unwrap();
        let (_, _, locals) = backend.collect_symbols(&ast);
        assert!(locals.iter().any(|l| l.name == "a"));
        assert!(locals.iter().any(|l| l.name == "b"));
    }

    // --- get_locals_at_offset ---

    #[test]
    fn test_get_locals_at_offset_in_scope() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let locals = vec![
            LocalVar {
                name: "x".to_string(),
                type_str: "i64".to_string(),
                def_span: Span::new(10, 15),
                scope_span: Span::new(15, 50),
            },
        ];
        // Offset 20 is within def_span.start..scope_span.end (10..50)
        let visible = backend.get_locals_at_offset(&locals, 20);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "x");
    }

    #[test]
    fn test_get_locals_at_offset_before_def() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let locals = vec![
            LocalVar {
                name: "x".to_string(),
                type_str: "i64".to_string(),
                def_span: Span::new(10, 15),
                scope_span: Span::new(15, 50),
            },
        ];
        // Offset 5 is before def_span.start (10)
        let visible = backend.get_locals_at_offset(&locals, 5);
        assert!(visible.is_empty());
    }

    #[test]
    fn test_get_locals_at_offset_after_scope() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let locals = vec![
            LocalVar {
                name: "x".to_string(),
                type_str: "i64".to_string(),
                def_span: Span::new(10, 15),
                scope_span: Span::new(15, 50),
            },
        ];
        // Offset 60 is after scope_span.end (50)
        let visible = backend.get_locals_at_offset(&locals, 60);
        assert!(visible.is_empty());
    }

    #[test]
    fn test_get_locals_at_offset_multiple_vars() {
        let (service, _) = LspService::new(Backend::new);
        let backend = service.inner();
        let locals = vec![
            LocalVar {
                name: "a".to_string(),
                type_str: "i64".to_string(),
                def_span: Span::new(10, 15),
                scope_span: Span::new(15, 100),
            },
            LocalVar {
                name: "b".to_string(),
                type_str: "bool".to_string(),
                def_span: Span::new(30, 35),
                scope_span: Span::new(35, 100),
            },
            LocalVar {
                name: "c".to_string(),
                type_str: "String".to_string(),
                def_span: Span::new(60, 65),
                scope_span: Span::new(65, 80),
            },
        ];
        // At offset 40: a and b visible, c not yet defined
        let visible = backend.get_locals_at_offset(&locals, 40);
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().any(|l| l.name == "a"));
        assert!(visible.iter().any(|l| l.name == "b"));

        // At offset 70: all three visible
        let visible = backend.get_locals_at_offset(&locals, 70);
        assert_eq!(visible.len(), 3);

        // At offset 90: a and b visible, c out of scope
        let visible = backend.get_locals_at_offset(&locals, 90);
        assert_eq!(visible.len(), 2);
        assert!(visible.iter().any(|l| l.name == "a"));
        assert!(visible.iter().any(|l| l.name == "b"));
    }
}
