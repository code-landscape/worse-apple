//! Bounded producer–consumer: decoder thread pushes frames; main thread pulls and displays.
//!
//! `play()` spawns the decoder on a thread, then runs the display loop (receive → ASCII → clear
//! and print → sleep) until the stream ends. Channel capacity limits decoded-ahead frames.

use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use crossterm::terminal::{Clear, ClearType};
use crossterm::ExecutableCommand;
use tracing::instrument;

use crate::ascii;
use crate::decode;

/// Default number of frames to decode ahead (backpressure when full).
const CHANNEL_CAP: usize = 4;

/// Runs the decoder in a thread and the display loop in the current thread.
///
/// Blocks until the video ends or the decoder errors. Uses `cols`×`rows` for ASCII size;
/// if either is 0, skips terminal clear/positioning and prints raw (for piping).
#[instrument(skip_all)]
pub fn play(path: &Path, cols: u16, rows: u16, fps: u32) -> Result<(), PlayError> {
    let (tx, rx) = mpsc::sync_channel(CHANNEL_CAP);
    let join = decode::spawn(path.to_path_buf(), tx);
    let frame_duration = Duration::from_secs_f64(1.0 / (fps as f64));

    let mut stdout = std::io::stdout();
    while let Ok(frame) = rx.recv() {
        let buf = ascii::frame_to_ascii(&frame, cols, rows);
        if cols > 0 && rows > 0 {
            stdout.execute(Clear(ClearType::All)).map_err(PlayError::Terminal)?;
        }
        print!("{buf}");
        if cols > 0 && rows > 0 {
            stdout.execute(crossterm::cursor::MoveTo(0, 0)).map_err(PlayError::Terminal)?;
        }
        std::io::Write::flush(&mut stdout).map_err(PlayError::Terminal)?;
        std::thread::sleep(frame_duration);
    }

    match join.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(PlayError::Decoder(e)),
        Err(_) => Err(PlayError::DecoderPanic),
    }
}

/// Errors from the play pipeline.
#[derive(Debug, thiserror::Error)]
pub enum PlayError {
    #[error("decoder: {0}")]
    Decoder(#[from] decode::DecodeError),
    #[error("decoder thread panicked")]
    DecoderPanic,
    #[error("terminal: {0}")]
    Terminal(#[from] std::io::Error),
}
