use std::error::Error;
use zen::lsp::ZenLanguageServer;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .format_target(false)
        .init();

    let server = ZenLanguageServer::new()?;
    server.run()
}
