use nex_lsp::run_lsp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_lsp().await?;
    Ok(())
}