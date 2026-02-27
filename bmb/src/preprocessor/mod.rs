//! BMB Preprocessor - handles @include directives and prelude
//!
//! Expands @include "path" directives by inlining file contents before parsing.
//! Supports:
//! - Relative paths (from current file)
//! - Absolute paths
//! - Include search paths (-I flag)
//! - Circular include detection
//! - Prelude auto-include (v0.60.252)

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Preprocessor error types
#[derive(Debug)]
pub enum PreprocessorError {
    /// File not found
    FileNotFound(String, Vec<PathBuf>),
    /// Circular include detected
    CircularInclude(PathBuf),
    /// IO error
    IoError(std::io::Error),
    /// Invalid include syntax
    InvalidSyntax(String),
}

impl std::fmt::Display for PreprocessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PreprocessorError::FileNotFound(path, searched) => {
                write!(f, "Include file not found: '{}'\nSearched in:", path)?;
                for p in searched {
                    write!(f, "\n  - {}", p.display())?;
                }
                Ok(())
            }
            PreprocessorError::CircularInclude(path) => {
                write!(f, "Circular include detected: {}", path.display())
            }
            PreprocessorError::IoError(e) => write!(f, "IO error: {}", e),
            PreprocessorError::InvalidSyntax(msg) => write!(f, "Invalid @include syntax: {}", msg),
        }
    }
}

impl std::error::Error for PreprocessorError {}

impl From<std::io::Error> for PreprocessorError {
    fn from(e: std::io::Error) -> Self {
        PreprocessorError::IoError(e)
    }
}

/// Preprocessor state
pub struct Preprocessor {
    /// Include search paths
    include_paths: Vec<PathBuf>,
    /// Already included files (for circular detection)
    included: HashSet<PathBuf>,
}

impl Preprocessor {
    /// Create a new preprocessor with given include paths
    pub fn new(include_paths: Vec<PathBuf>) -> Self {
        Self {
            include_paths,
            included: HashSet::new(),
        }
    }

