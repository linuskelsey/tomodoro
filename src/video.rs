use std::{
    io::{self, Read},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

pub struct RgbFrame {
    pub pixels: Vec<u8>, // RGB24, row-major
    pub width: usize,
    pub height: usize,
}

impl RgbFrame {
    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let idx = (y * self.width + x) * 3;
        (self.pixels[idx], self.pixels[idx + 1], self.pixels[idx + 2])
    }
}

pub struct VideoAnimation {
    frames: Vec<RgbFrame>,
    sixel_cache: Vec<Option<String>>,
    pub fps: f64,
    start: Option<Instant>,
    paused_elapsed: Duration,
}

impl VideoAnimation {
    pub fn load(path: &str, target_w: usize, target_h: usize) -> io::Result<Self> {
        let (_, _, fps) = probe_video(path)?;
        eprintln!("Decoding at {}x{}...", target_w, target_h);
        let frames = decode_frames(path, target_w, target_h)?;
        let n = frames.len();
        eprintln!("{} frames decoded.", n);
        Ok(Self {
            sixel_cache: vec![None; n],
            frames,
            fps,
            start: None,
            paused_elapsed: Duration::ZERO,
        })
    }

    pub fn play(&mut self) {
        let already_elapsed = self.paused_elapsed;
        self.start = Some(Instant::now() - already_elapsed);
    }

    pub fn pause(&mut self) {
        if let Some(s) = self.start.take() {
            self.paused_elapsed = s.elapsed();
        }
    }

    pub fn current_frame(&self) -> &RgbFrame {
        &self.frames[self.current_index()]
    }

    /// Lazily encode the current frame to sixel on first access.
    pub fn current_sixel(&mut self) -> &str {
        let idx = self.current_index();
        if self.sixel_cache[idx].is_none() {
            self.sixel_cache[idx] = Some(crate::sixel::encode(&self.frames[idx]));
        }
        self.sixel_cache[idx].as_deref().unwrap()
    }

    fn current_index(&self) -> usize {
        let elapsed = match self.start {
            Some(s) => s.elapsed().as_secs_f64(),
            None => self.paused_elapsed.as_secs_f64(),
        };
        ((elapsed * self.fps) as usize) % self.frames.len().max(1)
    }
}

fn probe_video(path: &str) -> io::Result<(usize, usize, f64)> {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate",
            "-of", "csv=p=0",
            path,
        ])
        .output()?;

    let s = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<&str> = s.trim().split(',').collect();

    if parts.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::Other, "ffprobe returned no stream data"));
    }

    let width: usize = parts[0].trim().parse()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "could not parse video width"))?;
    let height: usize = parts[1].trim().parse()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "could not parse video height"))?;

    let fps = parts.get(2).map(|s| parse_fps(s.trim())).unwrap_or(30.0);
    Ok((width, height, fps))
}

fn parse_fps(s: &str) -> f64 {
    if let Some((n, d)) = s.split_once('/') {
        let n: f64 = n.parse().unwrap_or(30.0);
        let d: f64 = d.parse().unwrap_or(1.0);
        if d > 0.0 { n / d } else { 30.0 }
    } else {
        s.parse().unwrap_or(30.0)
    }
}

fn decode_frames(path: &str, target_w: usize, target_h: usize) -> io::Result<Vec<RgbFrame>> {
    let scale_filter = format!("scale={}:{}", target_w, target_h);
    let mut child = Command::new("ffmpeg")
        .args([
            "-i", path,
            "-vf", &scale_filter,
            "-f", "rawvideo",
            "-pix_fmt", "rgb24",
            "pipe:1",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let frame_size = target_w * target_h * 3;
    let mut buf = Vec::new();
    child.stdout.take().unwrap().read_to_end(&mut buf)?;
    child.wait()?;

    if buf.len() < frame_size {
        return Err(io::Error::new(io::ErrorKind::Other, "no frames decoded from video"));
    }

    Ok(buf
        .chunks_exact(frame_size)
        .map(|chunk| RgbFrame { pixels: chunk.to_vec(), width: target_w, height: target_h })
        .collect())
}
