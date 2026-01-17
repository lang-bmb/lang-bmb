//! v0.46: Pattern exhaustiveness checking
//!
//! Implements the "usefulness" algorithm for pattern matching:
//! - Checks if all possible values are covered by match arms
//! - Detects unreachable patterns (dead code)
//! - Reports missing patterns for non-exhaustive matches
//!
//! P0 Correctness: All checks happen at compile-time
//! P0 Performance: No runtime overhead
//!
//! Algorithm based on Rust's exhaustiveness checker:
//! <https://rustc-dev-guide.rust-lang.org/pat-exhaustive-checking.html>

use crate::ast::{LiteralPattern, Pattern, Spanned, Type};
use std::collections::{HashMap, HashSet};

/// Represents a constructor in pattern matching
/// Constructors are the "head" of a pattern that determines what it matches
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constructor {
    /// Wildcard/variable - matches anything
    Wildcard,
    /// Integer literal
    IntLit(i64),
    /// Float literal (using bits for Eq/Hash)
    FloatLit(u64),
    /// Boolean literal
    BoolLit(bool),
    /// String literal
    StringLit(String),
    /// Enum variant with name
    EnumVariant { enum_name: String, variant: String },
    /// Struct constructor
    Struct(String),
    /// Integer range (inclusive)
    IntRange { start: i64, end: i64 },
    /// Tuple with arity
    Tuple(usize),
    /// Array with size
    Array(usize),
    /// Array with rest pattern (minimum size)
    ArrayRest { min_size: usize },
}

/// Result of exhaustiveness check
#[derive(Debug)]
pub struct ExhaustivenessResult {
    /// Whether the match is exhaustive
    pub is_exhaustive: bool,
    /// Indices of unreachable arms (for warnings)
    pub unreachable_arms: Vec<usize>,
    /// Missing patterns (if not exhaustive)
    pub missing_patterns: Vec<String>,
    /// v0.51: Whether guards are present without an unconditional fallback
    /// This indicates the match may not be exhaustive at runtime
    pub has_guards_without_fallback: bool,
}

/// Deconstructed pattern for analysis
#[derive(Debug, Clone)]
struct DeconstructedPattern {
    constructor: Constructor,
    /// Sub-patterns for fields (enum fields, tuple elements, etc.)
    fields: Vec<DeconstructedPattern>,
}

impl DeconstructedPattern {
    fn wildcard() -> Self {
        DeconstructedPattern {
            constructor: Constructor::Wildcard,
            fields: vec![],
        }
    }

    fn from_pattern(pattern: &Pattern, ty: &Type, ctx: &ExhaustivenessContext) -> Self {
        match pattern {
            Pattern::Wildcard | Pattern::Var(_) => DeconstructedPattern::wildcard(),

            Pattern::Literal(lit) => {
                let ctor = match lit {
                    LiteralPattern::Int(n) => Constructor::IntLit(*n),
                    LiteralPattern::Float(f) => Constructor::FloatLit(f.to_bits()),
                    LiteralPattern::Bool(b) => Constructor::BoolLit(*b),
                    LiteralPattern::String(s) => Constructor::StringLit(s.clone()),
                };
                DeconstructedPattern {
                    constructor: ctor,
                    fields: vec![],
                }
            }

            Pattern::EnumVariant {
                enum_name,
                variant,
                bindings,
            } => {
                // v0.58: Get field types, handling generic type substitution
                let field_types: Vec<Type> = if let Type::Generic { name, type_args } = ty {
                    // We have a generic type - need to substitute
                    if name == enum_name {
                        let type_params = ctx
                            .generic_enum_params
                            .get(name)
                            .cloned()
                            .unwrap_or_default();
                        let mut subst: HashMap<String, Type> = HashMap::new();
                        for (param, arg) in type_params.iter().zip(type_args.iter()) {
                            subst.insert(param.clone(), arg.as_ref().clone());
                        }

                        // Get variant fields and substitute
                        let raw_types = ctx.get_enum_variant_fields(enum_name, variant);
                        raw_types
                            .iter()
                            .map(|t| substitute_type(t, &subst))
                            .collect()
                    } else {
                        ctx.get_enum_variant_fields(enum_name, variant)
                    }
                } else {
                    ctx.get_enum_variant_fields(enum_name, variant)
                };

                let fields: Vec<_> = bindings
                    .iter()
                    .zip(field_types.iter())
                    .map(|(p, t)| DeconstructedPattern::from_pattern(&p.node, t, ctx))
                    .collect();

                DeconstructedPattern {
                    constructor: Constructor::EnumVariant {
                        enum_name: enum_name.clone(),
                        variant: variant.clone(),
                    },
                    fields,
                }
            }

            Pattern::Struct { name, fields } => {
                // v0.54: Look up actual field types from context
                let field_pats: Vec<_> = fields
                    .iter()
                    .map(|(field_name, p)| {
                        let field_ty = ctx
                            .get_struct_field_type(name, &field_name.node)
                            .unwrap_or(Type::I64); // Fallback for unknown fields
                        DeconstructedPattern::from_pattern(&p.node, &field_ty, ctx)
                    })
                    .collect();

                DeconstructedPattern {
                    constructor: Constructor::Struct(name.clone()),
                    fields: field_pats,
                }
            }

            Pattern::Range {
                start,
                end,
                inclusive,
            } => {
                let (s, e) = match (start, end) {
                    (LiteralPattern::Int(s), LiteralPattern::Int(e)) => {
                        (*s, if *inclusive { *e } else { *e - 1 })
                    }
                    _ => (i64::MIN, i64::MAX), // Non-int ranges match everything
                };
                DeconstructedPattern {
                    constructor: Constructor::IntRange { start: s, end: e },
                    fields: vec![],
                }
            }

            Pattern::Or(alts) => {
                // For or-patterns, we expand them during analysis
                // For now, treat as wildcard if any alt is wildcard
                for alt in alts {
                    if matches!(alt.node, Pattern::Wildcard | Pattern::Var(_)) {
                        return DeconstructedPattern::wildcard();
                    }
                }
                // Otherwise use first alternative (simplified)
                DeconstructedPattern::from_pattern(&alts[0].node, ty, ctx)
            }

            Pattern::Binding { pattern, .. } => {
                // Binding doesn't affect coverage, just the inner pattern
                DeconstructedPattern::from_pattern(&pattern.node, ty, ctx)
            }

            Pattern::Tuple(elems) => {
                let elem_types: Vec<Type> = if let Type::Tuple(types) = ty {
                    types.iter().map(|t| (**t).clone()).collect()
                } else {
                    vec![Type::I64; elems.len()]
                };
                let fields: Vec<_> = elems
                    .iter()
                    .zip(elem_types.iter())
                    .map(|(p, t)| DeconstructedPattern::from_pattern(&p.node, t, ctx))
                    .collect();

                DeconstructedPattern {
                    constructor: Constructor::Tuple(elems.len()),
                    fields,
                }
            }

            Pattern::Array(elems) => {
                let elem_ty = if let Type::Array(t, _) = ty {
                    (**t).clone()
                } else {
                    Type::I64
                };
                let fields: Vec<_> = elems
                    .iter()
                    .map(|p| DeconstructedPattern::from_pattern(&p.node, &elem_ty, ctx))
                    .collect();

                DeconstructedPattern {
                    constructor: Constructor::Array(elems.len()),
                    fields,
                }
            }

            Pattern::ArrayRest { prefix, suffix } => {
                let elem_ty = if let Type::Array(t, _) = ty {
                    (**t).clone()
                } else {
                    Type::I64
                };
                let min_size = prefix.len() + suffix.len();
                let mut fields: Vec<_> = prefix
                    .iter()
                    .map(|p| DeconstructedPattern::from_pattern(&p.node, &elem_ty, ctx))
                    .collect();
                fields.extend(
                    suffix
                        .iter()
                        .map(|p| DeconstructedPattern::from_pattern(&p.node, &elem_ty, ctx)),
                );

                DeconstructedPattern {
                    constructor: Constructor::ArrayRest { min_size },
                    fields,
                }
            }
        }
    }

