pub mod circular_buffer;
pub mod progress_bar;

use circular_buffer::CircularBuffer;
use flate2::read::DeflateDecoder;
use progress_bar::ProgressBar;
use std::error::Error;
use std::fmt;
use std::io::Read;

#[derive(Debug)]
pub enum ZipError {
    Http(reqwest::Error),
    UnexpectedEof,
    InvalidSignature(String),
    Io(std::io::Error),
    Decompression(String),
}

impl fmt::Display for ZipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZipError::Http(e) => write!(f, "HTTP error: {}", e),
            ZipError::UnexpectedEof => write!(f, "Unexpected end of stream"),
            ZipError::InvalidSignature(sig) => write!(f, "Invalid signature: {}", sig),
            ZipError::Io(e) => write!(f, "IO error: {}", e),
            ZipError::Decompression(e) => write!(f, "Decompression error: {}", e),
        }
    }
}

impl Error for ZipError {}

impl From<reqwest::Error> for ZipError {
    fn from(e: reqwest::Error) -> Self {
        ZipError::Http(e)
    }
}

impl From<std::io::Error> for ZipError {
    fn from(e: std::io::Error) -> Self {
        ZipError::Io(e)
    }
}

pub struct ZipEntry {
    pub filename: String,
    pub uncompressed_size: u32,
    pub data: Vec<u8>,
}

pub struct MuyZipido {
    response: Option<reqwest::blocking::Response>,
    chunk_size: usize,
    buffer: Vec<u8>,
    offset: usize,
    finished: bool,
    progress_bar: Option<ProgressBar>,
}

impl MuyZipido {
    pub fn new(url: &str, chunk_size: usize) -> Result<Self, ZipError> {
        let response = reqwest::blocking::get(url)?;

        if !response.status().is_success() {
            return Err(ZipError::Http(response.error_for_status().unwrap_err()));
        }

        Ok(Self {
            response: Some(response),
            chunk_size,
            buffer: Vec::new(),
            offset: 0,
            finished: false,
            progress_bar: None,
        })
    }

    pub fn with_progress(
        mut self,
        style: progress_bar::Style,
        color: progress_bar::Colour,
    ) -> Self {
        let content_length = if let Some(response) = &self.response {
            response
                .headers()
                .get("content-length")
                .and_then(|value| value.to_str().ok())
                .and_then(|s| s.parse::<usize>().ok())
        } else {
            None
        };

        let progress_bar = ProgressBar::new(content_length)
            .with_description("Downloading ZIP".to_string())
            .with_style(style)
            .with_color(color);
        self.progress_bar = Some(progress_bar);
        self
    }

    fn read_exact(&mut self, size: usize) -> Result<Vec<u8>, ZipError> {
        while self.buffer.len() < size {
            if let Some(response) = &mut self.response {
                let mut chunk = vec![0u8; self.chunk_size];
                let bytes_read = response.read(&mut chunk)?;

                if bytes_read == 0 {
                    return Err(ZipError::UnexpectedEof);
                }

                chunk.truncate(bytes_read);
                self.buffer.extend_from_slice(&chunk);

                if let Some(ref mut progress_bar) = self.progress_bar {
                    progress_bar.update(bytes_read);
                }
            } else {
                return Err(ZipError::UnexpectedEof);
            }
        }

        let data = self.buffer[..size].to_vec();
        self.buffer.drain(..size);
        self.offset += size;

        Ok(data)
    }

    fn process_with_descriptor(&mut self, compression: u16) -> Result<Vec<u8>, ZipError> {
        const DATA_DESC_SIG: [u8; 4] = [0x50, 0x4b, 0x07, 0x08];

        let mut data = Vec::new();
        let mut sig_buffer: CircularBuffer<u8> = CircularBuffer::new(4);

        if compression == 8 {
            let mut compressed_data = Vec::new();

            loop {
                let byte = self.read_exact(1)?[0];
                compressed_data.push(byte);
                sig_buffer.write(byte);

                if sig_buffer.len() >= 4 {
                    let last_4 = sig_buffer.get_last_n(4);
                    if last_4.as_slice() == DATA_DESC_SIG {
                        compressed_data.truncate(compressed_data.len() - 4);

                    let mut decoder = DeflateDecoder::new(&compressed_data[..]);
                    decoder.read_to_end(&mut data)?;

                    let _crc = self.read_exact(4)?;
                    let _compressed_size = self.read_exact(4)?;
                    let _uncompressed_size = self.read_exact(4)?;

                        break;
                    }
                }

                if compressed_data.len() > 100_000_000 {
                    return Err(ZipError::Decompression(
                        "Data descriptor not found within reasonable limit".to_string(),
                    ));
                }
            }
        } else if compression == 0 {
            loop {
                let byte = self.read_exact(1)?[0];
                data.push(byte);
                sig_buffer.write(byte);

                if sig_buffer.len() >= 4 {
                    let last_4 = sig_buffer.get_last_n(4);
                    if last_4.as_slice() == DATA_DESC_SIG {
                        data.truncate(data.len() - 4);

                    let _crc = self.read_exact(4)?;
                    let _compressed_size = self.read_exact(4)?;
                    let _uncompressed_size = self.read_exact(4)?;

                        break;
                    }
                }

                if data.len() > 100_000_000 {
                    return Err(ZipError::Decompression(
                        "Data descriptor not found within reasonable limit".to_string(),
                    ));
                }
            }
        } else {
            return Err(ZipError::Decompression(format!(
                "Unsupported compression method: {}",
                compression
            )));
        }

        Ok(data)
    }

