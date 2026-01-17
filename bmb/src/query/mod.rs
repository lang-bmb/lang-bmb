//! BMB AI Query System - v0.48.0
//!
//! Query interface for AI tools to understand BMB projects.
//! RFC-0001: AI-Native Code Query System
//!
//! ## Output Formats (v0.48)
//! - `json`: Structured JSON (default, programmatic parsing)
//! - `compact`: Single-line format (space-efficient)
//! - `llm`: LLM-optimized format (token-efficient, semantic sections)

use crate::index::{FunctionEntry, ProjectIndex, SymbolEntry, SymbolKind, TypeEntry};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Query result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<T>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Query error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<String>,
}

/// Query engine for the index
pub struct QueryEngine {
    index: ProjectIndex,
}

impl QueryEngine {
    pub fn new(index: ProjectIndex) -> Self {
        Self { index }
    }

    /// Query symbols by pattern
    pub fn query_symbols(&self, pattern: &str, kind: Option<SymbolKind>, pub_only: bool) -> QueryResult<SymbolEntry> {
        let pattern_lower = pattern.to_lowercase();

        let matches: Vec<SymbolEntry> = self
            .index
            .symbols
            .iter()
            .filter(|s| {
                let name_match = s.name.to_lowercase().contains(&pattern_lower);
                let kind_match = kind.is_none_or(|k| s.kind == k);
                let pub_match = !pub_only || s.is_pub;
                name_match && kind_match && pub_match
            })
            .cloned()
            .collect();

        if matches.is_empty() {
            QueryResult {
                query: pattern.to_string(),
                matches: None,
                result: None,
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("No symbols matching '{}' found", pattern),
                    suggestions: self.suggest_symbols(pattern),
                }),
            }
        } else {
            QueryResult {
                query: pattern.to_string(),
                matches: Some(matches),
                result: None,
                error: None,
            }
        }
    }

    /// Query function by name
    pub fn query_function(&self, name: &str) -> QueryResult<FunctionEntry> {
        let func = self.index.functions.iter().find(|f| f.name == name);

        match func {
            Some(f) => QueryResult {
                query: name.to_string(),
                matches: None,
                result: Some(f.clone()),
                error: None,
            },
            None => QueryResult {
                query: name.to_string(),
                matches: None,
                result: None,
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Function '{}' not found", name),
                    suggestions: self.suggest_functions(name),
                }),
            },
        }
    }

    /// Query functions with filters
    pub fn query_functions(
        &self,
        has_pre: Option<bool>,
        has_post: Option<bool>,
        recursive: Option<bool>,
        pub_only: bool,
    ) -> QueryResult<FunctionEntry> {
        let matches: Vec<FunctionEntry> = self
            .index
            .functions
            .iter()
            .filter(|f| {
                let pre_match = has_pre.is_none_or(|hp| {
                    hp == f.contracts.as_ref().is_some_and(|c| c.pre.is_some())
                });
                let post_match = has_post.is_none_or(|hp| {
                    hp == f.contracts.as_ref().is_some_and(|c| c.post.is_some())
                });
                let recursive_match = recursive.is_none_or(|r| {
                    r == f.body_info.as_ref().is_some_and(|b| b.recursive)
                });
                let pub_match = !pub_only || f.is_pub;
                pre_match && post_match && recursive_match && pub_match
            })
            .cloned()
            .collect();

        QueryResult {
            query: "functions".to_string(),
            matches: Some(matches),
            result: None,
            error: None,
        }
    }

    /// Query type by name
    pub fn query_type(&self, name: &str) -> QueryResult<TypeEntry> {
        let ty = self.index.types.iter().find(|t| t.name == name);

        match ty {
            Some(t) => QueryResult {
                query: name.to_string(),
                matches: None,
                result: Some(t.clone()),
                error: None,
            },
            None => QueryResult {
                query: name.to_string(),
                matches: None,
                result: None,
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Type '{}' not found", name),
                    suggestions: self.suggest_types(name),
                }),
            },
        }
    }

    /// Query types with filters
    pub fn query_types(&self, kind: Option<&str>, pub_only: bool) -> QueryResult<TypeEntry> {
        let matches: Vec<TypeEntry> = self
            .index
            .types
            .iter()
            .filter(|t| {
                let kind_match = kind.is_none_or(|k| t.kind == k);
                let pub_match = !pub_only || t.is_pub;
                kind_match && pub_match
            })
            .cloned()
            .collect();

        QueryResult {
            query: "types".to_string(),
            matches: Some(matches),
            result: None,
            error: None,
        }
    }

    /// Get project metrics
    pub fn query_metrics(&self) -> ProjectMetrics {
        let functions_with_pre = self
            .index
            .functions
            .iter()
            .filter(|f| f.contracts.as_ref().is_some_and(|c| c.pre.is_some()))
            .count();

        let functions_with_post = self
            .index
            .functions
            .iter()
            .filter(|f| f.contracts.as_ref().is_some_and(|c| c.post.is_some()))
            .count();

        let functions_with_both = self
            .index
            .functions
            .iter()
            .filter(|f| {
                f.contracts
                    .as_ref()
                    .is_some_and(|c| c.pre.is_some() && c.post.is_some())
            })
            .count();

        let recursive_functions = self
            .index
            .functions
            .iter()
            .filter(|f| f.body_info.as_ref().is_some_and(|b| b.recursive))
            .count();

        ProjectMetrics {
            project: ProjectStats {
                files: self.index.manifest.files,
                functions: self.index.manifest.functions,
                types: self.index.manifest.types,
                structs: self.index.manifest.structs,
                enums: self.index.manifest.enums,
            },
            contract_usage: ContractUsage {
                functions_with_pre,
                functions_with_post,
                functions_with_both,
            },
            body_analysis: BodyAnalysis {
                recursive_functions,
            },
        }
    }

    fn suggest_symbols(&self, pattern: &str) -> Vec<String> {
        let pattern_lower = pattern.to_lowercase();
        self.index
            .symbols
            .iter()
            .filter(|s| {
                let name_lower = s.name.to_lowercase();
                levenshtein(&name_lower, &pattern_lower) <= 3
            })
            .take(5)
            .map(|s| s.name.clone())
            .collect()
    }

    fn suggest_functions(&self, name: &str) -> Vec<String> {
        let name_lower = name.to_lowercase();
        self.index
            .functions
            .iter()
            .filter(|f| {
                let fn_lower = f.name.to_lowercase();
                levenshtein(&fn_lower, &name_lower) <= 3
            })
            .take(5)
            .map(|f| f.name.clone())
            .collect()
    }

    fn suggest_types(&self, name: &str) -> Vec<String> {
        let name_lower = name.to_lowercase();
        self.index
            .types
            .iter()
            .filter(|t| {
                let ty_lower = t.name.to_lowercase();
                levenshtein(&ty_lower, &name_lower) <= 3
            })
            .take(5)
            .map(|t| t.name.clone())
            .collect()
    }
}

