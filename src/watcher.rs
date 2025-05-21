use anyhow::{Context, Result};
use colored::Colorize;
use log::{debug, error, info};
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent, DebouncedEventKind};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

const BUILD_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY: Duration = Duration::from_secs(1);

/// Watches the input paths defined in the Grimoire CSS configuration
/// and triggers a build when changes are detected.
pub struct GrimoireWatcher {
    /// Base directory where the command will be executed
    base_dir: PathBuf,
    /// Files to watch for changes
    files_to_watch: Vec<PathBuf>,
    /// Debounce duration to prevent multiple builds on rapid changes
    debounce_duration: Duration,
}

impl GrimoireWatcher {
    /// Creates a new watcher for the specified files and directories
    pub fn new(
        base_dir: PathBuf,
        files_to_watch: Vec<PathBuf>,
        debounce_duration: Duration,
    ) -> Self {
        Self {
            base_dir,
            files_to_watch,
            debounce_duration,
        }
    }

    /// Start watching files for changes
    pub fn start_watching(&self) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        let running = Arc::new(AtomicBool::new(true));
        let r = running.clone();

        ctrlc::set_handler(move || {
            info!(
                "{}",
                "\nReceived shutdown signal, gracefully stopping...".yellow()
            );
            r.store(false, Ordering::SeqCst);
        })?;

        // Create a debounced watcher with the correct API usage
        let mut debouncer = new_debouncer(
            self.debounce_duration,
            move |res: Result<Vec<DebouncedEvent>, notify::Error>| {
                match res {
                    Ok(events) => {
                        // Only trigger if there are changes that match our watched files
                        for event in events.iter() {
                            if event.kind == DebouncedEventKind::Any {
                                if let Err(e) = tx.send(()) {
                                    error!("Error sending watch event: {}", e);
                                }
                                break;
                            }
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                }
            },
        )?;

        for path in &self.files_to_watch {
            if let Some(parent) = path.parent() {
                debouncer
                    .watcher()
                    .watch(parent, RecursiveMode::Recursive)?;
            }
        }

        // Log all individual files we're watching
        info!("Files being monitored:");
        for file in &self.files_to_watch {
            info!("  {}", file.display().to_string().cyan());
        }

        info!("{}", "Waiting for file changes...".green().bold());

        // Main event processing loop
        while running.load(Ordering::SeqCst) {
            match rx.try_recv() {
                Ok(_) => {
                    info!("{}", "File change detected, rebuilding...".yellow().bold());
                    self.run_build_with_retry()?;
                    info!("{}", "Waiting for file changes...".green().bold());
                }
                Err(TryRecvError::Empty) => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(TryRecvError::Disconnected) => {
                    error!("Watch channel disconnected");
                    break;
                }
            }
        }

        info!("{}", "Shutting down watcher...".yellow());
        Ok(())
    }

    /// Run the grimoire_css build command with retry logic
    fn run_build_with_retry(&self) -> Result<()> {
        for attempt in 1..=MAX_RETRIES {
            match self.run_build() {
                Ok(_) => {
                    info!("{}", "Build completed successfully".green().bold());
                    return Ok(());
                }
                Err(e) => {
                    if attempt < MAX_RETRIES {
                        error!(
                            "{}: {} (attempt {}/{})",
                            "Build failed".red().bold(),
                            e,
                            attempt,
                            MAX_RETRIES
                        );
                        std::thread::sleep(RETRY_DELAY);
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Run the grimoire_css build command
    fn run_build(&self) -> Result<()> {
        info!("Running grimoire_css build...");
        let start_time = Instant::now();

        let mut child = Command::new("grimoire_css")
            .arg("build")
            .current_dir(&self.base_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to start grimoire_css build")?;

        // Wait for the process to finish with a timeout
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    if !status.success() {
                        let mut stderr_vec = Vec::new();
                        if let Some(mut stderr) = child.stderr.take() {
                            stderr.read_to_end(&mut stderr_vec)?;
                        }
                        let stderr = String::from_utf8_lossy(&stderr_vec);
                        anyhow::bail!("grimoire_css build failed: {}", stderr);
                    }
                    break;
                }
                Ok(None) => {
                    if start_time.elapsed() > BUILD_TIMEOUT {
                        child.kill()?;
                        anyhow::bail!("Build timed out after {:?}", BUILD_TIMEOUT);
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(e) => anyhow::bail!("Error waiting for build process: {}", e),
            }
        }

        debug!("Build completed in {:?}", start_time.elapsed());
        Ok(())
    }
}
