use std::{env, fs};

use color_eyre::Result;
use color_eyre::eyre::eyre;
use rustic_calc::{
    io::{get_state_from_file, reset_file_state},
    tui_app::App,
};

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
    Clear {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {} => run(),
        Commands::Clear {} => clear(),
    }
}

fn run() -> Result<()> {
    let home = env::var("HOME").map_err(|_| eyre!("HOME is not set"))?;
    fs::create_dir_all(format!("{home}/.config/rcalc"))?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_state = get_state_from_file();
    let app_result = match app_state {
        Ok(state) => App::from(&state).run(terminal),
        Err(_) => App::new().run(terminal),
    };
    ratatui::restore();
    app_result
}

fn clear() -> Result<()> {
    let _ = reset_file_state();
    Ok(())
}