/// Project metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub project: ProjectStats,
    pub contract_usage: ContractUsage,
    pub body_analysis: BodyAnalysis,
}

/// Project statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub files: usize,
    pub functions: usize,
    pub types: usize,
    pub structs: usize,
    pub enums: usize,
}

/// Contract usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractUsage {
    pub functions_with_pre: usize,
    pub functions_with_post: usize,
    pub functions_with_both: usize,
}

/// Body analysis statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyAnalysis {
    pub recursive_functions: usize,
}

// =============================================================================
// v0.47 - RFC-0001 Phase 2: Dependency and Contract Queries
// =============================================================================

/// Dependency query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepsResult {
    pub target: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<CallInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub called_by: Vec<CallerInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub type_deps: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Call information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallInfo {
    pub name: String,
    pub count: usize,
    #[serde(skip_serializing_if = "is_false")]
    pub recursive: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Caller information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallerInfo {
    pub name: String,
    pub file: String,
    pub line: usize,
}

/// Contract query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub name: String,
    pub file: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<ContractDetail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post: Option<Vec<ContractDetail>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Contract detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDetail {
    pub expr: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub quantifiers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<String>,
    #[serde(skip_serializing_if = "is_false")]
    pub uses_old: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub uses_ret: bool,
}

// =============================================================================
// v0.48 - RFC-0001 Phase 2-3: Context Generation
// =============================================================================

/// AI Context query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResult {
    pub target: TargetInfo,
    pub dependencies: DependencyContext,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependents: Vec<DependentInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related_tests: Vec<TestInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Target information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub kind: String,
    pub name: String,
    pub file: String,
    pub line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contracts_summary: Option<String>,
}

/// Dependency context for AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyContext {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<TargetInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<TargetInfo>,
}

/// Dependent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependentInfo {
    pub name: String,
    pub file: String,
    pub line: usize,
}

/// Test information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInfo {
    pub name: String,
    pub file: String,
    pub line: usize,
}

