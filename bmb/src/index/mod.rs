//! BMB AI Query Index - v0.25.0
//!
//! Generates index files for AI tools to query BMB projects.
//! RFC-0001: AI-Native Code Query System

use crate::ast::{self, Expr, FnDef, Item, Program, StateKind, Type, Visibility};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Index manifest containing metadata about the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub bmb_version: String,
    pub project: String,
    pub indexed_at: String,
    pub files: usize,
    pub functions: usize,
    pub types: usize,
    pub structs: usize,
    pub enums: usize,
    pub contracts: usize,
}

/// Symbol kind in the index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Type,
    Trait,
    Const,
}

/// A symbol entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub kind: SymbolKind,
    pub name: String,
    pub file: String,
    pub line: usize,
    #[serde(rename = "pub")]
    pub is_pub: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
}

/// Function details for the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub name: String,
    pub file: String,
    pub line: usize,
    #[serde(rename = "pub")]
    pub is_pub: bool,
    pub signature: FunctionSignature,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts: Option<ContractInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_info: Option<BodyInfo>,
}

/// Function signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub params: Vec<ParamInfo>,
    #[serde(rename = "return")]
    pub return_type: String,
}

/// Parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

/// Contract information (pre/post conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<ContractExpr>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Vec<ContractExpr>>,
}

/// A contract expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExpr {
    pub expr: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quantifiers: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub uses_old: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub uses_ret: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Body analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyInfo {
    pub calls: Vec<String>,
    pub recursive: bool,
    pub has_loop: bool,
}

/// Type entry for the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeEntry {
    pub name: String,
    pub file: String,
    pub line: usize,
    #[serde(rename = "pub")]
    pub is_pub: bool,
    pub kind: String,  // "struct", "enum", "type", "trait"
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldInfo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refinement: Option<RefinementInfo>,
}

/// Field information for structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
}

/// Refinement type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementInfo {
    pub base: String,
    pub constraint: String,
}

/// The complete index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIndex {
    pub manifest: Manifest,
    pub symbols: Vec<SymbolEntry>,
    pub functions: Vec<FunctionEntry>,
    pub types: Vec<TypeEntry>,
}

// =============================================================================
// v0.50.24 - Proof Verification Index (Task 47.7-47.8)
// =============================================================================

/// Verification status for a function's contracts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProofStatus {
    /// Contract verified (Z3 returned unsat)
    Verified,
    /// Contract failed with counterexample
    Failed,
    /// Verification timed out
    Timeout,
    /// Z3 returned unknown
    Unknown,
    /// Not yet verified
    Pending,
    /// Z3 not available on this system
    Unavailable,
}

/// A proof entry for a function with contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofEntry {
    /// Function name
    pub name: String,
    /// Source file
    pub file: String,
    /// Line number
    pub line: usize,
    /// Precondition verification status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_status: Option<ProofStatus>,
    /// Postcondition verification status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_status: Option<ProofStatus>,
    /// Counterexample if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterexample: Option<Vec<(String, String)>>,
    /// Verification time in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_time_ms: Option<u64>,
    /// Last verified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

/// Proof index containing verification results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofIndex {
    pub version: String,
    pub z3_available: bool,
    pub z3_version: Option<String>,
    pub verified_at: String,
    pub proofs: Vec<ProofEntry>,
}

impl ProofIndex {
    pub fn new(z3_available: bool, z3_version: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            version: "1".to_string(),
            z3_available,
            z3_version,
            verified_at: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            proofs: Vec::new(),
        }
    }

    pub fn add_proof(&mut self, entry: ProofEntry) {
        // Update or add proof entry
        if let Some(existing) = self.proofs.iter_mut().find(|p| p.name == entry.name && p.file == entry.file) {
            *existing = entry;
        } else {
            self.proofs.push(entry);
        }
    }
}

/// Write proof index to .bmb/index/proofs.json
pub fn write_proof_index(index: &ProofIndex, project_root: &Path) -> std::io::Result<()> {
    let index_dir = project_root.join(".bmb").join("index");
    std::fs::create_dir_all(&index_dir)?;

    let proofs_path = index_dir.join("proofs.json");
    let proofs_json = serde_json::to_string_pretty(index)?;
    std::fs::write(&proofs_path, proofs_json)?;

    Ok(())
}

