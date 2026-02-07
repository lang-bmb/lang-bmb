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
}