impl QueryEngine {
    /// v0.47: Query dependencies for a target
    pub fn query_deps(&self, target: &str, reverse: bool, transitive: bool) -> DepsResult {
        // Parse target format: "fn:name" or "type:name"
        let (kind, name) = if let Some(idx) = target.find(':') {
            (&target[..idx], &target[idx + 1..])
        } else {
            ("fn", target)
        };

        match kind {
            "fn" => self.query_function_deps(name, reverse, transitive),
            "type" => self.query_type_deps(name, reverse),
            _ => DepsResult {
                target: target.to_string(),
                calls: Vec::new(),
                called_by: Vec::new(),
                type_deps: Vec::new(),
                error: Some(QueryError {
                    code: "INVALID_TARGET".to_string(),
                    message: format!("Unknown target kind: {}. Use 'fn:name' or 'type:name'", kind),
                    suggestions: vec!["fn:main".to_string(), "type:MyStruct".to_string()],
                }),
            },
        }
    }

    fn query_function_deps(&self, name: &str, reverse: bool, transitive: bool) -> DepsResult {
        // Find the target function
        let func = self.index.functions.iter().find(|f| f.name == name);

        match func {
            Some(f) => {
                let mut calls = Vec::new();
                let mut called_by = Vec::new();

                // Get direct calls from body_info
                if let Some(body) = &f.body_info {
                    for call_name in &body.calls {
                        let is_recursive = call_name == name;
                        let count = body.calls.iter().filter(|c| *c == call_name).count();
                        if !calls.iter().any(|c: &CallInfo| c.name == *call_name) {
                            calls.push(CallInfo {
                                name: call_name.clone(),
                                count,
                                recursive: is_recursive,
                            });
                        }
                    }
                }

                // Get transitive calls if requested
                if transitive {
                    let mut visited = std::collections::HashSet::new();
                    visited.insert(name.to_string());
                    self.collect_transitive_calls(&calls, &mut visited, &mut calls.clone());
                }

                // Find who calls this function (reverse deps)
                if reverse {
                    for other_fn in &self.index.functions {
                        if let Some(body) = &other_fn.body_info
                            && body.calls.contains(&name.to_string())
                            && other_fn.name != name
                        {
                            called_by.push(CallerInfo {
                                name: other_fn.name.clone(),
                                file: other_fn.file.clone(),
                                line: other_fn.line,
                            });
                        }
                    }
                }

                DepsResult {
                    target: format!("fn:{}", name),
                    calls,
                    called_by,
                    type_deps: Vec::new(), // TODO: Extract type dependencies
                    error: None,
                }
            }
            None => DepsResult {
                target: format!("fn:{}", name),
                calls: Vec::new(),
                called_by: Vec::new(),
                type_deps: Vec::new(),
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Function '{}' not found", name),
                    suggestions: self.suggest_functions(name),
                }),
            },
        }
    }

    fn collect_transitive_calls(
        &self,
        current_calls: &[CallInfo],
        visited: &mut std::collections::HashSet<String>,
        all_calls: &mut Vec<CallInfo>,
    ) {
        for call in current_calls {
            if visited.contains(&call.name) {
                continue;
            }
            visited.insert(call.name.clone());

            if let Some(func) = self.index.functions.iter().find(|f| f.name == call.name)
                && let Some(body) = &func.body_info
            {
                for nested_call in &body.calls {
                    if !visited.contains(nested_call)
                        && !all_calls.iter().any(|c| c.name == *nested_call)
                    {
                        all_calls.push(CallInfo {
                            name: nested_call.clone(),
                            count: 1,
                            recursive: false,
                        });
                    }
                }
            }
        }
    }

    fn query_type_deps(&self, name: &str, reverse: bool) -> DepsResult {
        let type_entry = self.index.types.iter().find(|t| t.name == name);

        match type_entry {
            Some(_t) => {
                let mut called_by = Vec::new();

                if reverse {
                    // Find functions that use this type in their signature
                    for func in &self.index.functions {
                        let sig = &func.signature;
                        let uses_type = sig.params.iter().any(|p| p.ty.contains(name))
                            || sig.return_type.contains(name);

                        if uses_type {
                            called_by.push(CallerInfo {
                                name: func.name.clone(),
                                file: func.file.clone(),
                                line: func.line,
                            });
                        }
                    }
                }

                DepsResult {
                    target: format!("type:{}", name),
                    calls: Vec::new(),
                    called_by,
                    type_deps: Vec::new(),
                    error: None,
                }
            }
            None => DepsResult {
                target: format!("type:{}", name),
                calls: Vec::new(),
                called_by: Vec::new(),
                type_deps: Vec::new(),
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Type '{}' not found", name),
                    suggestions: self.suggest_types(name),
                }),
            },
        }
    }

    /// v0.47: Query contract details for a function
    pub fn query_contract(&self, name: &str, uses_old_filter: bool) -> ContractResult {
        let func = self.index.functions.iter().find(|f| f.name == name);

        match func {
            Some(f) => {
                let pre = f.contracts.as_ref().and_then(|c| {
                    c.pre.as_ref().map(|pre_list| {
                        pre_list
                            .iter()
                            .filter(|p| !uses_old_filter || p.uses_old)
                            .map(|p| ContractDetail {
                                expr: p.expr.clone(),
                                quantifiers: p.quantifiers.clone(),
                                calls: p.calls.clone(),
                                uses_old: p.uses_old,
                                uses_ret: p.uses_ret,
                            })
                            .collect::<Vec<_>>()
                    })
                }).filter(|v| !v.is_empty());

                let post = f.contracts.as_ref().and_then(|c| {
                    c.post.as_ref().map(|post_list| {
                        post_list
                            .iter()
                            .filter(|p| !uses_old_filter || p.uses_old)
                            .map(|p| ContractDetail {
                                expr: p.expr.clone(),
                                quantifiers: p.quantifiers.clone(),
                                calls: p.calls.clone(),
                                uses_old: p.uses_old,
                                uses_ret: p.uses_ret,
                            })
                            .collect::<Vec<_>>()
                    })
                }).filter(|v| !v.is_empty());

                ContractResult {
                    name: f.name.clone(),
                    file: f.file.clone(),
                    line: f.line,
                    pre,
                    post,
                    error: None,
                }
            }
            None => ContractResult {
                name: name.to_string(),
                file: String::new(),
                line: 0,
                pre: None,
                post: None,
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Function '{}' not found", name),
                    suggestions: self.suggest_functions(name),
                }),
            },
        }
    }

    /// v0.48: Generate AI context for a target
    pub fn query_context(&self, target: &str, depth: usize, include_tests: bool) -> ContextResult {
        // Parse target format
        let (kind, name) = if let Some(idx) = target.find(':') {
            (&target[..idx], &target[idx + 1..])
        } else {
            ("fn", target)
        };

        match kind {
            "fn" => self.query_function_context(name, depth, include_tests),
            "type" => self.query_type_context(name, include_tests),
            _ => ContextResult {
                target: TargetInfo {
                    kind: kind.to_string(),
                    name: name.to_string(),
                    file: String::new(),
                    line: 0,
                    signature: None,
                    contracts_summary: None,
                },
                dependencies: DependencyContext {
                    functions: Vec::new(),
                    types: Vec::new(),
                },
                dependents: Vec::new(),
                related_tests: Vec::new(),
                error: Some(QueryError {
                    code: "INVALID_TARGET".to_string(),
                    message: format!("Unknown target kind: {}", kind),
                    suggestions: vec!["fn:main".to_string()],
                }),
            },
        }
    }

    fn query_function_context(&self, name: &str, depth: usize, include_tests: bool) -> ContextResult {
        let func = self.index.functions.iter().find(|f| f.name == name);

        match func {
            Some(f) => {
                // Build target info
                let contracts_summary = f.contracts.as_ref().map(|c| {
                    let mut parts = Vec::new();
                    if let Some(pre) = &c.pre {
                        for p in pre {
                            parts.push(format!("pre: {}", p.expr));
                        }
                    }
                    if let Some(post) = &c.post {
                        for p in post {
                            parts.push(format!("post: {}", p.expr));
                        }
                    }
                    parts.join(", ")
                });

                let sig_str = format!(
                    "fn({}) -> {}",
                    f.signature.params.iter().map(|p| format!("{}: {}", p.name, p.ty)).collect::<Vec<_>>().join(", "),
                    f.signature.return_type
                );

                let target = TargetInfo {
                    kind: "fn".to_string(),
                    name: f.name.clone(),
                    file: f.file.clone(),
                    line: f.line,
                    signature: Some(sig_str),
                    contracts_summary,
                };

                // Collect dependencies
                let mut dep_functions = Vec::new();
                let mut dep_types = Vec::new();
                let mut visited = std::collections::HashSet::new();
                visited.insert(name.to_string());

                if let Some(body) = &f.body_info {
                    self.collect_context_deps(&body.calls, depth, &mut visited, &mut dep_functions);
                }

                // Collect type dependencies from signature
                for param in &f.signature.params {
                    self.add_type_to_context(&param.ty, &mut dep_types);
                }
                self.add_type_to_context(&f.signature.return_type, &mut dep_types);

                // Find dependents (reverse deps)
                let mut dependents = Vec::new();
                for other_fn in &self.index.functions {
                    if let Some(body) = &other_fn.body_info
                        && body.calls.contains(&name.to_string())
                        && other_fn.name != name
                    {
                        dependents.push(DependentInfo {
                            name: other_fn.name.clone(),
                            file: other_fn.file.clone(),
                            line: other_fn.line,
                        });
                    }
                }

                // Find related tests
                let related_tests = if include_tests {
                    self.index
                        .functions
                        .iter()
                        .filter(|tf| {
                            tf.name.starts_with("test_") &&
                            tf.body_info.as_ref().is_some_and(|b| b.calls.contains(&name.to_string()))
                        })
                        .map(|tf| TestInfo {
                            name: tf.name.clone(),
                            file: tf.file.clone(),
                            line: tf.line,
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                ContextResult {
                    target,
                    dependencies: DependencyContext {
                        functions: dep_functions,
                        types: dep_types,
                    },
                    dependents,
                    related_tests,
                    error: None,
                }
            }
            None => ContextResult {
                target: TargetInfo {
                    kind: "fn".to_string(),
                    name: name.to_string(),
                    file: String::new(),
                    line: 0,
                    signature: None,
                    contracts_summary: None,
                },
                dependencies: DependencyContext {
                    functions: Vec::new(),
                    types: Vec::new(),
                },
                dependents: Vec::new(),
                related_tests: Vec::new(),
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Function '{}' not found", name),
                    suggestions: self.suggest_functions(name),
                }),
            },
        }
    }

    fn collect_context_deps(
        &self,
        calls: &[String],
        depth: usize,
        visited: &mut std::collections::HashSet<String>,
        dep_functions: &mut Vec<TargetInfo>,
    ) {
        if depth == 0 {
            return;
        }

        for call_name in calls {
            if visited.contains(call_name) {
                continue;
            }
            visited.insert(call_name.clone());

            if let Some(func) = self.index.functions.iter().find(|f| &f.name == call_name) {
                let contracts_summary = func.contracts.as_ref().map(|c| {
                    let mut parts = Vec::new();
                    if let Some(pre) = &c.pre {
                        for p in pre {
                            parts.push(format!("pre: {}", p.expr));
                        }
                    }
                    if let Some(post) = &c.post {
                        for p in post {
                            parts.push(format!("post: {}", p.expr));
                        }
                    }
                    parts.join(", ")
                });

                let sig_str = format!(
                    "fn({}) -> {}",
                    func.signature.params.iter().map(|p| format!("{}: {}", p.name, p.ty)).collect::<Vec<_>>().join(", "),
                    func.signature.return_type
                );

                dep_functions.push(TargetInfo {
                    kind: "fn".to_string(),
                    name: func.name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    signature: Some(sig_str),
                    contracts_summary,
                });

                // Recurse if depth allows
                if depth > 1 && let Some(body) = &func.body_info {
                    self.collect_context_deps(&body.calls, depth - 1, visited, dep_functions);
                }
            }
        }
    }

    fn add_type_to_context(&self, type_name: &str, dep_types: &mut Vec<TargetInfo>) {
        // Extract base type name (remove generics, references, etc.)
        let base_name = type_name
            .trim_start_matches('&')
            .trim_start_matches("mut ")
            .split('<')
            .next()
            .unwrap_or(type_name);

        // Skip primitive types
        if matches!(base_name, "i32" | "i64" | "u32" | "u64" | "f64" | "bool" | "String" | "()" | "!" | "char") {
            return;
        }

        // Check if already added
        if dep_types.iter().any(|t| t.name == base_name) {
            return;
        }

        // Find type in index
        if let Some(type_entry) = self.index.types.iter().find(|t| t.name == base_name) {
            dep_types.push(TargetInfo {
                kind: type_entry.kind.clone(),
                name: type_entry.name.clone(),
                file: type_entry.file.clone(),
                line: type_entry.line,
                signature: None,
                contracts_summary: None,
            });
        }
    }

    fn query_type_context(&self, name: &str, include_tests: bool) -> ContextResult {
        let type_entry = self.index.types.iter().find(|t| t.name == name);

        match type_entry {
            Some(t) => {
                let target = TargetInfo {
                    kind: t.kind.clone(),
                    name: t.name.clone(),
                    file: t.file.clone(),
                    line: t.line,
                    signature: None,
                    contracts_summary: None,
                };

                // Find functions that use this type
                let mut dep_functions = Vec::new();
                for func in &self.index.functions {
                    let sig = &func.signature;
                    let uses_type = sig.params.iter().any(|p| p.ty.contains(name))
                        || sig.return_type.contains(name);

                    if uses_type {
                        let sig_str = format!(
                            "fn({}) -> {}",
                            func.signature.params.iter().map(|p| format!("{}: {}", p.name, p.ty)).collect::<Vec<_>>().join(", "),
                            func.signature.return_type
                        );
                        dep_functions.push(TargetInfo {
                            kind: "fn".to_string(),
                            name: func.name.clone(),
                            file: func.file.clone(),
                            line: func.line,
                            signature: Some(sig_str),
                            contracts_summary: None,
                        });
                    }
                }

                let related_tests = if include_tests {
                    self.index
                        .functions
                        .iter()
                        .filter(|tf| {
                            tf.name.starts_with("test_") &&
                            (tf.signature.params.iter().any(|p| p.ty.contains(name)) ||
                             tf.signature.return_type.contains(name))
                        })
                        .map(|tf| TestInfo {
                            name: tf.name.clone(),
                            file: tf.file.clone(),
                            line: tf.line,
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                ContextResult {
                    target,
                    dependencies: DependencyContext {
                        functions: dep_functions,
                        types: Vec::new(),
                    },
                    dependents: Vec::new(),
                    related_tests,
                    error: None,
                }
            }
            None => ContextResult {
                target: TargetInfo {
                    kind: "type".to_string(),
                    name: name.to_string(),
                    file: String::new(),
                    line: 0,
                    signature: None,
                    contracts_summary: None,
                },
                dependencies: DependencyContext {
                    functions: Vec::new(),
                    types: Vec::new(),
                },
                dependents: Vec::new(),
                related_tests: Vec::new(),
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Type '{}' not found", name),
                    suggestions: self.suggest_types(name),
                }),
            },
        }
    }
}

// =============================================================================
// v0.48 - RFC-0001 Phase 2-3: Signature Query
// =============================================================================

/// Signature query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigResult {
    pub query: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub matches: Vec<SigMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Signature match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigMatch {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param_match: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_match: Option<bool>,
}

// =============================================================================
// v0.49 - RFC-0001 Phase 3: Batch and Impact Queries
// =============================================================================

/// Batch query request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQuery {
    #[serde(rename = "type")]
    pub query_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transitive: Option<bool>,
}

