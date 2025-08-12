# MuyZipido

Simple Rust library to stream and decompress zip files without loading everything into memory.

Works well with zip files that have a corrupt central directory.

Uses local file headers to process and decompess data on the fly.

```rust
use muy_zipido::MuyZipido;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://opendata.manage-roadworks.service.gov.uk/permit/2020/07.zip";

    println!("Fetching and processing ZIP from: {}", url);

    let mut extractor = MuyZipido::new(url, 8192)?;
    let entries = extractor.process()?;

    println!("\n=== Summary ===");
    println!("Total entries: {}", entries.len());

    Ok(())
}
```
