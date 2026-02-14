use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification; // Model
use trustworthiness_checker::lang::dynamic_lola::lalr_parser::parse_str as lalr_parse_file; // Parser
use trustworthiness_checker::lang::dynamic_lola::type_checker::{
    SemanticError, TypedLOLASpecification,
};

pub struct Analysis {
    pub spec: Option<LOLASpecification>,
    pub typed: Option<TypedLOLASpecification>,
    pub diags: Vec<Diagnostic>,
}

impl Analysis {
    pub fn semantics_to_diagnostics(errors: &[SemanticError]) -> Vec<Diagnostic> {
        errors
            .iter()
            .map(|error| Diagnostic {
                range: Range::default(),
                severity: Some(match error {
                    SemanticError::TypeError(_) => DiagnosticSeverity::ERROR,
                    SemanticError::DeferredError(_) => DiagnosticSeverity::WARNING,
                    SemanticError::UndeclaredVariable(_) => DiagnosticSeverity::ERROR,
                }),
                code: Some(NumberOrString::String(error_code(error).to_string())),
                message: format_error_message(error),
                source: Some("lola-type-checker".to_string()),
                ..Default::default()
            })
            .collect()
    }
    pub fn clone(&self) -> Self {
        Self {
            spec: self.spec.clone(),
            typed: self.typed.clone(),
            diags: self.diags.clone(),
        }
    }
}

// Function to analyze the text and produce an Analysis struct
pub async fn analyze(text: &str) -> Analysis {
    match lalr_parse_file(text) {
        Ok(spec) => {
            let mut diagnostics = Vec::new();

            for (_var, expr) in spec.exprs() {
                for input in expr.inputs() {
                    if !spec.input_vars.contains(&input)
                        && !spec.output_vars.contains(&input)
                        && !spec.exprs.contains_key(&input)
                    {
                        diagnostics.push(Diagnostic {
                            range: Range::default(),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: format!("Undeclared Variable `{}`", input),
                            source: Some("lola-semantic".into()),
                            ..Default::default()
                        });
                    }
                }
            }
            Analysis {
              spec: Some(spec),
              typed: None,
              diags: diagnostics,
            }
        }

        Err(parse_error) => Analysis {
            spec: None,
            typed: None,
            diags: vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Syntax error: {:?}", parse_error),
                source: Some("lola-parser".into()),
                ..Default::default()
            }],
        },
    }
}

fn error_code(error: &SemanticError) -> &'static str {
    match error {
        SemanticError::TypeError(_) => "E001",
        SemanticError::DeferredError(_) => "E002",
        SemanticError::UndeclaredVariable(_) => "E003",
    }
}

fn format_error_message(error: &SemanticError) -> String {
    match error {
        SemanticError::TypeError(msg) => format!("Type Error: {}", msg),
        SemanticError::DeferredError(msg) => format!("Deferred Error: {}", msg),
        SemanticError::UndeclaredVariable(var) => format!("Undeclared Variable: {}", var),
    }
}