/// Batch query file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchQueryFile {
    pub queries: Vec<BatchQuery>,
}

/// Batch query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub results: Vec<BatchResultEntry>,
}

/// Single batch result entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultEntry {
    pub query: usize,
    pub result: serde_json::Value,
}

/// Impact analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactResult {
    pub target: String,
    pub change: String,
    pub impact: ImpactAnalysis,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Impact analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub breaking: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub direct_callers: Vec<CallerInfo>,
    pub transitive_callers: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files_affected: Vec<String>,
}

impl QueryEngine {
    /// v0.48: Query functions by signature pattern
    pub fn query_signature(&self, pattern: &str, accepts: Option<&str>, returns: Option<&str>) -> SigResult {
        let mut matches = Vec::new();

        for func in &self.index.functions {
            let sig_str = format!(
                "({}) -> {}",
                func.signature.params.iter().map(|p| format!("{}: {}", p.name, p.ty)).collect::<Vec<_>>().join(", "),
                func.signature.return_type
            );

            // Check pattern match
            let pattern_match = pattern.is_empty() || sig_str.contains(pattern);

            // Check accepts filter
            let (accepts_match, param_match) = if let Some(accepts_type) = accepts {
                let matched_param = func.signature.params.iter().find(|p| p.ty.contains(accepts_type));
                (matched_param.is_some(), matched_param.map(|p| p.name.clone()))
            } else {
                (true, None)
            };

            // Check returns filter
            let returns_match = returns.is_none_or(|ret_type| func.signature.return_type.contains(ret_type));

            if pattern_match && accepts_match && returns_match {
                matches.push(SigMatch {
                    name: func.name.clone(),
                    file: func.file.clone(),
                    line: func.line,
                    signature: format!("fn{}", sig_str),
                    param_match,
                    return_match: returns.map(|_| true),
                });
            }
        }

        let query = if !pattern.is_empty() {
            pattern.to_string()
        } else if let Some(a) = accepts {
            format!("--accepts {}", a)
        } else if let Some(r) = returns {
            format!("--returns {}", r)
        } else {
            "all".to_string()
        };

        SigResult {
            query,
            matches,
            error: None,
        }
    }

