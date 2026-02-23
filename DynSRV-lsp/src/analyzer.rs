use regex::Regex;
use lalrpop_util::ParseError;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification;
use trustworthiness_checker::lang::dynamic_lola::lalr_parser;
// use trustworthiness_checker::lang::dynamic_lola::parser::lola_specification;
use trustworthiness_checker::lang::dynamic_lola::type_checker::TypedLOLASpecification;

pub struct Analysis {
    pub spec: Option<LOLASpecification>, // The parsed specification, if parsing was successful
    pub typed: Option<TypedLOLASpecification>, //For future use, when type checker is implemented
    pub diags: Vec<Diagnostic>,          // Diagnostics from both syntax and semantic analysis
}

impl Analysis {
    // Create Clone function for Analysis struct
    pub fn clone(&self) -> Self {
        Self {
            spec: self.spec.clone(),
            typed: self.typed.clone(),
            diags: self.diags.clone(),
        }
    }

    pub async fn analyze(text: &str) -> Analysis {
        match lalr_parser::parse_str(text) {
            Ok(spec) => {
                //Found no syntax error in the code return empty diagnostics
                Analysis {
                    spec: Some(spec),
                    typed: None,
                    diags: vec![],
                }
            }

            Err(parse_error) => {
              //Returned to only parsing one line for now
                let range = Analysis::extract_range_from_error(&(format!("{:?}", parse_error)))
                    .unwrap_or_default();
                  log::info!("{:?}", range);

                Analysis {
                    spec: None,
                    typed: None,
                    diags: vec![Diagnostic {
                        range: range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("DSRV".into()),
                        message: parse_error.to_string(),
                        ..Default::default()
                    }],
                }

                // Removed the code checking each line for errors, to ensure only actual errors are reported to the client, even if its only the one
                // //Found at least one error in the code, checking the code again for more errors by running each line individual
                // let mut more_diags: Vec<Diagnostic> = Vec::new();

                // for (line_num, line) in text.lines().enumerate() {
                //     //Running each line of the input code
                //     match lalr_parse_file(line) {
                //         Ok(_spec) => {
                //             //No errors on this line, running next line
                //         }

                //         Err(error) => {
                //             //Error found on line, creating error message and diagnostic to return to the client
                //             let msg = format!("{:?}", error); // Convert the error to a string

                //             //Extract Range from the error message using regex
                //             let range =
                //                 Analysis::extract_range_from_error(&msg).unwrap_or_default();

                //             let error_message = Analysis::contruct_error_message(&msg);

                //             //Add the diagnostic to the vector
                //             more_diags.push(Diagnostic {
                //                 range: Range::new(
                //                     Position::new(line_num as u32, range.start.character),
                //                     Position::new(line_num as u32, range.end.character),
                //                 ),
                //                 severity: Some(DiagnosticSeverity::ERROR),
                //                 source: Some("DSRV".into()),
                //                 message: error_message,
                //                 ..Default::default()
                //             });
                //         }
                //     }
                // }

                // //Return the diagnostics to the client
            }
        }
    }

    fn extract_range_from_error(msg: &str) -> Option<Range> {
        //Construct the regex pattern to extract the line and column numbers from the error message
        let re = Regex::new(r"found at line (\d+), column (\d+):line (\d+), column (\d+)").ok()?;
        //Get the numbers from the error message using regex
        let cap = re.captures(msg)?;

        // Parse the captured groups into u32 values for line and column numbers
        let line_start: u32 = cap.get(1)?.as_str().parse().ok()?;
        let char_start: u32 = cap.get(2)?.as_str().parse().ok()?;
        let line_end: u32 = cap.get(3)?.as_str().parse().ok()?;
        let char_end: u32 = cap.get(4)?.as_str().parse().ok()?;

        //Create the Range object using the extracted line and column numbers, adjusting for zero-based indexing
        Some(Range::new(
            Position::new(line_start - 1, char_start - 1),
            Position::new(line_end - 1, char_end - 1),
        ))
    }

    fn contruct_error_message(msg: &str) -> String {
        let mut lines = msg.lines();

        let mut l = lines.nth(3).unwrap_or_default().split(" found ");
        format!(
            "Syntax error: {:?}",
            l.nth(0).unwrap_or_default().trim_start()
        )
    }
}
