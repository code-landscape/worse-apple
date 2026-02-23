//! Map a decoded frame to an ASCII buffer that fills the terminal.
//!
//! Samples the frame on a cols×rows grid (one sample per cell), uses luminance (Y plane for
//! YUV, grayscale from RGB), and maps to a character ramp.

use ffmpeg_next::frame::Video as VideoFrame;
use ffmpeg_next::format::Pixel;

/// Character ramp from dark to bright (index 0 = black, last = white).
const RAMP: &[u8] = b" .,:;+*#%@";

/// Converts a video frame to an ASCII art string that fits (cols, rows).
///
/// Samples one pixel per output cell; supports YUV420P, YUVJ420P, RGB24, BGR24, and GRAY8.
/// Unsupported formats return a placeholder grid.
pub fn frame_to_ascii(frame: &VideoFrame, cols: u16, rows: u16) -> String {
    let cols = cols as usize;
    let rows = rows as usize;
    if cols == 0 || rows == 0 {
        return String::new();
    }
    let w = frame.width() as usize;
    let h = frame.height() as usize;
    if w == 0 || h == 0 {
        return "?\n".repeat(rows);
    }

    let mut out = String::with_capacity(rows * (cols + 1));
    for r in 0..rows {
        let y_src = (r * h) / rows;
        for c in 0..cols {
            let x_src = (c * w) / cols;
            let lum = sample_luminance(frame, x_src, y_src);
            let idx = (lum as usize * (RAMP.len().saturating_sub(1))) / 256;
            out.push(RAMP[idx.min(RAMP.len() - 1)] as char);
        }
        out.push('\n');
    }
    out
}

/// Samples luminance at (x, y). Returns 0..256 (256 = white).
fn sample_luminance(frame: &VideoFrame, x: usize, y: usize) -> u8 {
    match frame.format() {
        Pixel::YUV420P | Pixel::YUVJ420P => {
            let stride = frame.stride(0);
            let data = frame.data(0);
            let idx = y * stride + x;
            if idx < data.len() {
                data[idx]
            } else {
                0
            }
        }
        Pixel::GRAY8 => {
            let stride = frame.stride(0);
            let data = frame.data(0);
            let idx = y * stride + x;
            if idx < data.len() {
                data[idx]
            } else {
                0
            }
        }
        Pixel::RGB24 => {
            let stride = frame.stride(0);
            let data = frame.data(0);
            let idx = y * stride + x * 3;
            if idx + 2 < data.len() {
                let r = data[idx] as u32;
                let g = data[idx + 1] as u32;
                let b = data[idx + 2] as u32;
                (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) as u8
            } else {
                0
            }
        }
        Pixel::BGR24 => {
            let stride = frame.stride(0);
            let data = frame.data(0);
            let idx = y * stride + x * 3;
            if idx + 2 < data.len() {
                let b = data[idx] as u32;
                let g = data[idx + 1] as u32;
                let r = data[idx + 2] as u32;
                (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) as u8
            } else {
                0
            }
        }
        _ => {
            // Unsupported format; use plane 0 as raw bytes if 1 byte per pixel
            let stride = frame.stride(0);
            let data = frame.data(0);
            let idx = y * stride + x;
            if idx < data.len() {
                data[idx]
            } else {
                0
            }
        }
    }
}