    /// v0.49: Run batch queries from file
    pub fn query_batch(&self, file: &Path) -> Result<BatchResult, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file)?;
        let batch: BatchQueryFile = serde_json::from_str(&content)?;

        let mut results = Vec::new();

        for (idx, query) in batch.queries.iter().enumerate() {
            let result = match query.query_type.as_str() {
                "fn" => {
                    if let Some(name) = &query.name {
                        serde_json::to_value(self.query_function(name))?
                    } else {
                        serde_json::to_value(self.query_functions(None, None, None, false))?
                    }
                }
                "type" => {
                    if let Some(name) = &query.name {
                        serde_json::to_value(self.query_type(name))?
                    } else {
                        serde_json::to_value(self.query_types(None, false))?
                    }
                }
                "deps" => {
                    if let Some(target) = &query.target {
                        let transitive = query.transitive.unwrap_or(false);
                        serde_json::to_value(self.query_deps(target, false, transitive))?
                    } else {
                        serde_json::json!({"error": "deps requires target"})
                    }
                }
                "contract" => {
                    if let Some(name) = &query.name {
                        serde_json::to_value(self.query_contract(name, false))?
                    } else {
                        serde_json::json!({"error": "contract requires name"})
                    }
                }
                "metrics" => serde_json::to_value(self.query_metrics())?,
                _ => serde_json::json!({"error": format!("unknown query type: {}", query.query_type)}),
            };

            results.push(BatchResultEntry { query: idx, result });
        }

        Ok(BatchResult { results })
    }

    /// v0.49: Analyze change impact
    pub fn query_impact(&self, target: &str, change: &str) -> ImpactResult {
        // Parse target
        let (kind, name) = if let Some(idx) = target.find(':') {
            (&target[..idx], &target[idx + 1..])
        } else {
            ("fn", target)
        };

        if kind != "fn" {
            return ImpactResult {
                target: target.to_string(),
                change: change.to_string(),
                impact: ImpactAnalysis {
                    breaking: false,
                    direct_callers: Vec::new(),
                    transitive_callers: 0,
                    files_affected: Vec::new(),
                },
                error: Some(QueryError {
                    code: "UNSUPPORTED".to_string(),
                    message: "Impact analysis currently only supports functions".to_string(),
                    suggestions: vec!["fn:function_name".to_string()],
                }),
            };
        }

        // Find the function
        let func = self.index.functions.iter().find(|f| f.name == name);

        match func {
            Some(_f) => {
                // Find direct callers
                let mut direct_callers = Vec::new();
                let mut files_affected = std::collections::HashSet::new();

                for other_fn in &self.index.functions {
                    if let Some(body) = &other_fn.body_info
                        && body.calls.contains(&name.to_string())
                        && other_fn.name != name
                    {
                        direct_callers.push(CallerInfo {
                            name: other_fn.name.clone(),
                            file: other_fn.file.clone(),
                            line: other_fn.line,
                        });
                        files_affected.insert(other_fn.file.clone());
                    }
                }

                // Count transitive callers (simplified: just count direct for now)
                let transitive_callers = direct_callers.len();

                // Determine if breaking based on change description
                let breaking = change.contains("add param")
                    || change.contains("remove")
                    || change.contains("rename")
                    || change.contains("change type");

                ImpactResult {
                    target: target.to_string(),
                    change: change.to_string(),
                    impact: ImpactAnalysis {
                        breaking,
                        direct_callers,
                        transitive_callers,
                        files_affected: files_affected.into_iter().collect(),
                    },
                    error: None,
                }
            }
            None => ImpactResult {
                target: target.to_string(),
                change: change.to_string(),
                impact: ImpactAnalysis {
                    breaking: false,
                    direct_callers: Vec::new(),
                    transitive_callers: 0,
                    files_affected: Vec::new(),
                },
                error: Some(QueryError {
                    code: "NOT_FOUND".to_string(),
                    message: format!("Function '{}' not found", name),
                    suggestions: self.suggest_functions(name),
                }),
            },
        }
    }
}

