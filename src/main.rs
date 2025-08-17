use muy_zipido::MuyZipido;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.os.uk/downloads/v1/products/CodePointOpen/downloads?area=GB&format=GeoPackage&redirect";
    println!("Fetching and processing ZIP from: {}", url);

    let extractor = MuyZipido::new(url, 8192)?.with_progress();

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
