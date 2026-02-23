//! Lazy MP4 decoding via ffmpeg-next.
//!
//! A decoder thread opens the file, finds the video stream, and sends decoded frames on a
//! channel. Backpressure (blocking send) limits how many frames are decoded ahead.

use std::path::Path;
use std::sync::mpsc::SyncSender;
use std::thread;

use ffmpeg_next::codec::context::Context as CodecContext;
use ffmpeg_next::format;
use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::media::Type;
use thiserror::Error;
use tracing::instrument;

/// Errors produced while opening or running the decoder.
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("ffmpeg init failed")]
    Init,
    #[error("open input: {0}")]
    OpenInput(#[from] ffmpeg_next::Error),
    #[error("no video stream")]
    NoVideoStream,
    #[error("decoder: {0}")]
    Decoder(ffmpeg_next::Error),
}

/// Runs the decode loop in the current thread. Call from a dedicated thread.
///
/// Opens `path`, decodes the best video stream, and sends each decoded frame on `tx`.
/// When the stream ends or an error occurs, the sender is dropped (receiver will get `RecvError`).
#[instrument(skip(tx))]
pub fn run(path: &Path, tx: SyncSender<VideoFrame>) -> Result<(), DecodeError> {
    ffmpeg_next::init().map_err(|_| DecodeError::Init)?;

    let mut ictx = format::input(path).map_err(DecodeError::OpenInput)?;
    let stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or(DecodeError::NoVideoStream)?;
    let video_stream_index = stream.index();
    let mut decoder = CodecContext::from_parameters(stream.parameters())
        .map_err(DecodeError::OpenInput)?
        .decoder()
        .video()
        .map_err(DecodeError::Decoder)?;

    let receive_frame = |dec: &mut ffmpeg_next::decoder::Video| -> Result<(), DecodeError> {
        let mut frame = VideoFrame::empty();
        while dec.receive_frame(&mut frame).is_ok() {
            if tx.send(frame.clone()).is_err() {
                return Ok(());
            }
            frame = VideoFrame::empty();
        }
        Ok(())
    };

    for (stream, packet) in ictx.packets() {
        if stream.index() != video_stream_index {
            continue;
        }
        decoder.send_packet(&packet).map_err(DecodeError::Decoder)?;
        receive_frame(&mut decoder)?;
    }
    decoder.send_eof().map_err(DecodeError::Decoder)?;
    receive_frame(&mut decoder)?;

    Ok(())
}

/// Spawns a thread that decodes the file and sends frames on `tx`.
/// Returns the join handle; when dropped or joined, the thread stops (channel disconnect).
pub fn spawn(path: impl AsRef<Path> + Send + 'static, tx: SyncSender<VideoFrame>) -> thread::JoinHandle<Result<(), DecodeError>> {
    let path = path.as_ref().to_path_buf();
    thread::spawn(move || run(&path, tx))
}