// =============================================================================
// v0.48 - Output Format Functions
// =============================================================================

/// Format output based on format type (v0.48 - RFC-0001)
pub fn format_output<T: Serialize>(data: &T, format: &str) -> Result<String, serde_json::Error> {
    match format {
        "compact" => serde_json::to_string(data),
        "llm" => format_llm(data),
        _ => serde_json::to_string_pretty(data), // json (default)
    }
}

/// LLM-optimized output format (v0.48)
/// Designed for token efficiency based on research:
/// - Clear section headers
/// - No redundant whitespace
/// - Semantic grouping
fn format_llm<T: Serialize>(data: &T) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(data)?;
    Ok(format_llm_value(&value, 0))
}

fn format_llm_value(value: &serde_json::Value, indent: usize) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let mut lines = Vec::new();
            for (key, val) in map {
                let key_upper = key.to_uppercase().replace('_', " ");
                match val {
                    serde_json::Value::Array(arr) if !arr.is_empty() => {
                        lines.push(format!("{}:", key_upper));
                        for item in arr {
                            let item_str = format_llm_value(item, indent + 1);
                            for line in item_str.lines() {
                                lines.push(format!("  {}", line));
                            }
                        }
                    }
                    serde_json::Value::Object(_) => {
                        lines.push(format!("{}:", key_upper));
                        let nested = format_llm_value(val, indent + 1);
                        for line in nested.lines() {
                            lines.push(format!("  {}", line));
                        }
                    }
                    serde_json::Value::String(s) => {
                        lines.push(format!("{}: {}", key_upper, s));
                    }
                    serde_json::Value::Number(n) => {
                        lines.push(format!("{}: {}", key_upper, n));
                    }
                    serde_json::Value::Bool(b) => {
                        lines.push(format!("{}: {}", key_upper, b));
                    }
                    serde_json::Value::Null => {}
                    serde_json::Value::Array(_) => {} // empty array, skip
                }
            }
            lines.join("\n")
        }
        serde_json::Value::Array(arr) => {
            arr.iter()
                .map(|v| format_llm_value(v, indent))
                .collect::<Vec<_>>()
                .join("\n")
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => String::new(),
    }
}

