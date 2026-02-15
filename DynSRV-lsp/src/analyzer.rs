use regex::Regex;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification; // Model
use trustworthiness_checker::lang::dynamic_lola::lalr_parser::parse_str as lalr_parse_file; // Parser
// use trustworthiness_checker::lang::dynamic_lola::parser::lola_specification;
use ropey::Rope;
use trustworthiness_checker::lang::dynamic_lola::type_checker::{
    SemanticError, TypedLOLASpecification, type_check,
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

// Function to analyze the lines of the document and return diagnostics
pub async fn analyze(text: &str) -> Analysis {
    match lalr_parse_file(text) {
        Ok(spec) => Analysis {
            spec: Some(spec),
            typed: None,
            diags: vec![],
        },

        Err(parse_error) => {
            let msg = format!("{:?}", parse_error);
            let rope = Rope::from_str(&text);
            let range = extract_range_from_error(&msg, &rope).unwrap_or_default();
            Analysis {
                spec: None,
                typed: None,
                diags: vec![Diagnostic {
                    range: range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Syntax error: {:?}", parse_error),
                    source: Some("lola-parser".into()),
                    ..Default::default()
                }],
            }
        }
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


// Credit to github user: IWANABETHATGUY for the following functions to extract error locations from the parser error messages.
// Found in https://github.com/IWANABETHATGUY/tower-lsp-boilerplate/src/main.rs lines 805-810 for the `offset_to_pos` function and lines 812-816 for the `pos_to_offset` function. 
fn offset_to_pos(offset: usize, rope: &Rope) -> Option<Position> {
    let line = rope.try_char_to_line(offset).ok()?;
    let first_char_line = rope.try_line_to_char(line).ok()?;
    let column = offset - first_char_line;
    Some(Position::new(line as u32, column as u32))
}

// fn pos_to_offset(position: Position, rope: &Rope) -> Option<usize> {
//     let line_char_offset = rope.try_line_to_char(position.line as usize).ok()?;
//     let slice = rope.slice(0..line_char_offset + position.character as usize);
//     Some(slice.len_bytes())
// }


// Helper function for extracting error locations from the parser error messages. It uses a regular expression to find the character offsets in the error message and converts them to LSP positions using the `offset_to_pos` function.
fn extract_range_from_error(msg: &str, rope: &Rope) -> Option<Range> {
    let re = Regex::new(r"found at (\d+):(\d+)").ok()?;
    let cap = re.captures(msg)?;

    let char_start: usize = cap.get(1)?.as_str().parse().ok()?;
    let char_end: usize = cap.get(2)?.as_str().parse().ok()?;

    Some(Range::new(
        offset_to_pos(char_start, rope)?,
        offset_to_pos(char_end, rope)?,
    ))
}
