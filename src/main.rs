mod config;
mod watcher;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use config::Config;
use env_logger::Env;
use log::{error, info};
use std::path::{Path, PathBuf};
use std::time::Duration;
use watcher::GrimoireWatcher;

/// Grimoire CSS Watcher - Monitors files defined in grimoire.config.json and triggers builds
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the project directory (defaults to current directory)
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Debounce duration in milliseconds to avoid multiple builds for rapid changes
    #[arg(short, long, default_value_t = 300)]
    debounce: u64,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set up logging
    let env = if args.verbose {
        Env::default().default_filter_or("debug")
    } else {
        Env::default().default_filter_or("info")
    };
    env_logger::Builder::from_env(env)
        .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
        .init();

    println!("\n{}\n", "GRIMOIRE CSS WATCHER".bright_magenta().bold());

    // Get the project directory
    let project_dir = match &args.path {
        Some(path) => path.clone(),
        None => std::env::current_dir()?,
    };

    info!(
        "Starting watcher in directory: {}",
        project_dir.display().to_string().cyan()
    );

    // Try to load the config file
    match run_watcher(&project_dir, Duration::from_millis(args.debounce)) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("{}: {}", "Error".red().bold(), e);
            error!(
                "{}",
                "Make sure you have a valid grimoire.config.json file in the grimoire directory"
                    .yellow()
            );
            Err(e)
        }
    }
}

fn run_watcher(project_dir: &Path, debounce_duration: Duration) -> Result<()> {
    let config_path = project_dir.join("grimoire/config/grimoire.config.json");

    if !config_path.exists() {
        return Err(anyhow::anyhow!(
            "Grimoire config file not found at: {}",
            config_path.display()
        ));
    }

    let config = Config::load(&config_path)?;

    // Get files and directories to watch
    let files_to_watch = config.get_files_to_watch(&config_path);
    if files_to_watch.is_empty() {
        info!(
            "{}",
            "No files to watch found in the configuration. Make sure your input_paths are correct."
                .yellow()
        );
        return Ok(());
    }

    let watcher =
        GrimoireWatcher::new(project_dir.to_path_buf(), files_to_watch, debounce_duration);

    info!("{}", "Watcher initialized successfully".green());
    info!(
        "Debounce duration set to {} ms",
        debounce_duration.as_millis()
    );

    watcher.start_watching()?;

    Ok(())
}
