# MuyZipido💨

Simple Rust library to stream and decompress zip files without loading everything into memory.

Works well with zip files that have a corrupt central directory.

Uses local file headers to process and decompess data on the fly.

Optional progress bar - still in development.

```rust
use muy_zipido::{
    MuyZipido,
    progress_bar::{Colour, Style},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "URL_HERE";
    println!("Fetching and processing ZIP from: {}", url);

    let extractor = MuyZipido::new(url, 10240)?.with_progress(Style::Blocks, Colour::Magenta);

    let mut total_entries = 0;
    let mut total_bytes = 0;

    for entry_result in extractor {
        match entry_result {
            Ok(entry) => {
                total_entries += 1;
                total_bytes += entry.data.len();

                println!(
                    "Entry {}: {} ({} bytes)",
                    total_entries,
                    entry.filename,
                    entry.data.len()
                );
            }
            Err(e) => {
                eprintln!("Error processing entry: {}", e);
                break;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total entries: {}", total_entries);
    println!("Total bytes processed: {}", total_bytes);

    Ok(())
}
```
