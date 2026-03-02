use color_eyre::Result;
use rustic_calc::tui_app::App;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rcalc")]
#[command(about = "Run rust calc")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the application using cargo
    Run {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {} => run(),
    }
}

fn run() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}
