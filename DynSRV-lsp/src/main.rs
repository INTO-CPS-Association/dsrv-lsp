use tower_lsp::{LspService, Server};
use dynsrv_lsp::Backend;


#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
      .with_writer(std::io::stderr)
      .with_max_level(tracing::Level::INFO)
      .init();

  
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();
  
  let (service, socket) = LspService::build(|client| {Backend::new(client)}).finish();
  Server::new(stdin, stdout, socket).serve(service).await;
}
  