/// Read proof index from .bmb/index/proofs.json
pub fn read_proof_index(project_root: &Path) -> std::io::Result<ProofIndex> {
    let proofs_path = project_root.join(".bmb").join("index").join("proofs.json");
    let proofs_json = std::fs::read_to_string(&proofs_path)?;
    let index: ProofIndex = serde_json::from_str(&proofs_json)?;
    Ok(index)
}

/// Index generator
pub struct IndexGenerator {
    project_name: String,
    files_indexed: usize,
    symbols: Vec<SymbolEntry>,
    functions: Vec<FunctionEntry>,
    types: Vec<TypeEntry>,
}

impl IndexGenerator {
    pub fn new(project_name: &str) -> Self {
        Self {
            project_name: project_name.to_string(),
            files_indexed: 0,
            symbols: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
        }
    }

    /// Index a single file
    pub fn index_file(&mut self, filename: &str, program: &Program) {
        self.files_indexed += 1;

        for item in &program.items {
            match item {
                Item::FnDef(fn_def) => {
                    self.index_function(filename, fn_def);
                }
                Item::StructDef(s) => {
                    self.index_struct(filename, s);
                }
                Item::EnumDef(e) => {
                    self.index_enum(filename, e);
                }
                Item::TraitDef(t) => {
                    self.index_trait(filename, t);
                }
                Item::ExternFn(e) => {
                    self.index_extern_fn(filename, e);
                }
                _ => {}
            }
        }
    }

    fn index_function(&mut self, filename: &str, fn_def: &FnDef) {
        let is_pub = fn_def.visibility == Visibility::Public;
        let line = 1; // Would need span info for accurate line numbers

        // Create symbol entry
        let signature = self.format_fn_signature(fn_def);
        self.symbols.push(SymbolEntry {
            kind: SymbolKind::Function,
            name: fn_def.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: Some(signature.clone()),
            doc: None,
        });

        // Create detailed function entry
        let params: Vec<ParamInfo> = fn_def
            .params
            .iter()
            .map(|p| ParamInfo {
                name: p.name.node.clone(),
                ty: self.format_type(&p.ty.node),
            })
            .collect();

        let contracts = self.extract_contracts(fn_def);
        let body_info = self.analyze_body(&fn_def.body.node, &fn_def.name.node);

        self.functions.push(FunctionEntry {
            name: fn_def.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: FunctionSignature {
                params,
                return_type: self.format_type(&fn_def.ret_ty.node),
            },
            contracts,
            body_info,
        });
    }

    fn index_struct(&mut self, filename: &str, s: &ast::StructDef) {
        let is_pub = s.visibility == Visibility::Public;
        let line = 1;

        self.symbols.push(SymbolEntry {
            kind: SymbolKind::Struct,
            name: s.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: None,
            doc: None,
        });

        let fields: Vec<FieldInfo> = s
            .fields
            .iter()
            .map(|f| FieldInfo {
                name: f.name.node.clone(),
                ty: self.format_type(&f.ty.node),
            })
            .collect();

        self.types.push(TypeEntry {
            name: s.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            kind: "struct".to_string(),
            fields,
            variants: Vec::new(),
            refinement: None,
        });
    }

    fn index_enum(&mut self, filename: &str, e: &ast::EnumDef) {
        let is_pub = e.visibility == Visibility::Public;
        let line = 1;

        self.symbols.push(SymbolEntry {
            kind: SymbolKind::Enum,
            name: e.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: None,
            doc: None,
        });

        let variants: Vec<String> = e.variants.iter().map(|v| v.name.node.clone()).collect();

        self.types.push(TypeEntry {
            name: e.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            kind: "enum".to_string(),
            fields: Vec::new(),
            variants,
            refinement: None,
        });
    }

    fn index_trait(&mut self, filename: &str, t: &ast::TraitDef) {
        let is_pub = t.visibility == Visibility::Public;
        let line = 1;

        self.symbols.push(SymbolEntry {
            kind: SymbolKind::Trait,
            name: t.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: None,
            doc: None,
        });

        self.types.push(TypeEntry {
            name: t.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            kind: "trait".to_string(),
            fields: Vec::new(),
            variants: Vec::new(),
            refinement: None,
        });
    }

