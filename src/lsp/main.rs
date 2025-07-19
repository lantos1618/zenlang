use zen::lsp;

#[tokio::main]
async fn main() {
    lsp::run_lsp_server().await;
} 