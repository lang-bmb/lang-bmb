//! Module Resolver for BMB
//!
//! Handles multi-file compilation by resolving `use` statements and
//! loading/parsing modules from the file system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::{Item, Program, Span, UseStmt, Visibility};
use crate::error::{CompileError, Result};

// ============================================================================
// v0.68: Levenshtein Distance for Module/Item Suggestions
// ============================================================================

/// Calculate Levenshtein edit distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1)
                .min(curr[j - 1] + 1)
                .min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Find the most similar name from a list of candidates (threshold = 2)
fn find_similar_name<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let threshold = 2;
    let mut best_match: Option<&str> = None;
    let mut best_distance = usize::MAX;

    for &candidate in candidates {
        let distance = levenshtein_distance(name, candidate);
        if distance < best_distance && distance <= threshold {
            best_distance = distance;
            best_match = Some(candidate);
        }
    }

    best_match
}

/// Format a suggestion hint
fn format_suggestion_hint(suggestion: Option<&str>) -> String {
    match suggestion {
        Some(name) => format!("\n  hint: did you mean `{}`?", name),
        None => String::new(),
    }
}

/// A resolved module containing its path and parsed content
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name (e.g., "lexer" for lexer.bmb)
    pub name: String,
    /// Canonical file path
    pub path: PathBuf,
    /// Parsed program content
    pub program: Program,
    /// Exported items (pub items)
    pub exports: HashMap<String, ExportedItem>,
}

/// An exported item from a module
#[derive(Debug, Clone)]
pub enum ExportedItem {
    Function(String),
    Struct(String),
    Enum(String),
}

/// Module resolver for multi-file compilation
#[derive(Debug)]
pub struct Resolver {
    /// Base directory for module resolution
    base_dir: PathBuf,
    /// Loaded modules by name
    modules: HashMap<String, Module>,
    /// Module load order (for dependency tracking)
    load_order: Vec<String>,
}

