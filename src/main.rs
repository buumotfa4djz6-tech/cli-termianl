use anyhow::Result;
use clap::Parser;

use cli_terminal::app::App;

#[derive(Parser)]
#[command(name = "cli-terminal", about = "TUI terminal interaction enhancer")]
struct Args {
    /// Target program to connect to
    target: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    App::new(args.target)?.run()
}
