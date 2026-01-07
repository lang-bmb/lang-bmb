//! BMB AI Query System - v0.25.0
//!
//! Query interface for AI tools to understand BMB projects.
//! RFC-0001: AI-Native Code Query System

use crate::index::{FunctionEntry, ProjectIndex, SymbolEntry, SymbolKind, TypeEntry};
use serde::{Deserialize, Serialize};

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