    fn index_extern_fn(&mut self, filename: &str, e: &ast::ExternFn) {
        let is_pub = e.visibility == Visibility::Public;
        let line = 1;

        let params: Vec<String> = e
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name.node, self.format_type(&p.ty.node)))
            .collect();
        let signature = format!(
            "extern fn({}) -> {}",
            params.join(", "),
            self.format_type(&e.ret_ty.node)
        );

        self.symbols.push(SymbolEntry {
            kind: SymbolKind::Function,
            name: e.name.node.clone(),
            file: filename.to_string(),
            line,
            is_pub,
            signature: Some(signature),
            doc: None,
        });
    }

    fn format_fn_signature(&self, fn_def: &FnDef) -> String {
        let params: Vec<String> = fn_def
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name.node, self.format_type(&p.ty.node)))
            .collect();
        format!(
            "fn({}) -> {}",
            params.join(", "),
            self.format_type(&fn_def.ret_ty.node)
        )
    }

    fn format_type(&self, ty: &Type) -> String {
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
            Type::Named(name) => name.clone(),
            Type::TypeVar(name) => name.clone(),
            Type::Generic { name, type_args } => {
                let args: Vec<String> = type_args.iter().map(|t| self.format_type(t)).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            Type::Struct { name, .. } => name.clone(),
            Type::Enum { name, .. } => name.clone(),
            Type::Array(elem, size) => format!("[{}; {}]", self.format_type(elem), size),
            Type::Ref(inner) => format!("&{}", self.format_type(inner)),
            Type::RefMut(inner) => format!("&mut {}", self.format_type(inner)),
            Type::Range(elem) => format!("Range<{}>", self.format_type(elem)),
            Type::Refined { base, constraints } => {
                let constraint_strs: Vec<String> =
                    constraints.iter().map(|c| self.format_expr(&c.node)).collect();
                format!("{}{{{}}}", self.format_type(base), constraint_strs.join(", "))
            }
            Type::Fn { params, ret } => {
                let param_strs: Vec<String> = params.iter().map(|p| self.format_type(p)).collect();
                format!("fn({}) -> {}", param_strs.join(", "), self.format_type(ret))
            }
            // v0.31: Never type
            Type::Never => "!".to_string(),
            // v0.37: Nullable type
            Type::Nullable(inner) => format!("{}?", self.format_type(inner)),
            // v0.42: Tuple type
            Type::Tuple(elems) => {
                let elems_str: Vec<_> = elems.iter().map(|t| self.format_type(t)).collect();
                format!("({})", elems_str.join(", "))
            }
        }
    }

    fn format_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(n) => n.to_string(),
            Expr::FloatLit(f) => f.to_string(),
            Expr::BoolLit(b) => b.to_string(),
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::Unit => "()".to_string(),
            Expr::Var(name) => name.clone(),
            Expr::Ret => "ret".to_string(),
            Expr::It => "it".to_string(),
            Expr::Binary { left, op, right } => {
                let op_str = match op {
                    ast::BinOp::Add => "+",
                    ast::BinOp::Sub => "-",
                    ast::BinOp::Mul => "*",
                    ast::BinOp::Div => "/",
                    ast::BinOp::Mod => "%",
                    // v0.37: Wrapping arithmetic
                    ast::BinOp::AddWrap => "+%",
                    ast::BinOp::SubWrap => "-%",
                    ast::BinOp::MulWrap => "*%",
                    // v0.38: Checked arithmetic
                    ast::BinOp::AddChecked => "+?",
                    ast::BinOp::SubChecked => "-?",
                    ast::BinOp::MulChecked => "*?",
                    // v0.38: Saturating arithmetic
                    ast::BinOp::AddSat => "+|",
                    ast::BinOp::SubSat => "-|",
                    ast::BinOp::MulSat => "*|",
                    ast::BinOp::Eq => "==",
                    ast::BinOp::Ne => "!=",
                    ast::BinOp::Lt => "<",
                    ast::BinOp::Le => "<=",
                    ast::BinOp::Gt => ">",
                    ast::BinOp::Ge => ">=",
                    ast::BinOp::And => "and",
                    ast::BinOp::Or => "or",
                    // v0.32: Shift operators
                    ast::BinOp::Shl => "<<",
                    ast::BinOp::Shr => ">>",
                    // v0.36: Bitwise operators
                    ast::BinOp::Band => "band",
                    ast::BinOp::Bor => "bor",
                    ast::BinOp::Bxor => "bxor",
                    // v0.36: Logical implication
                    ast::BinOp::Implies => "implies",
                };
                format!(
                    "{} {} {}",
                    self.format_expr(&left.node),
                    op_str,
                    self.format_expr(&right.node)
                )
            }
            Expr::Unary { op, expr } => {
                let op_str = match op {
                    ast::UnOp::Neg => "-",
                    ast::UnOp::Not => "not ",
                    // v0.36: Bitwise not
                    ast::UnOp::Bnot => "bnot ",
                };
                format!("{}{}", op_str, self.format_expr(&expr.node))
            }
            Expr::Call { func, args } => {
                let args_str: Vec<String> = args.iter().map(|a| self.format_expr(&a.node)).collect();
                format!("{}({})", func, args_str.join(", "))
            }
            Expr::If { cond, then_branch, else_branch } => {
                format!(
                    "if {} then {} else {}",
                    self.format_expr(&cond.node),
                    self.format_expr(&then_branch.node),
                    self.format_expr(&else_branch.node)
                )
            }
            _ => "...".to_string(),
        }
    }

    fn extract_contracts(&self, fn_def: &FnDef) -> Option<ContractInfo> {
        let has_pre = fn_def.pre.is_some();
        let has_post = fn_def.post.is_some();

        if !has_pre && !has_post {
            return None;
        }

        let pre = fn_def.pre.as_ref().map(|p| {
            vec![self.analyze_contract_expr(&p.node)]
        });

        let post = fn_def.post.as_ref().map(|p| {
            vec![self.analyze_contract_expr(&p.node)]
        });

        Some(ContractInfo { pre, post })
    }

    fn analyze_contract_expr(&self, expr: &Expr) -> ContractExpr {
        let quantifiers = Vec::new();
        let mut calls = Vec::new();
        let uses_old = self.contains_old(expr);
        let uses_ret = self.contains_ret(expr);

        self.collect_calls(expr, &mut calls);

        ContractExpr {
            expr: self.format_expr(expr),
            quantifiers,
            calls,
            uses_old,
            uses_ret,
        }
    }

    fn contains_old(&self, expr: &Expr) -> bool {
        match expr {
            Expr::StateRef { state, .. } => matches!(state, StateKind::Pre),
            Expr::Binary { left, right, .. } => {
                self.contains_old(&left.node) || self.contains_old(&right.node)
            }
            Expr::Unary { expr, .. } => self.contains_old(&expr.node),
            Expr::Call { args, .. } => args.iter().any(|a| self.contains_old(&a.node)),
            Expr::If { cond, then_branch, else_branch } => {
                self.contains_old(&cond.node)
                    || self.contains_old(&then_branch.node)
                    || self.contains_old(&else_branch.node)
            }
            _ => false,
        }
    }

    fn contains_ret(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ret => true,
            Expr::Binary { left, right, .. } => {
                self.contains_ret(&left.node) || self.contains_ret(&right.node)
            }
            Expr::Unary { expr, .. } => self.contains_ret(&expr.node),
            Expr::Call { args, .. } => args.iter().any(|a| self.contains_ret(&a.node)),
            Expr::If { cond, then_branch, else_branch } => {
                self.contains_ret(&cond.node)
                    || self.contains_ret(&then_branch.node)
                    || self.contains_ret(&else_branch.node)
            }
            _ => false,
        }
    }

    fn collect_calls(&self, expr: &Expr, calls: &mut Vec<String>) {
        match expr {
            Expr::Call { func, args } => {
                if !calls.contains(func) {
                    calls.push(func.clone());
                }
                for arg in args {
                    self.collect_calls(&arg.node, calls);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_calls(&left.node, calls);
                self.collect_calls(&right.node, calls);
            }
            Expr::Unary { expr, .. } => {
                self.collect_calls(&expr.node, calls);
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.collect_calls(&cond.node, calls);
                self.collect_calls(&then_branch.node, calls);
                self.collect_calls(&else_branch.node, calls);
            }
            Expr::Let { value, body, .. } => {
                self.collect_calls(&value.node, calls);
                self.collect_calls(&body.node, calls);
            }
            _ => {}
        }
    }

    fn analyze_body(&self, expr: &Expr, fn_name: &str) -> Option<BodyInfo> {
        let mut calls = Vec::new();
        self.collect_calls(expr, &mut calls);

        let recursive = calls.contains(&fn_name.to_string());
        let has_loop = self.contains_loop(expr);

        Some(BodyInfo {
            calls,
            recursive,
            has_loop,
        })
    }

    fn contains_loop(&self, expr: &Expr) -> bool {
        match expr {
            Expr::While { .. } | Expr::For { .. } => true,
            Expr::Let { value, body, .. } => {
                self.contains_loop(&value.node) || self.contains_loop(&body.node)
            }
            Expr::If { cond, then_branch, else_branch } => {
                self.contains_loop(&cond.node)
                    || self.contains_loop(&then_branch.node)
                    || self.contains_loop(&else_branch.node)
            }
            Expr::Block(stmts) => stmts.iter().any(|s| self.contains_loop(&s.node)),
            _ => false,
        }
    }

    /// Generate the final index
    pub fn generate(self) -> ProjectIndex {
        let now = chrono::Utc::now();
        let indexed_at = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let manifest = Manifest {
            version: "1".to_string(),
            bmb_version: env!("CARGO_PKG_VERSION").to_string(),
            project: self.project_name,
            indexed_at,
            files: self.files_indexed,
            functions: self.functions.len(),
            types: self.types.len(),
            structs: self.types.iter().filter(|t| t.kind == "struct").count(),
            enums: self.types.iter().filter(|t| t.kind == "enum").count(),
            contracts: self
                .functions
                .iter()
                .filter(|f| f.contracts.is_some())
                .count(),
        };

        ProjectIndex {
            manifest,
            symbols: self.symbols,
            functions: self.functions,
            types: self.types,
        }
    }
}