/// Simple Levenshtein distance for suggestions
fn levenshtein(a: &str, b: &str) -> usize {
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
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

// =============================================================================
// v0.50.24 - Proof Query (Task 47.7-47.8)
// =============================================================================

use crate::index::{ProofEntry, ProofIndex, ProofStatus};

/// Proof query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofResult {
    pub query: String,
    pub z3_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z3_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub proofs: Vec<ProofEntry>,
    pub summary: ProofSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<QueryError>,
}

/// Summary of proof verification status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSummary {
    pub total: usize,
    pub verified: usize,
    pub failed: usize,
    pub timeout: usize,
    pub unknown: usize,
    pub pending: usize,
}

impl ProofSummary {
    pub fn from_proofs(proofs: &[ProofEntry]) -> Self {
        let mut verified = 0;
        let mut failed = 0;
        let mut timeout = 0;
        let mut unknown = 0;
        let mut pending = 0;

        for proof in proofs {
            // Count both pre and post status
            for status in [&proof.pre_status, &proof.post_status].into_iter().flatten() {
                match status {
                    ProofStatus::Verified => verified += 1,
                    ProofStatus::Failed => failed += 1,
                    ProofStatus::Timeout => timeout += 1,
                    ProofStatus::Unknown => unknown += 1,
                    ProofStatus::Pending | ProofStatus::Unavailable => pending += 1,
                }
            }
        }

        Self {
            total: proofs.len(),
            verified,
            failed,
            timeout,
            unknown,
            pending,
        }
    }
}

