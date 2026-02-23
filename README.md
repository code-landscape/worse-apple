# Worse Apple

Play your favorite video in your favorite terminal. In ASCII art.

## Usage

```bash
cargo run -- path/to/video.mp4
```

Optional `--fps` to set playback speed (default 24):

```bash
cargo run -- video.mp4 --fps 30
```

Terminal size is detected automatically. If stdout isn’t a TTY, it falls back to 80×24.

## How it works

The video is decoded on a separate thread and sent over a small bounded channel. The main thread turns each frame into a grid of characters (using a simple luminance-to-character ramp), clears the screen, prints the frame, and sleeps until the next one. Only a few frames are decoded ahead, so memory use stays low.

## Prior art

[tplay](https://github.com/maxcurzi/tplay) is a more capable alternative; the main drawback, if any, is its heavy dependencies (especially OpenCV).
