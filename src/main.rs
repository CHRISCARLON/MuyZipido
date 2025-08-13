use muy_zipido::MuyZipido;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://opendata.manage-roadworks.service.gov.uk/permit/2020/07.zip";
    println!("Fetching and processing ZIP from: {}", url);

    let extractor = MuyZipido::new(url, 8192)?;

    let mut total_entries = 0;
    let mut total_bytes = 0;

    // Process entries one at a time as they stream in
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

                // Entry is dropped here, freeing memory before next entry is read
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
