//! Binary entry point: parse CLI, get terminal size, run ASCII video playback.

use std::path::PathBuf;

use clap::Parser;
use crossterm::terminal::size;
use tracing_subscriber::EnvFilter;
use worse_apple::stream;

#[derive(Parser, Debug)]
#[command(name = "worse-apple", about = "Play video in the terminal as ASCII art")]
struct Args {
    /// Path to an MP4 (or other format supported by ffmpeg) file
    path: PathBuf,

    /// Frames per second for playback (default: 24)
    #[arg(short, long, default_value = "24")]
    fps: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("worse_apple=info".parse()?))
        .init();

    let args = Args::parse();
    let (cols, rows) = size().unwrap_or((80, 24));

    stream::play(&args.path, cols, rows, args.fps)?;
    Ok(())
}