    /// Check if this pattern is a wildcard
    fn is_wildcard(&self) -> bool {
        matches!(self.constructor, Constructor::Wildcard)
    }
}

/// Context for exhaustiveness checking
pub struct ExhaustivenessContext {
    /// Enum definitions: enum_name -> list of (variant_name, field_types)
    pub enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    /// v0.54: Struct definitions: struct_name -> list of (field_name, field_type)
    pub structs: HashMap<String, Vec<(String, Type)>>,
    /// v0.58: Generic enum type params: enum_name -> type_param_names
    pub generic_enum_params: HashMap<String, Vec<String>>,
}

impl ExhaustivenessContext {
    pub fn new() -> Self {
        ExhaustivenessContext {
            enums: HashMap::new(),
            structs: HashMap::new(),
            generic_enum_params: HashMap::new(),
        }
    }

    /// Register an enum definition
    pub fn add_enum(&mut self, name: &str, variants: Vec<(String, Vec<Type>)>) {
        self.enums.insert(name.to_string(), variants);
    }

    /// v0.58: Register generic enum type parameters
    pub fn add_generic_enum_params(&mut self, name: &str, params: Vec<String>) {
        self.generic_enum_params.insert(name.to_string(), params);
    }

    /// v0.54: Register a struct definition
    pub fn add_struct(&mut self, name: &str, fields: Vec<(String, Type)>) {
        self.structs.insert(name.to_string(), fields);
    }

    /// v0.54: Get the type of a struct field by name
    pub fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<Type> {
        self.structs
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _)| n == field_name))
            .map(|(_, ty)| ty.clone())
    }

    /// Get all variants of an enum
    #[allow(dead_code)]
    fn get_enum_variants(&self, enum_name: &str) -> Vec<String> {
        self.enums
            .get(enum_name)
            .map(|vs| vs.iter().map(|(n, _)| n.clone()).collect())
            .unwrap_or_default()
    }

    /// Get field types for an enum variant
    fn get_enum_variant_fields(&self, enum_name: &str, variant: &str) -> Vec<Type> {
        self.enums
            .get(enum_name)
            .and_then(|vs| vs.iter().find(|(n, _)| n == variant))
            .map(|(_, fields)| fields.clone())
            .unwrap_or_default()
    }
}

impl Default for ExhaustivenessContext {
    fn default() -> Self {
        Self::new()
    }
}

/// v0.58: Substitute type variables in a type
/// Given a type like `Option<T>` and a substitution `{T -> bool}`, returns `Option<bool>`
fn substitute_type(ty: &Type, subst: &HashMap<String, Type>) -> Type {
    match ty {
        Type::TypeVar(name) => {
            subst.get(name).cloned().unwrap_or_else(|| ty.clone())
        }
        // v0.58: Also handle Named types that match type parameters
        // (enum variants may store type params as Named instead of TypeVar)
        Type::Named(name) => {
            subst.get(name).cloned().unwrap_or_else(|| ty.clone())
        }
        Type::Generic { name, type_args } => {
            let new_args: Vec<Box<Type>> = type_args
                .iter()
                .map(|arg| Box::new(substitute_type(arg, subst)))
                .collect();
            Type::Generic {
                name: name.clone(),
                type_args: new_args,
            }
        }
        Type::Tuple(elems) => {
            Type::Tuple(
                elems
                    .iter()
                    .map(|e| Box::new(substitute_type(e, subst)))
                    .collect(),
            )
        }
        Type::Array(elem, size) => {
            Type::Array(Box::new(substitute_type(elem, subst)), *size)
        }
        Type::Ref(inner) => Type::Ref(Box::new(substitute_type(inner, subst))),
        Type::RefMut(inner) => Type::RefMut(Box::new(substitute_type(inner, subst))),
        _ => ty.clone(),
    }
}

