use crate::error::Result;
use crate::parser::CodeAst;

/// Transformer for language-specific AST extraction and transformation
pub struct Transformer;

#[derive(Debug, Clone)]
pub struct TransformResult {
    pub original_language: String,
    pub transformed_language: Option<String>,
    pub transformations_applied: Vec<String>,
    pub optimizations: Vec<String>,
    pub warnings: Vec<String>,
}

impl Transformer {
    /// Transform AST to extract and enhance information
    pub fn transform(ast: &CodeAst, target_language: Option<&str>) -> Result<TransformResult> {
        let mut result = TransformResult {
            original_language: ast.language.clone(),
            transformed_language: target_language.map(String::from),
            transformations_applied: Vec::new(),
            optimizations: Vec::new(),
            warnings: Vec::new(),
        };

        // Language-specific transformations
        match ast.language.as_str() {
            "rust" => {
                Self::transform_rust(ast, &mut result)?;
            }
            "typescript" => {
                Self::transform_typescript(ast, &mut result)?;
            }
            "python" => {
                Self::transform_python(ast, &mut result)?;
            }
            _ => {
                result.transformations_applied
                    .push(format!("Basic transformation for {}", ast.language));
            }
        }

        Ok(result)
    }

    fn transform_rust(ast: &CodeAst, result: &mut TransformResult) -> Result<()> {
        // Check for common patterns and suggest optimizations
        if !ast.functions.is_empty()
            && ast.functions.iter().any(|f| f.contains("_async")) {
                result.optimizations.push(
                    "Consider using tokio runtime for async code optimization".to_string(),
                );
            }

        if ast.traits.len() > 5 {
            result.optimizations.push(
                "High trait count detected - consider trait composition".to_string(),
            );
        }

        result
            .transformations_applied
            .push("Applied Rust-specific AST transformation".to_string());

        Ok(())
    }

    fn transform_typescript(ast: &CodeAst, result: &mut TransformResult) -> Result<()> {
        // Check for TypeScript-specific patterns
        if ast.interfaces.is_empty() && !ast.classes.is_empty() {
            result
                .warnings
                .push("No interfaces defined - consider using interfaces for better type safety"
                    .to_string());
        }

        if ast.functions.len() > ast.classes.len() {
            result.optimizations.push(
                "Consider using more OOP patterns with classes".to_string(),
            );
        }

        result
            .transformations_applied
            .push("Applied TypeScript-specific AST transformation".to_string());

        Ok(())
    }

    fn transform_python(ast: &CodeAst, result: &mut TransformResult) -> Result<()> {
        // Check for Python-specific patterns
        if ast.classes.is_empty() && !ast.functions.is_empty() {
            result.warnings.push(
                "Functional style detected - consider using classes for better organization"
                    .to_string(),
            );
        }

        result
            .transformations_applied
            .push("Applied Python-specific AST transformation".to_string());

        Ok(())
    }

    /// Extract semantic features from code
    pub fn extract_features(ast: &CodeAst) -> Vec<String> {
        let mut features = Vec::new();

        // Count metrics
        if !ast.functions.is_empty() {
            features.push(format!("defines_{}_{}_functions", ast.language, ast.functions.len()));
        }

        if !ast.classes.is_empty() {
            features.push(format!("defines_{}_{}_classes", ast.language, ast.classes.len()));
        }

        if !ast.traits.is_empty() {
            features.push(format!("uses_{}_{}_traits", ast.language, ast.traits.len()));
        }

        if !ast.imports.is_empty() {
            features.push(format!("has_{}_{}_dependencies", ast.language, ast.imports.len()));
        }

        features
    }

    /// Check code quality metrics
    pub fn compute_quality_metrics(ast: &CodeAst) -> CodeQualityMetrics {
        CodeQualityMetrics {
            cyclomatic_complexity_estimate: estimate_complexity(ast),
            api_surface_size: ast.functions.len() + ast.classes.len() + ast.traits.len(),
            dependency_count: ast.imports.len(),
            modularity_score: compute_modularity(ast),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeQualityMetrics {
    pub cyclomatic_complexity_estimate: f32,
    pub api_surface_size: usize,
    pub dependency_count: usize,
    pub modularity_score: f32,
}

fn estimate_complexity(ast: &CodeAst) -> f32 {
    // Simple estimation based on AST structure
    let func_count = ast.functions.len() as f32;
    let class_count = ast.classes.len() as f32;
    let trait_count = ast.traits.len() as f32;

    (func_count + (class_count * 2.0) + (trait_count * 1.5)).min(100.0)
}

fn compute_modularity(ast: &CodeAst) -> f32 {
    // Score based on how well the code is modularized
    let total_elements = ast.functions.len() + ast.classes.len() + ast.traits.len();

    if total_elements == 0 {
        return 0.0;
    }

    // Higher score for more classes/traits relative to functions
    let class_trait_ratio = (ast.classes.len() + ast.traits.len()) as f32 / total_elements as f32;
    (class_trait_ratio * 100.0).min(100.0)
}