    /// Expand all @include directives in source
    pub fn expand(&mut self, source: &str, source_path: &Path) -> Result<String, PreprocessorError> {
        let canonical = source_path.canonicalize().unwrap_or_else(|_| source_path.to_path_buf());

        // Check for circular include
        if self.included.contains(&canonical) {
            return Err(PreprocessorError::CircularInclude(canonical));
        }
        self.included.insert(canonical.clone());

        let source_dir = source_path.parent().unwrap_or(Path::new("."));
        let mut result = String::with_capacity(source.len());
        let lines = source.lines().peekable();
        let mut line_num = 0;

        for line in lines {
            line_num += 1;
            let trimmed = line.trim();

            // Check for @include directive
            if trimmed.starts_with("@include") {
                let include_path = self.parse_include_directive(trimmed)?;
                let resolved = self.resolve_include_path(&include_path, source_dir)?;

                // Read and recursively expand the included file
                let included_source = std::fs::read_to_string(&resolved)?;
                let expanded = self.expand(&included_source, &resolved)?;

                // Add a comment marking the include for debugging
                result.push_str(&format!("// @include \"{}\" (from {}:{})\n",
                    include_path, source_path.display(), line_num));
                result.push_str(&expanded);
                result.push_str(&format!("// end @include \"{}\"\n", include_path));
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// Parse @include "path" directive and extract the path
    fn parse_include_directive(&self, line: &str) -> Result<String, PreprocessorError> {
        let rest = line.strip_prefix("@include").unwrap().trim();

        // Support both @include "path" and @include("path")
        let path = if let Some(stripped) = rest.strip_prefix('"') {
            // @include "path"
            let end = stripped.find('"').ok_or_else(|| {
                PreprocessorError::InvalidSyntax("Missing closing quote".to_string())
            })?;
            &stripped[..end]
        } else if rest.starts_with('(') {
            // @include("path")
            let inner = rest.strip_prefix('(').and_then(|s| s.strip_suffix(')'))
                .ok_or_else(|| PreprocessorError::InvalidSyntax("Missing closing parenthesis".to_string()))?;
            let inner = inner.trim();
            if inner.starts_with('"') && inner.ends_with('"') {
                &inner[1..inner.len()-1]
            } else {
                return Err(PreprocessorError::InvalidSyntax("Path must be quoted".to_string()));
            }
        } else {
            return Err(PreprocessorError::InvalidSyntax(
                "Expected @include \"path\" or @include(\"path\")".to_string()
            ));
        };

        Ok(path.to_string())
    }

    /// Resolve include path, searching in order:
    /// 1. Relative to current file
    /// 2. Include search paths
    fn resolve_include_path(&self, include_path: &str, source_dir: &Path) -> Result<PathBuf, PreprocessorError> {
        let mut searched = Vec::new();

        // Try relative to source file first
        let relative = source_dir.join(include_path);
        searched.push(relative.clone());
        if relative.exists() {
            return Ok(relative);
        }

        // Try include search paths
        for search_path in &self.include_paths {
            let candidate = search_path.join(include_path);
            searched.push(candidate.clone());
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(PreprocessorError::FileNotFound(include_path.to_string(), searched))
    }
}

/// Convenience function to expand includes in a source file
pub fn expand_includes(
    source: &str,
    source_path: &Path,
    include_paths: &[PathBuf],
) -> Result<String, PreprocessorError> {
    let mut preprocessor = Preprocessor::new(include_paths.to_vec());
    preprocessor.expand(source, source_path)
}

/// Expand includes with automatic prelude prepending (v0.60.252)
///
/// If `prelude_path` is Some, automatically prepends @include directive
/// for the prelude file before processing user source.
/// The prelude_path should be the direct path to the prelude.bmb file.
pub fn expand_with_prelude(
    source: &str,
    source_path: &Path,
    include_paths: &[PathBuf],
    prelude_path: Option<&Path>,
) -> Result<String, PreprocessorError> {
    let mut preprocessor = Preprocessor::new(include_paths.to_vec());

    // If prelude is enabled, prepend prelude include
    let source_with_prelude = if let Some(prelude_file) = prelude_path {
        if prelude_file.exists() {
            // Add prelude include directive
            let prelude_include = format!(
                "@include \"{}\"\n// === End of prelude ===\n\n",
                prelude_file.display().to_string().replace('\\', "/")
            );
            format!("{}{}", prelude_include, source)
        } else {
            source.to_string()
        }
    } else {
        source.to_string()
    };

    preprocessor.expand(&source_with_prelude, source_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_include_directive() {
        let pp = Preprocessor::new(vec![]);

        // Test quoted format
        assert_eq!(
            pp.parse_include_directive("@include \"path/to/file.bmb\"").unwrap(),
            "path/to/file.bmb"
        );

        // Test parenthesis format
        assert_eq!(
            pp.parse_include_directive("@include(\"path/to/file.bmb\")").unwrap(),
            "path/to/file.bmb"
        );
    }

    #[test]
    fn test_no_includes() {
        let source = r#"
fn main() -> i64 = 42;
"#;
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("fn main()"));
    }

    // --- Cycle 64: Additional preprocessor tests ---

    #[test]
    fn test_parse_include_directive_with_spaces() {
        let pp = Preprocessor::new(vec![]);
        // Extra spaces should be handled
        assert_eq!(
            pp.parse_include_directive("@include  \"file.bmb\"").unwrap(),
            "file.bmb"
        );
    }

    #[test]
    fn test_parse_include_directive_invalid_syntax() {
        let pp = Preprocessor::new(vec![]);
        // Missing quotes
        let result = pp.parse_include_directive("@include file.bmb");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_include_directive_unclosed_quote() {
        let pp = Preprocessor::new(vec![]);
        let result = pp.parse_include_directive("@include \"file.bmb");
        assert!(result.is_err());
    }

    #[test]
    fn test_preprocessor_error_display_file_not_found() {
        let err = PreprocessorError::FileNotFound(
            "missing.bmb".to_string(),
            vec![PathBuf::from("/usr/include"), PathBuf::from("./lib")],
        );
        let msg = format!("{}", err);
        assert!(msg.contains("missing.bmb"));
        assert!(msg.contains("Searched in:"));
    }

    #[test]
    fn test_preprocessor_error_display_circular() {
        let err = PreprocessorError::CircularInclude(PathBuf::from("a.bmb"));
        let msg = format!("{}", err);
        assert!(msg.contains("Circular include"));
        assert!(msg.contains("a.bmb"));
    }

    #[test]
    fn test_preprocessor_error_display_invalid_syntax() {
        let err = PreprocessorError::InvalidSyntax("bad directive".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid @include syntax"));
        assert!(msg.contains("bad directive"));
    }

    #[test]
    fn test_multiple_non_include_lines() {
        let source = "fn a() -> i64 = 1;\nfn b() -> i64 = 2;\nfn c() -> i64 = 3;";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("fn a()"));
        assert!(result.contains("fn b()"));
        assert!(result.contains("fn c()"));
    }

    #[test]
    fn test_include_file_not_found() {
        let source = "@include \"nonexistent_file_xyz.bmb\"";
        let result = expand_includes(source, Path::new("test.bmb"), &[]);
        assert!(result.is_err());
    }

    // ====================================================================
    // Additional preprocessor tests (Cycle 429)
    // ====================================================================

    #[test]
    fn test_expand_with_prelude_none() {
        let source = "fn main() -> i64 = 42;";
        let result = expand_with_prelude(source, Path::new("test.bmb"), &[], None).unwrap();
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_expand_with_prelude_nonexistent_file() {
        let source = "fn main() -> i64 = 42;";
        // If prelude file doesn't exist, source should pass through unchanged
        let result = expand_with_prelude(
            source,
            Path::new("test.bmb"),
            &[],
            Some(Path::new("/nonexistent/prelude.bmb")),
        ).unwrap();
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_expand_preserves_non_include_lines() {
        let source = "fn a() -> i64 = 1;\n// comment\nfn b() -> i64 = 2;";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("fn a()"));
        assert!(result.contains("// comment"));
        assert!(result.contains("fn b()"));
    }

    #[test]
    fn test_parse_include_directive_invalid_no_quote() {
        let pp = Preprocessor::new(vec![]);
        let result = pp.parse_include_directive("@include ");
        assert!(result.is_err());
    }

    // --- Cycle 1226: Additional Preprocessor Tests ---

    #[test]
    fn test_preprocessor_new_with_include_paths() {
        let pp = Preprocessor::new(vec![
            PathBuf::from("/usr/include"),
            PathBuf::from("./lib"),
        ]);
        assert_eq!(pp.include_paths.len(), 2);
        assert!(pp.included.is_empty());
    }

    #[test]
    fn test_parse_include_directive_paren_with_spaces() {
        let pp = Preprocessor::new(vec![]);
        assert_eq!(
            pp.parse_include_directive("@include( \"file.bmb\" )").unwrap(),
            "file.bmb"
        );
    }

    #[test]
    fn test_parse_include_directive_paren_no_quotes() {
        let pp = Preprocessor::new(vec![]);
        let result = pp.parse_include_directive("@include(file.bmb)");
        assert!(result.is_err());
    }

    #[test]
    fn test_expand_empty_source() {
        let result = expand_includes("", Path::new("test.bmb"), &[]).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_expand_single_line_no_include() {
        let result = expand_includes("fn main() -> i64 = 42;", Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_preprocessor_error_display_io() {
        let err = PreprocessorError::IoError(
            std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")
        );
        let msg = format!("{}", err);
        assert!(msg.contains("IO error"));
    }

    #[test]
    fn test_preprocessor_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let pp_err: PreprocessorError = io_err.into();
        assert!(matches!(pp_err, PreprocessorError::IoError(_)));
    }

    #[test]
    fn test_expand_with_prelude_no_path() {
        let source = "fn add(a: i64, b: i64) -> i64 = a + b;";
        let result = expand_with_prelude(source, Path::new("test.bmb"), &[], None).unwrap();
        assert!(result.contains("fn add"));
    }

    #[test]
    fn test_resolve_include_path_not_found() {
        let pp = Preprocessor::new(vec![PathBuf::from("/nonexistent_dir_xyz")]);
        let result = pp.resolve_include_path("missing.bmb", Path::new("/tmp"));
        assert!(result.is_err());
        if let Err(PreprocessorError::FileNotFound(name, searched)) = result {
            assert_eq!(name, "missing.bmb");
            assert_eq!(searched.len(), 2); // relative + 1 include path
        }
    }

    #[test]
    fn test_expand_preserves_line_count() {
        let source = "line1\nline2\nline3\n";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        let line_count = result.lines().count();
        assert_eq!(line_count, 3);
    }

    // ================================================================
    // Additional preprocessor tests (Cycle 1236)
    // ================================================================

    #[test]
    fn test_preprocessor_error_is_std_error() {
        let err = PreprocessorError::InvalidSyntax("test".to_string());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_preprocessor_error_debug_format() {
        let err = PreprocessorError::CircularInclude(PathBuf::from("loop.bmb"));
        let debug = format!("{:?}", err);
        assert!(debug.contains("CircularInclude"));
    }

    #[test]
    fn test_preprocessor_new_empty() {
        let pp = Preprocessor::new(vec![]);
        assert!(pp.include_paths.is_empty());
        assert!(pp.included.is_empty());
    }

    #[test]
    fn test_parse_include_paren_unclosed() {
        let pp = Preprocessor::new(vec![]);
        let result = pp.parse_include_directive("@include(\"file.bmb\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_include_path_empty_search_paths() {
        let pp = Preprocessor::new(vec![]);
        let result = pp.resolve_include_path("missing.bmb", Path::new("/nonexistent"));
        assert!(result.is_err());
        if let Err(PreprocessorError::FileNotFound(_, searched)) = result {
            assert_eq!(searched.len(), 1); // only relative path
        }
    }

    #[test]
    fn test_expand_whitespace_only() {
        let result = expand_includes("   \n  \n   ", Path::new("test.bmb"), &[]).unwrap();
        assert!(!result.contains("@include"));
    }

    #[test]
    fn test_preprocessor_error_file_not_found_empty_searched() {
        let err = PreprocessorError::FileNotFound("x.bmb".to_string(), vec![]);
        let msg = format!("{}", err);
        assert!(msg.contains("x.bmb"));
        assert!(msg.contains("Searched in:"));
    }

    #[test]
    fn test_expand_includes_preserves_comments() {
        let source = "// this is a comment\nfn main() -> i64 = 0;";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("// this is a comment"));
    }

    #[test]
    fn test_preprocessor_error_io_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "custom error");
        let pp_err = PreprocessorError::IoError(io_err);
        let msg = format!("{}", pp_err);
        assert!(msg.contains("IO error"));
        assert!(msg.contains("custom error"));
    }

    #[test]
    fn test_expand_with_prelude_both_args() {
        // When prelude path is provided but doesn't exist, source should pass through
        let source = "fn test() -> i64 = 1;";
        let result = expand_with_prelude(
            source,
            Path::new("test.bmb"),
            &[PathBuf::from("/some/path")],
            Some(Path::new("/nonexistent/prelude.bmb")),
        ).unwrap();
        assert!(result.contains("fn test()"));
    }

    // ================================================================
    // Additional preprocessor tests (Cycle 1242)
    // ================================================================

    #[test]
    fn test_preprocessor_error_circular_display_path() {
        let err = PreprocessorError::CircularInclude(PathBuf::from("/a/b/c.bmb"));
        let msg = format!("{}", err);
        assert!(msg.contains("Circular include"));
        assert!(msg.contains("c.bmb"));
    }

    #[test]
    fn test_parse_include_directive_tab_before_path() {
        let pp = Preprocessor::new(vec![]);
        // Tab character before quote
        assert_eq!(
            pp.parse_include_directive("@include\t\"file.bmb\"").unwrap(),
            "file.bmb"
        );
    }

    #[test]
    fn test_expand_multiline_preserves_all_lines() {
        let source = "fn a() -> i64 = 1;\nfn b() -> i64 = 2;\nfn c() -> i64 = 3;\nfn d() -> i64 = 4;";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("fn a()"));
        assert!(result.contains("fn b()"));
        assert!(result.contains("fn c()"));
        assert!(result.contains("fn d()"));
    }

    #[test]
    fn test_preprocessor_new_single_include_path() {
        let pp = Preprocessor::new(vec![PathBuf::from("/lib")]);
        assert_eq!(pp.include_paths.len(), 1);
        assert_eq!(pp.include_paths[0], PathBuf::from("/lib"));
    }

    #[test]
    fn test_resolve_include_multiple_search_paths_not_found() {
        let pp = Preprocessor::new(vec![
            PathBuf::from("/nonexistent_a"),
            PathBuf::from("/nonexistent_b"),
            PathBuf::from("/nonexistent_c"),
        ]);
        let result = pp.resolve_include_path("missing.bmb", Path::new("/nonexistent_dir"));
        assert!(result.is_err());
        if let Err(PreprocessorError::FileNotFound(_, searched)) = result {
            assert_eq!(searched.len(), 4); // relative + 3 search paths
        }
    }

    #[test]
    fn test_parse_include_directive_empty_path() {
        let pp = Preprocessor::new(vec![]);
        // Empty path inside quotes is syntactically valid
        let result = pp.parse_include_directive("@include \"\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_expand_includes_convenience_no_include_paths() {
        let source = "// just a comment";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("just a comment"));
    }

    #[test]
    fn test_preprocessor_error_file_not_found_single() {
        let err = PreprocessorError::FileNotFound(
            "one.bmb".to_string(),
            vec![PathBuf::from("./src")],
        );
        let msg = format!("{}", err);
        assert!(msg.contains("one.bmb"));
        assert!(msg.contains("./src"));
    }

    #[test]
    fn test_expand_source_with_inline_comment_preserved() {
        let source = "fn add(a: i64, b: i64) -> i64 = a + b; // inline comment";
        let result = expand_includes(source, Path::new("test.bmb"), &[]).unwrap();
        assert!(result.contains("// inline comment"));
    }

    #[test]
    fn test_expand_with_prelude_none_with_include_paths() {
        let source = "fn main() -> i64 = 0;";
        let result = expand_with_prelude(
            source,
            Path::new("test.bmb"),
            &[PathBuf::from("/lib"), PathBuf::from("/stdlib")],
            None,
        ).unwrap();
        assert!(result.contains("fn main()"));
    }
}