/// v0.57: Expand Or-patterns into multiple individual patterns
/// e.g., `true | false` becomes [`true`, `false`]
fn expand_or_pattern(pattern: &Pattern) -> Vec<&Pattern> {
    match pattern {
        Pattern::Or(alts) => {
            // Recursively expand nested Or-patterns
            alts.iter().flat_map(|p| expand_or_pattern(&p.node)).collect()
        }
        _ => vec![pattern],
    }
}

/// Check if a match expression is exhaustive
pub fn check_exhaustiveness(
    match_type: &Type,
    arms: &[(Spanned<Pattern>, Option<Spanned<crate::ast::Expr>>)],
    ctx: &ExhaustivenessContext,
) -> ExhaustivenessResult {
    // Convert patterns to deconstructed form
    let mut matrix: Vec<DeconstructedPattern> = vec![];
    let mut unreachable_arms = vec![];

    // v0.51: Track guards and unconditional fallbacks
    let mut has_any_guard = false;
    let mut has_unconditional_fallback = false;

    for (i, (pattern, guard)) in arms.iter().enumerate() {
        // v0.57: Expand Or-patterns into multiple individual patterns
        let expanded_patterns = expand_or_pattern(&pattern.node);

        // v0.51: Track if this arm has a guard
        if guard.is_some() {
            has_any_guard = true;
        }

        // v0.51: Check for unconditional fallback (wildcard/variable WITHOUT guard)
        if guard.is_none() && is_unconditional_pattern(&pattern.node) {
            has_unconditional_fallback = true;
        }

        // Process all expanded patterns
        let mut any_useful = false;
        for expanded_pat in &expanded_patterns {
            let decon = DeconstructedPattern::from_pattern(expanded_pat, match_type, ctx);

            // Check if this pattern is useful (adds new coverage)
            if is_useful(&matrix, &decon, match_type, ctx) {
                any_useful = true;
            }

            matrix.push(decon);
        }

        // Only mark as unreachable if NONE of the expanded patterns are useful
        if !any_useful {
            unreachable_arms.push(i);
        }
    }

    // Check for missing patterns
    let missing = find_missing_patterns(&matrix, match_type, ctx);

    ExhaustivenessResult {
        is_exhaustive: missing.is_empty(),
        unreachable_arms,
        missing_patterns: missing,
        // v0.51: Warn if guards are present but no unconditional fallback
        has_guards_without_fallback: has_any_guard && !has_unconditional_fallback,
    }
}

/// v0.51: Check if a pattern is unconditional (will always match its type)
/// Wildcards and variables are unconditional
/// v0.57: Or-patterns are unconditional if any alternative is unconditional
fn is_unconditional_pattern(pattern: &Pattern) -> bool {
    match pattern {
        Pattern::Wildcard | Pattern::Var(_) => true,
        Pattern::Or(alts) => alts.iter().any(|p| is_unconditional_pattern(&p.node)),
        Pattern::Binding { pattern, .. } => is_unconditional_pattern(&pattern.node),
        _ => false,
    }
}

/// Check if a pattern is useful (covers at least one case not covered by matrix)
fn is_useful(
    matrix: &[DeconstructedPattern],
    pattern: &DeconstructedPattern,
    ty: &Type,
    ctx: &ExhaustivenessContext,
) -> bool {
    // Empty matrix - any pattern is useful
    if matrix.is_empty() {
        return true;
    }

    // Wildcard is useful if matrix doesn't already have a wildcard
    if pattern.is_wildcard() {
        return !matrix.iter().any(|p| p.is_wildcard());
    }

    // For non-wildcard patterns, check if covered by existing patterns
    for existing in matrix {
        if patterns_overlap(existing, pattern, ty, ctx) {
            // If existing pattern completely covers new pattern, not useful
            if pattern_covers(existing, pattern) {
                return false;
            }
        }
    }

    true
}

/// Check if two patterns overlap (can match same value)
fn patterns_overlap(
    p1: &DeconstructedPattern,
    p2: &DeconstructedPattern,
    _ty: &Type,
    _ctx: &ExhaustivenessContext,
) -> bool {
    // Wildcard overlaps with everything
    if p1.is_wildcard() || p2.is_wildcard() {
        return true;
    }

    // Same constructor type
    match (&p1.constructor, &p2.constructor) {
        (Constructor::IntLit(a), Constructor::IntLit(b)) => a == b,
        (Constructor::IntLit(n), Constructor::IntRange { start, end })
        | (Constructor::IntRange { start, end }, Constructor::IntLit(n)) => {
            *n >= *start && *n <= *end
        }
        (
            Constructor::IntRange {
                start: s1,
                end: e1,
            },
            Constructor::IntRange {
                start: s2,
                end: e2,
            },
        ) => {
            // Ranges overlap if they intersect
            s1 <= e2 && s2 <= e1
        }
        (Constructor::BoolLit(a), Constructor::BoolLit(b)) => a == b,
        (Constructor::StringLit(a), Constructor::StringLit(b)) => a == b,
        (
            Constructor::EnumVariant {
                enum_name: e1,
                variant: v1,
            },
            Constructor::EnumVariant {
                enum_name: e2,
                variant: v2,
            },
        ) => e1 == e2 && v1 == v2,
        (Constructor::Tuple(a), Constructor::Tuple(b)) => a == b,
        (Constructor::Array(a), Constructor::Array(b)) => a == b,
        (Constructor::Array(size), Constructor::ArrayRest { min_size })
        | (Constructor::ArrayRest { min_size }, Constructor::Array(size)) => *size >= *min_size,
        (
            Constructor::ArrayRest { min_size: _ },
            Constructor::ArrayRest { min_size: _ },
        ) => true, // Rest patterns can always overlap
        _ => false,
    }
}