/// Write index to the .bmb/index directory
pub fn write_index(index: &ProjectIndex, project_root: &Path) -> std::io::Result<()> {
    let index_dir = project_root.join(".bmb").join("index");
    std::fs::create_dir_all(&index_dir)?;

    // Write manifest
    let manifest_path = index_dir.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&index.manifest)?;
    std::fs::write(&manifest_path, manifest_json)?;

    // Write symbols index
    let symbols_path = index_dir.join("symbols.json");
    let symbols_json = serde_json::to_string_pretty(&index.symbols)?;
    std::fs::write(&symbols_path, symbols_json)?;

    // Write functions index
    let functions_path = index_dir.join("functions.json");
    let functions_json = serde_json::to_string_pretty(&index.functions)?;
    std::fs::write(&functions_path, functions_json)?;

    // Write types index
    let types_path = index_dir.join("types.json");
    let types_json = serde_json::to_string_pretty(&index.types)?;
    std::fs::write(&types_path, types_json)?;

    Ok(())
}

/// Read index from the .bmb/index directory
pub fn read_index(project_root: &Path) -> std::io::Result<ProjectIndex> {
    let index_dir = project_root.join(".bmb").join("index");

    let manifest_path = index_dir.join("manifest.json");
    let manifest_json = std::fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_json)?;

    let symbols_path = index_dir.join("symbols.json");
    let symbols_json = std::fs::read_to_string(&symbols_path)?;
    let symbols: Vec<SymbolEntry> = serde_json::from_str(&symbols_json)?;

    let functions_path = index_dir.join("functions.json");
    let functions_json = std::fs::read_to_string(&functions_path)?;
    let functions: Vec<FunctionEntry> = serde_json::from_str(&functions_json)?;

    let types_path = index_dir.join("types.json");
    let types_json = std::fs::read_to_string(&types_path)?;
    let types: Vec<TypeEntry> = serde_json::from_str(&types_json)?;

    Ok(ProjectIndex {
        manifest,
        symbols,
        functions,
        types,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_generator() {
        let generator = IndexGenerator::new("test-project");
        assert_eq!(generator.files_indexed, 0);
    }
}