    fn process_next_entry(&mut self) -> Result<Option<ZipEntry>, ZipError> {
        const LOCAL_FILE_HEADER_SIG: &[u8] = b"PK\x03\x04";
        const CENTRAL_DIR_SIG: &[u8] = b"PK\x01\x02";
        const END_CENTRAL_DIR_SIG: &[u8] = b"PK\x05\x06";

        if self.finished {
            return Ok(None);
        }

        let sig = self.read_exact(4)?;

        if sig == CENTRAL_DIR_SIG || sig == END_CENTRAL_DIR_SIG {
            println!("Reached end of local file entries");
            self.finished = true;
            return Ok(None);
        }

        if sig != LOCAL_FILE_HEADER_SIG {
            let mut hex_string = String::with_capacity(sig.len() * 2);
            for b in &sig {
                hex_string.push_str(&format!("{:02x}", b));
            }
            return Err(ZipError::InvalidSignature(hex_string));
        }

        let header_data = self.read_exact(26)?;
        let _version = u16::from_le_bytes([header_data[0], header_data[1]]);
        let flags = u16::from_le_bytes([header_data[2], header_data[3]]);
        let compression = u16::from_le_bytes([header_data[4], header_data[5]]);
        let _mod_time = u16::from_le_bytes([header_data[6], header_data[7]]);
        let _mod_date = u16::from_le_bytes([header_data[8], header_data[9]]);
        let _crc32 = u32::from_le_bytes([
            header_data[10],
            header_data[11],
            header_data[12],
            header_data[13],
        ]);
        let compressed_size = u32::from_le_bytes([
            header_data[14],
            header_data[15],
            header_data[16],
            header_data[17],
        ]);
        let uncompressed_size = u32::from_le_bytes([
            header_data[18],
            header_data[19],
            header_data[20],
            header_data[21],
        ]);
        let filename_len = u16::from_le_bytes([header_data[22], header_data[23]]);
        let extra_len = u16::from_le_bytes([header_data[24], header_data[25]]);

        let filename_bytes = self.read_exact(filename_len as usize)?;
        let filename = String::from_utf8_lossy(&filename_bytes).to_string();
        let _extra_field = self.read_exact(extra_len as usize)?;

        let has_data_descriptor = (flags & 0x08) != 0;

        println!("\nProcessing: {}", filename);
        println!("  Compression: {} (0=none, 8=deflate)", compression);

        let data = if !has_data_descriptor && compressed_size > 0 {
            let compressed_data = self.read_exact(compressed_size as usize)?;

            match compression {
                0 => compressed_data,
                8 => {
                    let mut decoder = DeflateDecoder::new(&compressed_data[..]);
                    let mut decompressed = Vec::new();
                    decoder.read_to_end(&mut decompressed)?;
                    decompressed
                }
                _ => {
                    return Err(ZipError::Decompression(format!(
                        "Unsupported compression method: {}",
                        compression
                    )));
                }
            }
        } else if has_data_descriptor {
            println!("  Streaming with data descriptor...");
            self.process_with_descriptor(compression)?
        } else {
            Vec::new()
        };

        println!("  Processed {} bytes", data.len());

        Ok(Some(ZipEntry {
            filename,
            uncompressed_size,
            data,
        }))
    }
}

impl Drop for MuyZipido {
    fn drop(&mut self) {
        if let Some(ref mut progress_bar) = self.progress_bar {
            progress_bar.finish();
        }
    }
}

impl Iterator for MuyZipido {
    type Item = Result<ZipEntry, ZipError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.process_next_entry() {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => None,
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}