/// Check if p1 completely covers p2
fn pattern_covers(p1: &DeconstructedPattern, p2: &DeconstructedPattern) -> bool {
    // Wildcard covers everything
    if p1.is_wildcard() {
        return true;
    }

    // Non-wildcard can only cover if same constructor
    if p1.constructor != p2.constructor {
        // Special case: range covering literal
        if let (Constructor::IntRange { start, end }, Constructor::IntLit(n)) =
            (&p1.constructor, &p2.constructor)
        {
            return *n >= *start && *n <= *end;
        }
        return false;
    }

    // Check all fields
    if p1.fields.len() != p2.fields.len() {
        return false;
    }

    for (f1, f2) in p1.fields.iter().zip(p2.fields.iter()) {
        if !pattern_covers(f1, f2) {
            return false;
        }
    }

    true
}

// ============================================================================
// v0.55: Tuple Exhaustiveness Helpers
// ============================================================================

/// Get all possible values for a finite type (bool, enum)
/// Returns None for infinite types (integers, strings, etc.)
fn get_finite_type_values(ty: &Type, ctx: &ExhaustivenessContext) -> Option<Vec<String>> {
    match ty {
        Type::Bool => Some(vec!["true".to_string(), "false".to_string()]),
        Type::Named(name) => {
            // Check if it's an enum
            ctx.enums.get(name).map(|variants| variants
                        .iter()
                        .map(|(v, _)| format!("{}::{}", name, v))
                        .collect())
        }
        // All other types are considered infinite
        _ => None,
    }
}

/// Generate all combinations (cartesian product) of tuple element values
fn generate_tuple_combinations(values: &[Vec<String>]) -> Vec<Vec<String>> {
    if values.is_empty() {
        return vec![vec![]];
    }

    let mut result = vec![vec![]];
    for element_values in values {
        let mut new_result = vec![];
        for existing in &result {
            for value in element_values {
                let mut new_combo = existing.clone();
                new_combo.push(value.clone());
                new_result.push(new_combo);
            }
        }
        result = new_result;
    }
    result
}

/// Extract the concrete values covered by a tuple pattern
/// A wildcard at position i expands to all values from all_values[i]
fn extract_tuple_pattern_values(
    p: &DeconstructedPattern,
    all_values: &[Vec<String>],
) -> Vec<Vec<String>> {
    if p.fields.len() != all_values.len() {
        return vec![];
    }

    // For each position, collect what values the pattern covers
    let mut position_values: Vec<Vec<String>> = vec![];

    for (i, field) in p.fields.iter().enumerate() {
        let values_at_pos = if field.is_wildcard() {
            // Wildcard covers all values at this position
            all_values[i].clone()
        } else {
            // Specific value - extract from constructor
            match &field.constructor {
                Constructor::BoolLit(b) => vec![b.to_string()],
                Constructor::EnumVariant { enum_name, variant, .. } => {
                    vec![format!("{}::{}", enum_name, variant)]
                }
                _ => {
                    // Unknown pattern type - treat as covering nothing
                    vec![]
                }
            }
        };
        position_values.push(values_at_pos);
    }

    // Generate all combinations covered by this pattern
    generate_tuple_combinations(&position_values)
}

/// v0.56: Extract the concrete values covered by a struct pattern
/// Similar to extract_tuple_pattern_values but uses struct field order
fn extract_struct_pattern_values(
    p: &DeconstructedPattern,
    _field_names: &[String], // Kept for API consistency, fields are already ordered
    all_values: &[Vec<String>],
) -> Vec<Vec<String>> {
    if p.fields.len() != all_values.len() {
        return vec![];
    }

    // For each field position, collect what values the pattern covers
    let mut position_values: Vec<Vec<String>> = vec![];

    for (i, field) in p.fields.iter().enumerate() {
        let values_at_pos = if field.is_wildcard() {
            // Wildcard covers all values at this position
            all_values[i].clone()
        } else {
            // Specific value - extract from constructor
            match &field.constructor {
                Constructor::BoolLit(b) => vec![b.to_string()],
                Constructor::EnumVariant { enum_name, variant, .. } => {
                    vec![format!("{}::{}", enum_name, variant)]
                }
                _ => {
                    // Unknown pattern type - treat as covering nothing
                    vec![]
                }
            }
        };
        position_values.push(values_at_pos);
    }

    // Generate all combinations covered by this pattern
    generate_tuple_combinations(&position_values)
}

/// v0.56: Format a missing struct pattern with specific field values
fn format_missing_struct_pattern(
    struct_name: &str,
    field_names: &[String],
    values: &[String],
) -> String {
    let field_assignments: Vec<String> = field_names
        .iter()
        .zip(values.iter())
        .map(|(name, val)| format!("{}: {}", name, val))
        .collect();
    format!("{} {{ {} }}", struct_name, field_assignments.join(", "))
}

// ============================================================================
// v0.47: Integer Range Helpers
// ============================================================================

