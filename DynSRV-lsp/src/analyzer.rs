use lalrpop_util::ParseError;
use ropey::Rope;
use tower_lsp::lsp_types::*;
use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification;
use trustworthiness_checker::lang::dynamic_lola::lalr::TopDeclsParser;
use trustworthiness_checker::lang::dynamic_lola::lalr_parser::create_lola_spec;
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

    pub async fn analyze_2_point_0(text: &str) -> Analysis {
        match TopDeclsParser::new().parse(text) {
            Ok(stmts) => Analysis {
                spec: Some(create_lola_spec(&stmts)),
                typed: None,
                diags: vec![],
            },

            Err(error) => {
                let error = error.map_location(|byte| Analysis::byte_to_pos(text, byte));

                let diags = match error {
                    ParseError::InvalidToken { location } => {
                        let range =
                            Range::new(location.unwrap_or_default(), location.unwrap_or_default());
                        Self::create_diag("Invalid Token", range)
                    }
                    ParseError::UnrecognizedEof {
                        location,
                        expected: _,
                    } => {
                        let range = Range {
                            start: location.unwrap_or_default(),
                            end: location.unwrap_or_default(),
                        };

                        Self::create_diag("Syntax error: Unexpected EOF", range)
                    }

                    ParseError::UnrecognizedToken { token, expected: _ } => {
                        let (start, _tok, end) = token;
                        Self::create_diag(
                            "Syntax error: Unrecognized token",
                            Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
                        )
                    }
                    ParseError::ExtraToken { token } => {
                        let (start, _tok, end) = token;

                        Self::create_diag(
                            "Syntax error: Extra token:",
                            Range::new(start.unwrap_or_default(), end.unwrap_or_default()),
                        )
                    }

                    ParseError::User { error } => {
                        let p = Position::new(1, 1);
                        Self::create_diag(&format!("User error: {:?}", error), Range::new(p, p))
                    }
                };

                Analysis {
                    spec: None,
                    typed: None,
                    diags: vec![diags],
                }
            }
        }
    }

    fn create_diag(msg: &str, range: Range) -> Diagnostic {
        Diagnostic {
            range: range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("DSRV".into()),
            message: msg.into(),
            ..Default::default()
        }
    }

    // fn extract_range_from_error(msg: &str) -> Option<Range> {
    //     //Construct the regex pattern to extract the line and column numbers from the error message
    //     let re = Regex::new(r"found at line (\d+), column (\d+):line (\d+), column (\d+)").ok()?;
    //     //Get the numbers from the error message using regex
    //     let cap = re.captures(msg)?;

    //     // Parse the captured groups into u32 values for line and column numbers
    //     let line_start: u32 = cap.get(1)?.as_str().parse().ok()?;
    //     let char_start: u32 = cap.get(2)?.as_str().parse().ok()?;
    //     let line_end: u32 = cap.get(3)?.as_str().parse().ok()?;
    //     let char_end: u32 = cap.get(4)?.as_str().parse().ok()?;

    //     //Create the Range object using the extracted line and column numbers, adjusting for zero-based indexing
    //     Some(Range::new(
    //         Position::new(line_start - 1, char_start - 1),
    //         Position::new(line_end - 1, char_end - 1),
    //     ))
    // }

    // fn contruct_error_message(msg: &str) -> String {
    //     let mut lines = msg.lines();

    //     let mut l = lines.nth(3).unwrap_or_default().split(" found ");
    //     format!(
    //         "Syntax error: {:?}",
    //         l.nth(0).unwrap_or_default().trim_start()
    //     )
    // }

    fn byte_to_pos(text: &str, byte: usize) -> Option<Position> {
        let rope = Rope::from_str(text);
        let line = rope.byte_to_line(byte);
        let line_start = rope.line_to_byte(line);

        let col = byte - line_start;

        Some(Position::new(line as u32, col as u32))
    }
}

// let line = rope.try_char_to_line(offset).ok()?;
// let first_char_of_line = rope.try_line_to_char(line).ok()?;
// let column = offset - first_char_of_line;
// Some(Position::new(line as u32, column as u32))

// pub async fn analyze(text: &str) -> Analysis {
//     match lalr_parser::parse_str(text) {
//         Ok(spec) => {
//             //Found no syntax error in the code return empty diagnostics
//             Analysis {
//                 spec: Some(spec),
//                 typed: None,
//                 diags: vec![],
//             }
//         }

//         Err(parse_error) => {
//           //Returned to only parsing one line for now
//             let range = Analysis::extract_range_from_error(&(format!("{:?}", parse_error)))
//                 .unwrap_or_default();
//               log::info!("{:?}", range);

//             Analysis {
//                 spec: None,
//                 typed: None,
//                 diags: vec![Diagnostic {
//                     range: range,
//                     severity: Some(DiagnosticSeverity::ERROR),
//                     source: Some("DSRV".into()),
//                     message: parse_error.to_string(),
//                     ..Default::default()
//                 }],
//             }

//             // Removed the code checking each line for errors, to ensure only actual errors are reported to the client, even if its only the one
//             // //Found at least one error in the code, checking the code again for more errors by running each line individual
//             // let mut more_diags: Vec<Diagnostic> = Vec::new();

//             // for (line_num, line) in text.lines().enumerate() {
//             //     //Running each line of the input code
//             //     match lalr_parse_file(line) {
//             //         Ok(_spec) => {
//             //             //No errors on this line, running next line
//             //         }

//             //         Err(error) => {
//             //             //Error found on line, creating error message and diagnostic to return to the client
//             //             let msg = format!("{:?}", error); // Convert the error to a string

//             //             //Extract Range from the error message using regex
//             //             let range =
//             //                 Analysis::extract_range_from_error(&msg).unwrap_or_default();

//             //             let error_message = Analysis::contruct_error_message(&msg);

//             //             //Add the diagnostic to the vector
//             //             more_diags.push(Diagnostic {
//             //                 range: Range::new(
//             //                     Position::new(line_num as u32, range.start.character),
//             //                     Position::new(line_num as u32, range.end.character),
//             //                 ),
//             //                 severity: Some(DiagnosticSeverity::ERROR),
//             //                 source: Some("DSRV".into()),
//             //                 message: error_message,
//             //                 ..Default::default()
//             //             });
//             //         }
//             //     }
//             // }

//             // //Return the diagnostics to the client
//         }
//     }
// }