/// Query proof verification results
pub fn query_proofs(
    proof_index: &ProofIndex,
    name: Option<&str>,
    unverified: bool,
    failed: bool,
    timeout: bool,
) -> ProofResult {
    let mut proofs: Vec<ProofEntry> = proof_index.proofs.clone();

    // Filter by name if specified
    if let Some(name) = name {
        proofs.retain(|p| p.name.contains(name));
    }

    // Filter by status
    if unverified {
        proofs.retain(|p| {
            let pre_unverified = p.pre_status.is_none_or(|s| s != ProofStatus::Verified);
            let post_unverified = p.post_status.is_none_or(|s| s != ProofStatus::Verified);
            pre_unverified || post_unverified
        });
    }

    if failed {
        proofs.retain(|p| {
            p.pre_status == Some(ProofStatus::Failed) || p.post_status == Some(ProofStatus::Failed)
        });
    }

    if timeout {
        proofs.retain(|p| {
            p.pre_status == Some(ProofStatus::Timeout) || p.post_status == Some(ProofStatus::Timeout)
        });
    }

    let query = if let Some(name) = name {
        format!("proof:{}", name)
    } else if unverified {
        "proof:--unverified".to_string()
    } else if failed {
        "proof:--failed".to_string()
    } else if timeout {
        "proof:--timeout".to_string()
    } else {
        "proof:all".to_string()
    };

    let summary = ProofSummary::from_proofs(&proofs);

    ProofResult {
        query,
        z3_available: proof_index.z3_available,
        z3_version: proof_index.z3_version.clone(),
        proofs,
        summary,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein("hello", "hello"), 0);
        assert_eq!(levenshtein("hello", "helo"), 1);
        assert_eq!(levenshtein("kitten", "sitting"), 3);
    }
}