/// Merge overlapping ranges into non-overlapping sorted ranges
fn merge_ranges(ranges: &[(i64, i64)]) -> Vec<(i64, i64)> {
    if ranges.is_empty() {
        return vec![];
    }

    let mut sorted = ranges.to_vec();
    sorted.sort_by_key(|(s, _)| *s);

    let mut merged: Vec<(i64, i64)> = vec![];

    for (start, end) in sorted {
        if merged.is_empty() {
            merged.push((start, end));
        } else {
            let last = merged.last_mut().unwrap();
            // Check for overlap or adjacency (with overflow protection)
            if start <= last.1 || (last.1 < i64::MAX && start <= last.1 + 1) {
                // Merge
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }
    }

    merged
}

/// Find gaps in range coverage between type_min and type_max
fn find_range_gaps(merged: &[(i64, i64)], (type_min, type_max): (i64, i64)) -> Vec<(i64, i64)> {
    let mut gaps = vec![];
    let mut current_pos = type_min;

    for (start, end) in merged {
        if current_pos < *start {
            // Gap before this range
            gaps.push((current_pos, *start - 1));
        }
        // Move past this range (with overflow protection)
        current_pos = if *end == i64::MAX { i64::MAX } else { *end + 1 };
    }

    // Gap after last range
    if current_pos <= type_max {
        gaps.push((current_pos, type_max));
    }

    gaps
}

/// Find patterns that are missing from the matrix
fn find_missing_patterns(
    matrix: &[DeconstructedPattern],
    ty: &Type,
    ctx: &ExhaustivenessContext,
) -> Vec<String> {
    // If any pattern is a wildcard, the match is exhaustive
    if matrix.iter().any(|p| p.is_wildcard()) {
        return vec![];
    }

    match ty {
        Type::Bool => {
            let mut covered = HashSet::new();
            for p in matrix {
                if let Constructor::BoolLit(b) = &p.constructor {
                    covered.insert(*b);
                }
            }
            let mut missing = vec![];
            if !covered.contains(&true) {
                missing.push("true".to_string());
            }
            if !covered.contains(&false) {
                missing.push("false".to_string());
            }
            missing
        }

        // v0.58: Handle generic types like Option<Option<bool>>
        Type::Generic { name, type_args } => {
            // Check if it's a generic enum
            if let Some(variants) = ctx.enums.get(name).cloned() {
                // Build type substitution map
                let type_params = ctx
                    .generic_enum_params
                    .get(name)
                    .cloned()
                    .unwrap_or_default();
                let mut subst: HashMap<String, Type> = HashMap::new();
                for (param, arg) in type_params.iter().zip(type_args.iter()) {
                    subst.insert(param.clone(), arg.as_ref().clone());
                }

                // Substitute type variables in variant payload types
                let substituted_variants: Vec<(String, Vec<Type>)> = variants
                    .iter()
                    .map(|(vname, payload_types)| {
                        let subst_types: Vec<Type> = payload_types
                            .iter()
                            .map(|t| substitute_type(t, &subst))
                            .collect();
                        (vname.clone(), subst_types)
                    })
                    .collect();

                // Now do exhaustiveness check with substituted types
                let mut variant_patterns: HashMap<String, Vec<&DeconstructedPattern>> =
                    HashMap::new();
                let mut has_wildcard = false;

                for p in matrix {
                    if p.is_wildcard() {
                        has_wildcard = true;
                        break;
                    }
                    if let Constructor::EnumVariant { variant, .. } = &p.constructor {
                        variant_patterns
                            .entry(variant.clone())
                            .or_default()
                            .push(p);
                    }
                }

                if has_wildcard {
                    return vec![];
                }

                let mut missing = vec![];

                for (variant_name, payload_types) in &substituted_variants {
                    if let Some(patterns_for_variant) = variant_patterns.get(variant_name) {
                        if !payload_types.is_empty() {
                            for (field_idx, field_type) in payload_types.iter().enumerate() {
                                let field_patterns: Vec<DeconstructedPattern> =
                                    patterns_for_variant
                                        .iter()
                                        .filter_map(|p| p.fields.get(field_idx).cloned())
                                        .collect();

                                if field_patterns.iter().any(|fp| fp.is_wildcard()) {
                                    continue;
                                }

                                let field_missing =
                                    find_missing_patterns(&field_patterns, field_type, ctx);

                                if !field_missing.is_empty() {
                                    for m in field_missing.iter().take(2) {
                                        missing.push(format!("{}::{}({})", name, variant_name, m));
                                    }
                                    if field_missing.len() > 2 {
                                        missing.push("...".to_string());
                                    }
                                }
                            }
                        }
                    } else if payload_types.is_empty() {
                        missing.push(format!("{}::{}", name, variant_name));
                    } else {
                        missing.push(format!("{}::{}(_)", name, variant_name));
                    }
                }

                missing
            } else {
                // Not an enum, fall through to wildcard requirement
                vec!["_".to_string()]
            }
        }

        Type::Named(name) => {
            // Check if it's an enum
            if let Some(variants) = ctx.enums.get(name).cloned() {
                // v0.58: Improved enum exhaustiveness with payload checking
                // Group patterns by variant and collect their field sub-patterns
                let mut variant_patterns: HashMap<String, Vec<&DeconstructedPattern>> =
                    HashMap::new();
                let mut has_wildcard = false;

                for p in matrix {
                    if p.is_wildcard() {
                        has_wildcard = true;
                        break;
                    }
                    if let Constructor::EnumVariant { variant, .. } = &p.constructor {
                        variant_patterns
                            .entry(variant.clone())
                            .or_default()
                            .push(p);
                    }
                }

                if has_wildcard {
                    return vec![];
                }

                let mut missing = vec![];

                for (variant_name, payload_types) in &variants {
                    if let Some(patterns_for_variant) = variant_patterns.get(variant_name) {
                        // Variant is covered - check if payload is exhaustive
                        if !payload_types.is_empty() {
                            // Has payload - need to check recursively
                            // Collect field patterns for each position
                            for (field_idx, field_type) in payload_types.iter().enumerate() {
                                let field_patterns: Vec<DeconstructedPattern> =
                                    patterns_for_variant
                                        .iter()
                                        .filter_map(|p| p.fields.get(field_idx).cloned())
                                        .collect();

                                // Check if any field pattern is a wildcard
                                if field_patterns.iter().any(|fp| fp.is_wildcard()) {
                                    continue; // This field position is covered
                                }

                                // Recursively check exhaustiveness
                                let field_missing =
                                    find_missing_patterns(&field_patterns, field_type, ctx);

                                if !field_missing.is_empty() {
                                    // Report missing patterns with variant context
                                    for m in field_missing.iter().take(2) {
                                        missing.push(format!("{}::{}({})", name, variant_name, m));
                                    }
                                    if field_missing.len() > 2 {
                                        missing.push("...".to_string());
                                    }
                                }
                            }
                        }
                        // No payload or payload is exhaustive - variant is fully covered
                    } else {
                        // Variant not covered at all
                        if payload_types.is_empty() {
                            missing.push(format!("{}::{}", name, variant_name));
                        } else {
                            missing.push(format!("{}::{}(_)", name, variant_name));
                        }
                    }
                }

                missing
            } else if let Some(fields) = ctx.structs.get(name) {
                // v0.56: Struct exhaustiveness with finite field type support
                // Check if any pattern is a full wildcard first
                for p in matrix {
                    if p.is_wildcard() {
                        return vec![];
                    }
                    if matches!(&p.constructor, Constructor::Struct(s) if s == name)
                        && p.fields.iter().all(|f| f.is_wildcard()) {
                            return vec![];
                        }
                }

                // Get field names and types
                let field_names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
                let field_types: Vec<&Type> = fields.iter().map(|(_, t)| t).collect();

                // Check if all fields have finite types
                let finite_values: Vec<Option<Vec<String>>> = field_types
                    .iter()
                    .map(|t| get_finite_type_values(t, ctx))
                    .collect();

                if finite_values.iter().all(|v| v.is_some()) {
                    // All fields are finite - check full coverage with cartesian product
                    let all_values: Vec<Vec<String>> = finite_values
                        .into_iter()
                        .map(|v| v.unwrap())
                        .collect();

                    // Generate all possible combinations
                    let all_combos = generate_tuple_combinations(&all_values);

                    // Check which combinations are covered
                    let mut covered: HashSet<Vec<String>> = HashSet::new();
                    for p in matrix {
                        if p.is_wildcard() {
                            return vec![]; // Already checked above but safety
                        }
                        if matches!(&p.constructor, Constructor::Struct(s) if s == name) {
                            let pattern_values =
                                extract_struct_pattern_values(p, &field_names, &all_values);
                            for pv in pattern_values {
                                covered.insert(pv);
                            }
                        }
                    }

                    // Find missing combinations
                    let missing: Vec<String> = all_combos
                        .into_iter()
                        .filter(|c| !covered.contains(c))
                        .take(3)
                        .map(|c| format_missing_struct_pattern(name, &field_names, &c))
                        .collect();

                    if missing.is_empty() {
                        vec![]
                    } else if missing.len() < 3 {
                        missing
                    } else {
                        let mut result = missing;
                        result.push("...".to_string());
                        result
                    }
                } else {
                    // At least one field has infinite type - check partial coverage
                    // Similar to tuple mixed handling: finite fields must be exhaustive,
                    // infinite fields must have wildcards in all patterns

                    // Identify finite and infinite field positions
                    let finite_positions: Vec<(usize, Vec<String>)> = finite_values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| v.clone().map(|vals| (i, vals)))
                        .collect();

                    let infinite_positions: Vec<usize> = finite_values
                        .iter()
                        .enumerate()
                        .filter_map(|(i, v)| if v.is_none() { Some(i) } else { None })
                        .collect();

                    // All struct patterns must have wildcards at infinite positions
                    let all_infinite_covered = matrix.iter().all(|p| {
                        if p.is_wildcard() {
                            return true;
                        }
                        if matches!(&p.constructor, Constructor::Struct(s) if s == name) {
                            infinite_positions
                                .iter()
                                .all(|&pos| pos < p.fields.len() && p.fields[pos].is_wildcard())
                        } else {
                            false
                        }
                    });

                    if !all_infinite_covered {
                        return vec![format!("{} {{ .. }}", name)];
                    }

                    // Check finite positions are exhaustive
                    for (pos, expected_values) in &finite_positions {
                        let mut covered_values: HashSet<String> = HashSet::new();
                        for p in matrix {
                            if p.is_wildcard() {
                                covered_values.extend(expected_values.iter().cloned());
                                continue;
                            }
                            if matches!(&p.constructor, Constructor::Struct(s) if s == name)
                                && *pos < p.fields.len() {
                                    if p.fields[*pos].is_wildcard() {
                                        covered_values.extend(expected_values.iter().cloned());
                                    } else {
                                        match &p.fields[*pos].constructor {
                                            Constructor::BoolLit(b) => {
                                                covered_values.insert(b.to_string());
                                            }
                                            Constructor::EnumVariant {
                                                enum_name, variant, ..
                                            } => {
                                                covered_values
                                                    .insert(format!("{}::{}", enum_name, variant));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                        }
                        // Check if all expected values are covered
                        if !expected_values.iter().all(|v| covered_values.contains(v)) {
                            return vec![format!("{} {{ .. }}", name)];
                        }
                    }

                    vec![] // Exhaustive
                }
            } else {
                // Not an enum or struct, needs wildcard
                vec!["_".to_string()]
            }
        }

        Type::I64 | Type::I32 | Type::U64 | Type::U32 => {
            // v0.47: Improved integer range exhaustiveness
            // Collect all covered ranges/literals
            let mut covered_ranges: Vec<(i64, i64)> = vec![];

            for p in matrix {
                match &p.constructor {
                    Constructor::IntLit(n) => {
                        covered_ranges.push((*n, *n));
                    }
                    Constructor::IntRange { start, end } => {
                        covered_ranges.push((*start, *end));
                    }
                    _ => {}
                }
            }

            // Sort and merge overlapping ranges
            if covered_ranges.is_empty() {
                return vec!["_".to_string()];
            }

            covered_ranges.sort_by_key(|(s, _)| *s);
            let merged = merge_ranges(&covered_ranges);

            // Check for gaps and find missing patterns
            let mut missing = vec![];
            let type_range = match ty {
                Type::I64 => (i64::MIN, i64::MAX),
                Type::I32 => (i32::MIN as i64, i32::MAX as i64),
                Type::U64 => (0_i64, i64::MAX), // Approximate for u64
                Type::U32 => (0_i64, u32::MAX as i64),
                _ => (i64::MIN, i64::MAX),
            };

            // Find gaps in coverage
            let gaps = find_range_gaps(&merged, type_range);

            // Report up to 3 specific missing patterns
            for (i, (gap_start, gap_end)) in gaps.iter().enumerate() {
                if i >= 3 {
                    missing.push("...".to_string());
                    break;
                }
                if *gap_start == *gap_end {
                    missing.push(format!("{}", gap_start));
                } else if *gap_end - *gap_start <= 5 {
                    // Small range - list individual values
                    for v in *gap_start..=(*gap_end).min(*gap_start + 4) {
                        missing.push(format!("{}", v));
                    }
                    if *gap_end > *gap_start + 4 {
                        missing.push("...".to_string());
                    }
                } else {
                    missing.push(format!("{}..{}", gap_start, gap_end));
                }
            }

            if missing.is_empty() {
                vec![] // Exhaustive!
            } else {
                missing
            }
        }

        Type::F64 | Type::String => {
            // Truly infinite types - always need wildcard
            vec!["_".to_string()]
        }

        Type::Tuple(elem_types) => {
            // v0.55: Improved tuple exhaustiveness for finite element types
            // Check if all elements are finite (bool or known enum)
            let finite_values: Vec<Option<Vec<String>>> = elem_types
                .iter()
                .map(|t| get_finite_type_values(t, ctx))
                .collect();

            if finite_values.iter().all(|v| v.is_some()) {
                // All elements are finite - check full coverage
                let all_values: Vec<Vec<String>> = finite_values
                    .into_iter()
                    .map(|v| v.unwrap())
                    .collect();

                // Generate all possible combinations
                let all_combos = generate_tuple_combinations(&all_values);

                // Check which combinations are covered
                let mut covered: HashSet<Vec<String>> = HashSet::new();
                for p in matrix {
                    if p.is_wildcard() {
                        return vec![]; // Wildcard covers everything
                    }
                    if let Constructor::Tuple(_) = &p.constructor {
                        // Extract the concrete values this pattern covers
                        let pattern_values = extract_tuple_pattern_values(p, &all_values);
                        for pv in pattern_values {
                            covered.insert(pv);
                        }
                    }
                }

                // Find missing combinations
                let missing: Vec<String> = all_combos
                    .into_iter()
                    .filter(|c| !covered.contains(c))
                    .take(3) // Limit to 3 examples
                    .map(|c| format!("({})", c.join(", ")))
                    .collect();

                if missing.len() < 3 {
                    missing
                } else {
                    let mut result = missing;
                    result.push("...".to_string());
                    result
                }
            } else {
                // At least one element is infinite - check coverage with wildcards
                if matrix.is_empty() {
                    return vec![format!(
                        "({})",
                        vec!["_"; elem_types.len()].join(", ")
                    )];
                }

                // Check if any pattern is a full wildcard
                for p in matrix {
                    if p.is_wildcard() {
                        return vec![];
                    }
                    if let Constructor::Tuple(_) = &p.constructor
                        && p.fields.iter().all(|f| f.is_wildcard()) {
                            return vec![];
                        }
                }

                // For mixed finite/infinite: check if finite positions are exhaustive
                // and infinite positions have wildcards in all patterns
                let finite_positions: Vec<(usize, Vec<String>)> = finite_values
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| v.clone().map(|vals| (i, vals)))
                    .collect();

                // Check infinite positions all have wildcards
                let infinite_positions: Vec<usize> = finite_values
                    .iter()
                    .enumerate()
                    .filter_map(|(i, v)| if v.is_none() { Some(i) } else { None })
                    .collect();

                // All patterns must have wildcards at infinite positions
                let all_infinite_covered = matrix.iter().all(|p| {
                    if let Constructor::Tuple(_) = &p.constructor {
                        infinite_positions
                            .iter()
                            .all(|&pos| pos < p.fields.len() && p.fields[pos].is_wildcard())
                    } else {
                        p.is_wildcard()
                    }
                });

                if !all_infinite_covered {
                    return vec![format!(
                        "({})",
                        vec!["_"; elem_types.len()].join(", ")
                    )];
                }

                // Check finite positions are exhaustive
                for (pos, expected_values) in &finite_positions {
                    let mut covered_values: HashSet<String> = HashSet::new();
                    for p in matrix {
                        if p.is_wildcard() {
                            covered_values.extend(expected_values.iter().cloned());
                            continue;
                        }
                        if let Constructor::Tuple(_) = &p.constructor
                            && *pos < p.fields.len() {
                                if p.fields[*pos].is_wildcard() {
                                    covered_values.extend(expected_values.iter().cloned());
                                } else {
                                    match &p.fields[*pos].constructor {
                                        Constructor::BoolLit(b) => {
                                            covered_values.insert(b.to_string());
                                        }
                                        Constructor::EnumVariant {
                                            enum_name, variant, ..
                                        } => {
                                            covered_values
                                                .insert(format!("{}::{}", enum_name, variant));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                    }
                    // Check if all expected values are covered
                    if !expected_values.iter().all(|v| covered_values.contains(v)) {
                        return vec![format!(
                            "({})",
                            vec!["_"; elem_types.len()].join(", ")
                        )];
                    }
                }

                vec![] // Exhaustive
            }
        }

        Type::Array(_elem_ty, size) => {
            // For fixed-size arrays, check coverage
            if matrix.is_empty() {
                vec![format!("[{}]", vec!["_"; *size].join(", "))]
            } else {
                vec![]
            }
        }

        Type::Unit => vec![],

        _ => {
            // For other types, be conservative
            if matrix.is_empty() {
                vec!["_".to_string()]
            } else {
                vec![]
            }
        }
    }
}

/// Format a pattern for error messages
pub fn format_missing_pattern(pattern: &str) -> String {
    pattern.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_bool_exhaustiveness() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::Bool;

        // Both true and false covered
        let arms = vec![
            (
                Spanned::new(
                    Pattern::Literal(LiteralPattern::Bool(true)),
                    Span::new(0, 0),
                ),
                None,
            ),
            (
                Spanned::new(
                    Pattern::Literal(LiteralPattern::Bool(false)),
                    Span::new(0, 0),
                ),
                None,
            ),
        ];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(result.is_exhaustive);
        assert!(result.missing_patterns.is_empty());
    }

    #[test]
    fn test_bool_non_exhaustive() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::Bool;

        // Only true covered
        let arms = vec![(
            Spanned::new(
                Pattern::Literal(LiteralPattern::Bool(true)),
                Span::new(0, 0),
            ),
            None,
        )];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(!result.is_exhaustive);
        assert!(result.missing_patterns.contains(&"false".to_string()));
    }

    #[test]
    fn test_wildcard_exhaustive() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::I64;

        // Wildcard covers everything
        let arms = vec![(
            Spanned::new(Pattern::Wildcard, Span::new(0, 0)),
            None,
        )];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(result.is_exhaustive);
    }

    #[test]
    fn test_unreachable_arm() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::Bool;

        // Wildcard makes subsequent patterns unreachable
        let arms = vec![
            (Spanned::new(Pattern::Wildcard, Span::new(0, 0)), None),
            (
                Spanned::new(
                    Pattern::Literal(LiteralPattern::Bool(true)),
                    Span::new(0, 0),
                ),
                None,
            ),
        ];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(result.is_exhaustive);
        assert_eq!(result.unreachable_arms, vec![1]);
    }

    #[test]
    fn test_enum_exhaustiveness() {
        let mut ctx = ExhaustivenessContext::new();
        ctx.add_enum(
            "Option",
            vec![
                ("Some".to_string(), vec![Type::I64]),
                ("None".to_string(), vec![]),
            ],
        );

        let ty = Type::Named("Option".to_string());

        // Both variants covered
        let arms = vec![
            (
                Spanned::new(
                    Pattern::EnumVariant {
                        enum_name: "Option".to_string(),
                        variant: "Some".to_string(),
                        bindings: vec![Spanned::new(Pattern::Wildcard, Span::new(0, 0))],
                    },
                    Span::new(0, 0),
                ),
                None,
            ),
            (
                Spanned::new(
                    Pattern::EnumVariant {
                        enum_name: "Option".to_string(),
                        variant: "None".to_string(),
                        bindings: vec![],
                    },
                    Span::new(0, 0),
                ),
                None,
            ),
        ];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(result.is_exhaustive);
    }

    #[test]
    fn test_enum_non_exhaustive() {
        let mut ctx = ExhaustivenessContext::new();
        ctx.add_enum(
            "Option",
            vec![
                ("Some".to_string(), vec![Type::I64]),
                ("None".to_string(), vec![]),
            ],
        );

        let ty = Type::Named("Option".to_string());

        // Only Some covered
        let arms = vec![(
            Spanned::new(
                Pattern::EnumVariant {
                    enum_name: "Option".to_string(),
                    variant: "Some".to_string(),
                    bindings: vec![Spanned::new(Pattern::Wildcard, Span::new(0, 0))],
                },
                Span::new(0, 0),
            ),
            None,
        )];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(!result.is_exhaustive);
        assert!(result.missing_patterns.contains(&"Option::None".to_string()));
    }

    // v0.47: Integer range exhaustiveness tests

    #[test]
    fn test_merge_ranges() {
        // Empty
        assert_eq!(merge_ranges(&[]), vec![]);

        // Single range
        assert_eq!(merge_ranges(&[(0, 10)]), vec![(0, 10)]);

        // Non-overlapping
        assert_eq!(
            merge_ranges(&[(0, 5), (10, 15)]),
            vec![(0, 5), (10, 15)]
        );

        // Overlapping
        assert_eq!(merge_ranges(&[(0, 10), (5, 15)]), vec![(0, 15)]);

        // Adjacent
        assert_eq!(merge_ranges(&[(0, 5), (6, 10)]), vec![(0, 10)]);

        // Multiple merges
        assert_eq!(
            merge_ranges(&[(0, 3), (5, 8), (2, 6), (10, 12)]),
            vec![(0, 8), (10, 12)]
        );
    }

    #[test]
    fn test_find_range_gaps() {
        // Full coverage
        assert_eq!(find_range_gaps(&[(0, 10)], (0, 10)), vec![]);

        // Gap at start
        assert_eq!(find_range_gaps(&[(5, 10)], (0, 10)), vec![(0, 4)]);

        // Gap at end
        assert_eq!(find_range_gaps(&[(0, 5)], (0, 10)), vec![(6, 10)]);

        // Gap in middle
        assert_eq!(
            find_range_gaps(&[(0, 3), (7, 10)], (0, 10)),
            vec![(4, 6)]
        );

        // Multiple gaps
        assert_eq!(
            find_range_gaps(&[(2, 3), (6, 7)], (0, 10)),
            vec![(0, 1), (4, 5), (8, 10)]
        );
    }

    #[test]
    fn test_int_range_non_exhaustive() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::I64;

        // Only literal 0 covered - many values missing
        let arms = vec![(
            Spanned::new(
                Pattern::Literal(LiteralPattern::Int(0)),
                Span::new(0, 0),
            ),
            None,
        )];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(!result.is_exhaustive);
        // Should report missing values
        assert!(!result.missing_patterns.is_empty());
    }

    #[test]
    fn test_int_range_pattern() {
        let ctx = ExhaustivenessContext::new();
        let ty = Type::I64;

        // Range 0..10 and wildcard for rest - should be exhaustive
        let arms = vec![
            (
                Spanned::new(
                    Pattern::Range {
                        start: LiteralPattern::Int(0),
                        end: LiteralPattern::Int(10),
                        inclusive: true,
                    },
                    Span::new(0, 0),
                ),
                None,
            ),
            (
                Spanned::new(Pattern::Wildcard, Span::new(0, 0)),
                None,
            ),
        ];
        let result = check_exhaustiveness(&ty, &arms, &ctx);
        assert!(result.is_exhaustive);
    }
}