impl Resolver {
    /// Create a new resolver with the given base directory
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            modules: HashMap::new(),
            load_order: Vec::new(),
        }
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Load a module by name, parsing the corresponding .bmb file
    pub fn load_module(&mut self, module_name: &str) -> Result<&Module> {
        // Check if already loaded
        if self.modules.contains_key(module_name) {
            return Ok(self.modules.get(module_name).unwrap());
        }

        // Resolve file path
        let file_path = self.resolve_module_path(module_name)?;

        // Read the file
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            CompileError::io_error(format!(
                "Failed to read module '{}' at {:?}: {}",
                module_name, file_path, e
            ))
        })?;

        // Tokenize
        let tokens = crate::lexer::tokenize(&source)?;

        // Parse
        let program = crate::parser::parse(module_name, &source, tokens)?;

        // Extract exports (pub items)
        let exports = Self::extract_exports(&program);

        // Create and store the module
        let module = Module {
            name: module_name.to_string(),
            path: file_path,
            program,
            exports,
        };

        self.modules.insert(module_name.to_string(), module);
        self.load_order.push(module_name.to_string());

        Ok(self.modules.get(module_name).unwrap())
    }

    /// v0.70: Load a module with span for error localization
    pub fn load_module_with_span(&mut self, module_name: &str, span: Span) -> Result<&Module> {
        // Check if already loaded
        if self.modules.contains_key(module_name) {
            return Ok(self.modules.get(module_name).unwrap());
        }

        // Resolve file path (with span for better error messages)
        let file_path = self.resolve_module_path_with_span(module_name, span)?;

        // Read the file
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            CompileError::io_error(format!(
                "Failed to read module '{}' at {:?}: {}",
                module_name, file_path, e
            ))
        })?;

        // Tokenize
        let tokens = crate::lexer::tokenize(&source)?;

        // Parse
        let program = crate::parser::parse(module_name, &source, tokens)?;

        // Extract exports (pub items)
        let exports = Self::extract_exports(&program);

        // Create and store the module
        let module = Module {
            name: module_name.to_string(),
            path: file_path,
            program,
            exports,
        };

        self.modules.insert(module_name.to_string(), module);
        self.load_order.push(module_name.to_string());

        Ok(self.modules.get(module_name).unwrap())
    }

    /// Resolve a module name to a file path
    fn resolve_module_path(&self, module_name: &str) -> Result<PathBuf> {
        // Try module_name.bmb in the base directory
        let mut path = self.base_dir.join(format!("{}.bmb", module_name));
        if path.exists() {
            return Ok(path);
        }

        // Try module_name/mod.bmb
        path = self.base_dir.join(module_name).join("mod.bmb");
        if path.exists() {
            return Ok(path);
        }

        // v0.68: Suggest similar module names
        let suggestion = self.suggest_module_name(module_name);
        let hint = format_suggestion_hint(suggestion.as_deref());

        Err(CompileError::resolve_error(format!(
            "Module '{}' not found in {:?}{}",
            module_name, self.base_dir, hint
        )))
    }

    /// v0.70: Resolve module path with span for error localization
    fn resolve_module_path_with_span(&self, module_name: &str, span: Span) -> Result<PathBuf> {
        // Try module_name.bmb in the base directory
        let mut path = self.base_dir.join(format!("{}.bmb", module_name));
        if path.exists() {
            return Ok(path);
        }

        // Try module_name/mod.bmb
        path = self.base_dir.join(module_name).join("mod.bmb");
        if path.exists() {
            return Ok(path);
        }

        // v0.68: Suggest similar module names
        // v0.70: Include span for error localization
        let suggestion = self.suggest_module_name(module_name);
        let hint = format_suggestion_hint(suggestion.as_deref());

        Err(CompileError::resolve_error_at(
            format!("Module '{}' not found in {:?}{}",
                module_name, self.base_dir, hint),
            span,
        ))
    }

    /// v0.68: Find similar module names for suggestions
    fn suggest_module_name(&self, module_name: &str) -> Option<String> {
        // Collect available module names from the base directory
        let mut available_modules = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check for .bmb files
                if path.is_file()
                    && let Some(ext) = path.extension()
                        && ext == "bmb"
                            && let Some(stem) = path.file_stem()
                                && let Some(name) = stem.to_str() {
                                    available_modules.push(name.to_string());
                                }

                // Check for directories with mod.bmb
                if path.is_dir() {
                    let mod_file = path.join("mod.bmb");
                    if mod_file.exists()
                        && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            available_modules.push(name.to_string());
                        }
                }
            }
        }

        // Find similar name using levenshtein distance
        let candidates: Vec<&str> = available_modules.iter().map(|s| s.as_str()).collect();
        find_similar_name(module_name, &candidates).map(|s| s.to_string())
    }

    /// Extract exported (pub) items from a program
    fn extract_exports(program: &Program) -> HashMap<String, ExportedItem> {
        let mut exports = HashMap::new();

        for item in &program.items {
            match item {
                Item::FnDef(fn_def) if fn_def.visibility == Visibility::Public => {
                    exports.insert(
                        fn_def.name.node.clone(),
                        ExportedItem::Function(fn_def.name.node.clone()),
                    );
                }
                Item::StructDef(struct_def) if struct_def.visibility == Visibility::Public => {
                    exports.insert(
                        struct_def.name.node.clone(),
                        ExportedItem::Struct(struct_def.name.node.clone()),
                    );
                }
                Item::EnumDef(enum_def) if enum_def.visibility == Visibility::Public => {
                    exports.insert(
                        enum_def.name.node.clone(),
                        ExportedItem::Enum(enum_def.name.node.clone()),
                    );
                }
                _ => {}
            }
        }

        exports
    }

    /// Resolve all use statements in a program, loading required modules
    pub fn resolve_uses(&mut self, program: &Program) -> Result<ResolvedImports> {
        let mut imports = ResolvedImports::new();

        for item in &program.items {
            if let Item::Use(use_stmt) = item {
                self.resolve_use(use_stmt, &mut imports)?;
            }
        }

        Ok(imports)
    }

    /// Resolve a single use statement
    fn resolve_use(&mut self, use_stmt: &UseStmt, imports: &mut ResolvedImports) -> Result<()> {
        if use_stmt.path.is_empty() {
            // v0.70: Use statement span for empty path error
            return Err(CompileError::resolve_error_at("Empty use path", use_stmt.span));
        }

        // The first segment is the module name
        let module_name = &use_stmt.path[0].node;
        let module_span = use_stmt.path[0].span;

        // Load the module (v0.70: pass span for error localization)
        self.load_module_with_span(module_name, module_span)?;
        let module = self.modules.get(module_name).unwrap();

        // If there's only one segment, import everything (not supported yet)
        if use_stmt.path.len() == 1 {
            // Import all public items from the module
            // v0.74: Use statement span for glob imports
            for (name, item) in &module.exports {
                imports.add_import(name.clone(), module_name.clone(), item.clone(), use_stmt.span);
            }
        } else {
            // Import specific items (e.g., use lexer::Token)
            // The last segment is the item name
            let item_segment = use_stmt.path.last().unwrap();
            let item_name = &item_segment.node;
            let item_span = item_segment.span;

            if let Some(item) = module.exports.get(item_name) {
                // v0.74: Use item span for specific imports
                imports.add_import(item_name.clone(), module_name.clone(), item.clone(), item_span);
            } else {
                // v0.68: Suggest similar export names
                // v0.70: Include span for error localization
                let export_names: Vec<&str> = module.exports.keys().map(|s| s.as_str()).collect();
                let suggestion = find_similar_name(item_name, &export_names);
                let hint = format_suggestion_hint(suggestion);

                return Err(CompileError::resolve_error_at(
                    format!("Item '{}' not found in module '{}'{}",
                        item_name, module_name, hint),
                    item_span,
                ));
            }
        }

        Ok(())
    }

    /// Get a loaded module by name
    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    /// Get all loaded modules in load order
    pub fn modules_in_order(&self) -> impl Iterator<Item = &Module> {
        self.load_order
            .iter()
            .filter_map(|name| self.modules.get(name))
    }

    /// Get the number of loaded modules
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

