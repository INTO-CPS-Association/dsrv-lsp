use regex::Regex;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification;
use trustworthiness_checker::lang::dynamic_lola::lalr_parser::parse_str as lalr_parse_file; // Model // Parser
// use trustworthiness_checker::lang::dynamic_lola::parser::lola_specification;
use ropey::Rope;
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

// Function to analyze the lines of the document and return diagnostics
pub async fn analyze(text: &str) -> Analysis {
    match lalr_parse_file(text) {
        Ok(spec) => Analysis {
            spec: Some(spec),
            typed: None,
            diags: vec![],
        },

        Err(_parse_error) => {
            let mut diagnostics: Vec<Diagnostic> = Vec::new();

            for (linenum, line) in text.lines().enumerate() {
                match lalr_parse_file(line) {
                    Ok(_spec) => {
                        // Success on individual line, continue
                    }
                    Err(error) => {
                        let msg_line = format!("{:?}", error);
                        let rope = Rope::from_str(&line);
                        let range = extract_range_from_error(&msg_line, &rope).unwrap_or_default();

                        //extract info from msg_line
                        // log::info!("{}", msg_line);
                        
                        let lines = msg_line.lines();

                        let error_line = lines.clone().nth(3).unwrap_or_default().split(" found ");
                        let error_msg = "Syntax error: ".to_string()
                            + error_line.clone().nth(0).unwrap_or_default().trim_start();

                        let diag = Diagnostic {
                            range: Range::new(
                                Position::new(linenum as u32, range.start.character - 1),
                                Position::new(linenum as u32, range.end.character - 1),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: error_msg,
                            source: Some("DynSRV".into()),
                            ..Default::default()
                        };

                        diagnostics.push(diag);
                    }
                }
            }

            // let msg = format!("{:?}", parse_error);
            // let rope = Rope::from_str(&text);
            // let range = extract_range_from_error(&msg, &rope).unwrap_or_default();
            Analysis {
                spec: None,
                typed: None,
                diags: diagnostics, // vec![Diagnostic {
                                    //     range: range,
                                    //     severity: Some(DiagnosticSeverity::ERROR),
                                    //     message: format!("Syntax error: {:?}", parse_error),
                                    //     source: Some("lola-parser".into()),
                                    //     ..Default::default()
                                    // }],
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
    //Create string to look for in the error message
    let re = Regex::new(r"found at line (\d+), column (\d+):line (\d+), column (\d+)").ok()?;
    let cap = re.captures(msg)?;

    //Only get char loc
    let char_start: usize = cap.get(2)?.as_str().parse().ok()?;
    let char_end: usize = cap.get(4)?.as_str().parse().ok()?;
    
    Some(Range::new(
        offset_to_pos(char_start, rope)?,
        offset_to_pos(char_end, rope)?,
    ))
}
