use std::io::{self, Write};
use std::time::Instant;

pub struct ProgressBar {
    total_size: Option<usize>,
    current_chunk: usize,
    start_time: Instant,
    description: Option<String>,
}

impl ProgressBar {
    pub fn new(total_size: Option<usize>) -> Self {
        ProgressBar {
            total_size,
            current_chunk: 0,
            start_time: Instant::now(),
            description: None,
        }
    }

    pub fn update(&mut self, bytes_processed: usize) {
        self.current_chunk += bytes_processed;
        self.render();
    }

    pub fn set_description(&mut self, desc: String) {
        self.description = Some(desc);
    }

    pub fn finish(&self) {
        eprintln!();
    }

    fn render(&self) {
        let elapsed = self.start_time.elapsed();
        let speed = if elapsed.as_secs() > 0 {
            self.current_chunk as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let speed_mb = speed / (1024.0 * 1024.0);

        let desc = match &self.description {
            Some(d) => format!("{}: ", d),
            None => String::new(),
        };

        let output = match self.total_size {
            Some(total) if total > 0 => {
                // Known total size - show percentage bar
                let percentage = (self.current_chunk as f64 / total as f64) * 100.0;
                let bar_width = 40;
                let filled = ((percentage / 100.0) * bar_width as f64) as usize;
                let bar = "█".repeat(filled) + &"░".repeat(bar_width - filled);

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
                // Unknown total size - show spinning indicator
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
