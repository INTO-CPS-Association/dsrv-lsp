use trustworthiness_checker::lang::dynamic_lola::ast::LOLASpecification; // Model
use trustworthiness_checker::lang::dynamic_lola::lalr_parser::parse_file as lalr_parse_file; // Parser
// use trustworthiness_checker::lang::dynamic_lola::type_checker::TypedLOLASpecification;

pub struct Analysis {
    pub model: Option<LOLASpecification>,
    pub diags: Vec<String>,
}

pub async fn analyze(text: &str) -> Analysis {
    let diagnostics = Vec::new();

    match lalr_parse_file(text).await {
        Ok(model) => Analysis {
            model: Some(model),
            diags: (diagnostics),
        },

        Err(e) => Analysis {
            model: None,
            diags: vec![format!("Syntax Error: {:?}", e)],
        },
    }
}
