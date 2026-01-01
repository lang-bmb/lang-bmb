//! Module Resolver for BMB
//!
//! Handles multi-file compilation by resolving `use` statements and
//! loading/parsing modules from the file system.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ast::{Item, Program, UseStmt, Visibility};
use crate::error::{CompileError, Result};

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

        Err(CompileError::resolve_error(format!(
            "Module '{}' not found in {:?}",
            module_name, self.base_dir
        )))
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
            return Err(CompileError::resolve_error("Empty use path"));
        }

        // The first segment is the module name
        let module_name = &use_stmt.path[0].node;

        // Load the module
        self.load_module(module_name)?;
        let module = self.modules.get(module_name).unwrap();

        // If there's only one segment, import everything (not supported yet)
        if use_stmt.path.len() == 1 {
            // Import all public items from the module
            for (name, item) in &module.exports {
                imports.add_import(name.clone(), module_name.clone(), item.clone());
            }
        } else {
            // Import specific items (e.g., use lexer::Token)
            // The last segment is the item name
            let item_name = &use_stmt.path.last().unwrap().node;

            if let Some(item) = module.exports.get(item_name) {
                imports.add_import(item_name.clone(), module_name.clone(), item.clone());
            } else {
                return Err(CompileError::resolve_error(format!(
                    "Item '{}' not found in module '{}'",
                    item_name, module_name
                )));
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

/// Collection of resolved imports from use statements
#[derive(Debug, Default)]
pub struct ResolvedImports {
    /// Imported items: name -> (module, item)
    imports: HashMap<String, (String, ExportedItem)>,
}

impl ResolvedImports {
    /// Create a new empty imports collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an import
    pub fn add_import(&mut self, name: String, module: String, item: ExportedItem) {
        self.imports.insert(name, (module, item));
    }

    /// Check if a name is imported
    pub fn is_imported(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    /// Get the module an import came from
    pub fn get_import_module(&self, name: &str) -> Option<&str> {
        self.imports.get(name).map(|(m, _)| m.as_str())
    }

    /// Get all imports
    pub fn all_imports(&self) -> impl Iterator<Item = (&String, &(String, ExportedItem))> {
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
        imports.add_import(
            "Token".to_string(),
            "lexer".to_string(),
            ExportedItem::Struct("Token".to_string()),
        );

        assert!(imports.is_imported("Token"));
        assert!(!imports.is_imported("Foo"));
        assert_eq!(imports.get_import_module("Token"), Some("lexer"));
        assert_eq!(imports.len(), 1);
    }
}
