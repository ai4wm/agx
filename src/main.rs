use agx::app::{App, AppOptions};
use agx::SplitDirection;
use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "agx",
    version,
    about = "Cross-platform AI agent terminal multiplexer"
)]
struct Cli {
    /// Agent commands to run. Repeat the flag to launch multiple panes.
    #[arg(short, long)]
    run: Vec<String>,

    /// Split direction for the pane layout. Overrides config.toml.
    #[arg(short, long, value_enum)]
    split: Option<SplitDirection>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async move {
        let mut app = App::new(AppOptions {
            run: cli.run,
            split: cli.split,
        })?;
        app.run().await
    })
}
