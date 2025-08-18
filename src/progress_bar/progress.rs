use super::style::Style;
use std::io::{self, Write};
use std::time::{Duration, Instant};

pub struct ProgressBar {
    total_size: Option<usize>,
    current_chunk: usize,
    start_time: Instant,
    description: Option<String>,
    last_render_time: Instant,
    min_render_interval: Duration,
    smoothed_speed: Option<f64>,
    smoothing_factor: f64,
    style: Style,
    use_colour: Colour,
}

const RESET: &str = "\x1b[0m";

impl ProgressBar {
    pub fn new(total_size: Option<usize>) -> Self {
        let now = Instant::now();
        ProgressBar {
            total_size,
            current_chunk: 0,
            start_time: now,
            description: None,
            last_render_time: now,
            min_render_interval: Duration::from_millis(100),
            smoothed_speed: None,
            smoothing_factor: 0.3,
            style: Style::default(),
            use_colour: Colour::default(),
        }
    }

    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_color(mut self, color: Colour) -> Self {
        self.use_colour = color;
        self
    }

    pub fn update(&mut self, bytes_processed: usize) {
        self.current_chunk += bytes_processed;

        let elapsed = self.start_time.elapsed();
        let instant_speed = if elapsed.as_secs_f64() > 0.0 {
            self.current_chunk as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        self.smoothed_speed = match self.smoothed_speed {
            None => Some(instant_speed),
            Some(prev_speed) => {
                let beta = self.smoothing_factor;
                Some(instant_speed * beta + prev_speed * (1.0 - beta))
            }
        };

        let now = Instant::now();
        if now.duration_since(self.last_render_time) >= self.min_render_interval {
            self.render();
            self.last_render_time = now;
        }
    }

    pub fn finish(&mut self) {
        self.render();
        eprintln!();
    }

    fn render(&self) {
        let elapsed = self.start_time.elapsed();
        let speed = self.smoothed_speed.unwrap_or(0.0);
        let speed_mb = speed / (1024.0 * 1024.0);
        let desc = match &self.description {
            Some(d) => format!("{}: ", d),
            None => String::new(),
        };

        let output = match self.total_size {
            Some(total) if total > 0 => {
                let percentage = (self.current_chunk as f64 / total as f64) * 100.0;
                let bar_width = 40;
                let filled = ((percentage / 100.0) * bar_width as f64) as usize;
                let bar = match self.use_colour {
                    Colour::None => {
                        // No color
                        self.style.filled_char().to_string().repeat(filled)
                            + &self
                                .style
                                .empty_char()
                                .to_string()
                                .repeat(bar_width - filled)
                    }
                    _ => {
                        // With color
                        format!(
                            "{}{}{}{}",
                            self.use_colour.ansi_code(),
                            self.style.filled_char().to_string().repeat(filled),
                            RESET,
                            self.style
                                .empty_char()
                                .to_string()
                                .repeat(bar_width - filled)
                        )
                    }
                };

                let eta_secs = if speed > 0.0 && total > self.current_chunk {
                    (total - self.current_chunk) as f64 / speed
                } else {
                    0.0
                };

                format!(
                    "\r{}[{}] {:.1}% | {}/{} | {:.2} MB/s | ETA: {:.0}s",
                    desc,
                    bar,
                    percentage,
                    format_bytes(self.current_chunk),
                    format_bytes(total),
                    speed_mb,
                    eta_secs
                )
            }
            _ => {
                let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                let spinner_idx = (elapsed.as_millis() / 100) % spinner_chars.len() as u128;
                let spinner = spinner_chars[spinner_idx as usize];

                format!(
                    "\r{}{} {} | {:.2} MB/s | {}",
                    desc,
                    spinner,
                    format_bytes(self.current_chunk),
                    speed_mb,
                    format_elapsed(elapsed)
                )
            }
        };

        eprint!("{}", output);
        let _ = io::stderr().flush();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Colour {
    None,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl Colour {
    pub fn ansi_code(&self) -> &'static str {
        match self {
            Colour::None => "",
            Colour::Red => "\x1b[31m",
            Colour::Green => "\x1b[32m",
            Colour::Yellow => "\x1b[33m",
            Colour::Blue => "\x1b[34m",
            Colour::Magenta => "\x1b[35m",
            Colour::Cyan => "\x1b[36m",
            Colour::White => "\x1b[37m",
        }
    }
}

impl Default for Colour {
    fn default() -> Self {
        Colour::Cyan
    }
}

fn format_bytes(bytes: usize) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{:.0}{}", size, UNITS[unit_idx])
    } else {
        format!("{:.1}{}", size, UNITS[unit_idx])
    }
}

fn format_elapsed(elapsed: std::time::Duration) -> String {
    let total_secs = elapsed.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}
