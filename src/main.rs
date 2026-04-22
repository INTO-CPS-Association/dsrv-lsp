/*
 * Copyright (c) 2026 Emilie Bang Holmberg (github.com/EmmiPigen).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License.
 *
 * This project utilizes the 'trustworthiness-checker' crate, which is
 * property of the INTO-CPS Association and used under the ICAPL (GPL Mode).
 */

use tower_lsp_server::{LspService, Server};
use dsrv_lsp::server::Backend;


#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
      .with_writer(std::io::stderr)
      .with_max_level(tracing::Level::INFO)
      .with_ansi(false)
      .init();

  
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();
  
  let (service, socket) = LspService::build(|client| {Backend::new(client)}).finish();
  Server::new(stdin, stdout, socket).serve(service).await;
}


// #[cfg(test)]
// mod test {
//   use macro_rules_attribute::apply;
//   use trustworthiness_checker::async_test;
  
//   use super::*;
  
  
  
// }