/// v0.74: Import info for tracking usage
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Module the import came from
    pub module: String,
    /// The exported item
    pub item: ExportedItem,
    /// Span of the import statement
    pub span: Span,
    /// Whether this import has been used
    pub used: bool,
}

/// Collection of resolved imports from use statements
#[derive(Debug, Default)]
pub struct ResolvedImports {
    /// Imported items: name -> ImportInfo
    imports: HashMap<String, ImportInfo>,
}

impl ResolvedImports {
    /// Create a new empty imports collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an import (v0.74: now tracks span for unused import warnings)
    pub fn add_import(&mut self, name: String, module: String, item: ExportedItem, span: Span) {
        self.imports.insert(name, ImportInfo {
            module,
            item,
            span,
            used: false,
        });
    }

    /// Check if a name is imported
    pub fn is_imported(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    /// Get the module an import came from
    pub fn get_import_module(&self, name: &str) -> Option<&str> {
        self.imports.get(name).map(|info| info.module.as_str())
    }

    /// Get all imports (v0.74: returns ImportInfo)
    pub fn all_imports(&self) -> impl Iterator<Item = (&String, &ImportInfo)> {
        self.imports.iter()
    }

    /// Get the count of imports
    pub fn len(&self) -> usize {
        self.imports.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }

    /// v0.74: Mark an import as used
    pub fn mark_used(&mut self, name: &str) {
        if let Some(info) = self.imports.get_mut(name) {
            info.used = true;
        }
    }

    /// v0.74: Get all unused imports (for warning generation)
    /// Returns: (name, span) pairs for unused imports
    pub fn get_unused(&self) -> Vec<(String, Span)> {
        self.imports
            .iter()
            .filter(|(name, info)| !info.used && !name.starts_with('_'))
            .map(|(name, info)| (name.clone(), info.span))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_creation() {
        let resolver = Resolver::new(".");
        assert_eq!(resolver.module_count(), 0);
    }

    #[test]
    fn test_resolved_imports() {
        let mut imports = ResolvedImports::new();
        // v0.74: add_import now requires span
        let dummy_span = Span { start: 0, end: 0 };
        imports.add_import(
            "Token".to_string(),
            "lexer".to_string(),
            ExportedItem::Struct("Token".to_string()),
            dummy_span,
        );

        assert!(imports.is_imported("Token"));
        assert!(!imports.is_imported("Foo"));
        assert_eq!(imports.get_import_module("Token"), Some("lexer"));
        assert_eq!(imports.len(), 1);
    }

    #[test]
    fn test_unused_import_tracking() {
        let mut imports = ResolvedImports::new();
        let span1 = Span { start: 0, end: 10 };
        let span2 = Span { start: 20, end: 30 };

        imports.add_import(
            "Token".to_string(),
            "lexer".to_string(),
            ExportedItem::Struct("Token".to_string()),
            span1,
        );
        imports.add_import(
            "Parser".to_string(),
            "parser".to_string(),
            ExportedItem::Struct("Parser".to_string()),
            span2,
        );

        // Mark Token as used
        imports.mark_used("Token");

        // Get unused imports - should only be Parser
        let unused = imports.get_unused();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].0, "Parser");
        assert_eq!(unused[0].1, span2);
    }

    #[test]
    fn test_underscore_prefix_not_reported() {
        let mut imports = ResolvedImports::new();
        let span = Span { start: 0, end: 10 };

        imports.add_import(
            "_unused".to_string(),
            "lexer".to_string(),
            ExportedItem::Struct("_unused".to_string()),
            span,
        );

        // Underscore-prefixed imports should not be reported as unused
        let unused = imports.get_unused();
        assert!(unused.is_empty());
    }
}
